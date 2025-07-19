use glam::Vec2;
use miniquad::*;
use std::collections::HashMap;
use crate::terrain::{Chunk as TerrainChunk, chunk::LodLevel};
use crate::terrain::generation::get_blended_biome_height;
use crate::terrain_gpu_batch::TerrainGPUBatch;
use crate::terrain::predictive::PredictiveLoader;

const CHUNK_SIZE: f32 = 160.0; // Larger chunks for better performance

// Export the height calculation function for use by other modules
pub fn calculate_terrain_height(x: f32, z: f32) -> f32 {
    get_blended_biome_height(Vec2::new(x, z))
}

pub struct TerrainCPU {
    chunk_cache: HashMap<(i32, i32), TerrainChunk>,
    terrain_scale: f32,
    view_distance: i32,
    current_center_x: i32,
    current_center_z: i32,
    active_chunks: Vec<(i32, i32)>,
    max_cached_chunks: usize,
    chunks_to_update: Vec<(i32, i32)>,
    update_index: usize,
    // Batches for each LOD level
    batch_high: TerrainGPUBatch,
    batch_medium: TerrainGPUBatch,
    batch_low: TerrainGPUBatch,
    batches_dirty: bool,
    batch_rebuild_cooldown: u32,
    frame_count: u32,
    predictive_loader: PredictiveLoader,
}

impl TerrainCPU {
    pub fn new(ctx: &mut dyn RenderingBackend, view_distance: i32) -> Self {
        let mut terrain = Self {
            chunk_cache: HashMap::new(),
            terrain_scale: view_distance as f32 * CHUNK_SIZE,
            view_distance, // Use the provided view distance
            current_center_x: i32::MAX,
            current_center_z: i32::MAX,
            active_chunks: Vec::new(),
            max_cached_chunks: 8000, // Reduce memory pressure
            chunks_to_update: Vec::new(),
            update_index: 0,
            // Pre-allocate with more realistic sizes based on sparse loading
            batch_high: TerrainGPUBatch::new(LodLevel::High),
            batch_medium: TerrainGPUBatch::new(LodLevel::Medium),
            batch_low: TerrainGPUBatch::new(LodLevel::Low),
            batches_dirty: true,
            batch_rebuild_cooldown: 0,
            frame_count: 0,
            predictive_loader: PredictiveLoader::new(CHUNK_SIZE),
        };
        
        // Generate initial chunks around origin
        terrain.update_for_player_position(ctx, 0.0, 0.0, 0.0);
        
        terrain
    }
    
    pub fn update_for_player_position(&mut self, ctx: &mut dyn RenderingBackend, player_x: f32, player_z: f32, player_rotation: f32) {
        self.frame_count += 1;
        
        // Only update predictive loader every 10 frames to reduce overhead
        if self.frame_count % 10 == 0 {
            let update_start = miniquad::date::now();
            let player_pos = glam::Vec3::new(player_x, 0.0, player_z);
            self.predictive_loader.update(player_pos, update_start);
        }
        
        let chunk_x = (player_x / CHUNK_SIZE).floor() as i32;
        let chunk_z = (player_z / CHUNK_SIZE).floor() as i32;
        
        // Check if we need to update active chunks list
        let needs_active_update = chunk_x != self.current_center_x || chunk_z != self.current_center_z;
        
        if needs_active_update {
            self.current_center_x = chunk_x;
            self.current_center_z = chunk_z;
        }
        
        // Only recalculate active chunks if player moved to new chunk
        if needs_active_update || self.active_chunks.is_empty() {
            let mut new_active_chunks = Vec::new();
            
            // Generate chunks with sparse loading for distant areas
            let view_dist_sq = (self.view_distance * self.view_distance) as f32;
            for z in -self.view_distance..=self.view_distance {
                for x in -self.view_distance..=self.view_distance {
                    let dist_sq = (x * x + z * z) as f32;
                    
                    // Skip chunks outside circular radius
                    if dist_sq > view_dist_sq {
                        continue;
                    }
                    
                    // More aggressive sparse loading for very distant chunks
                    let distance = dist_sq.sqrt();
                    let skip = if distance > self.view_distance as f32 * 0.85 {
                        // Very far: only every 4th chunk
                        (x.abs() + z.abs()) % 4 != 0
                    } else if distance > self.view_distance as f32 * 0.7 {
                        // Far: only every 3rd chunk
                        (x.abs() + z.abs()) % 3 != 0
                    } else if distance > self.view_distance as f32 * 0.5 {
                        // Medium: only every 2nd chunk
                        (x.abs() + z.abs()) % 2 != 0
                    } else {
                        false // Near: all chunks
                    };
                    
                    if !skip {
                        let cx = chunk_x + x;
                        let cz = chunk_z + z;
                        new_active_chunks.push((cx, cz));
                    }
                }
            }
            
            self.active_chunks = new_active_chunks;
        }
        
        // Sort active chunks by distance to prioritize closer chunks
        let mut sorted_chunks = self.active_chunks.clone();
        sorted_chunks.sort_by_key(|&(cx, cz)| {
            let dx = cx - chunk_x;
            let dz = cz - chunk_z;
            dx * dx + dz * dz // Distance squared (no need for sqrt)
        });
        
        // Check if we need to generate any new chunks first
        let mut need_generation = false;
        for &(cx, cz) in &self.active_chunks {
            if !self.chunk_cache.contains_key(&(cx, cz)) {
                need_generation = true;
                break;
            }
        }
        
        // Get priority chunks from predictive loader only when we have capacity
        let priority_chunks = if self.frame_count % 10 == 0 {
            self.predictive_loader.get_priority_chunks(10)
        } else {
            vec![]  
        };
        
        // Only iterate through chunks if we need to generate or update
        if need_generation || needs_active_update || !priority_chunks.is_empty() {
            // Generate/update chunks as needed
            let mut updates_this_frame = 0;
            const MAX_UPDATES_PER_FRAME: i32 = 5; // Further reduce to prevent stuttering
            
            // First, handle predictive chunks
            for &(cx, cz) in &priority_chunks {
                if updates_this_frame >= MAX_UPDATES_PER_FRAME {
                    break;
                }
                
                let chunk_key = (cx, cz);
                if !self.chunk_cache.contains_key(&chunk_key) {
                    // Calculate distance for LOD
                    let dx = cx - chunk_x;
                    let dz = cz - chunk_z;
                    let distance = ((dx * dx + dz * dz) as f32).sqrt() * CHUNK_SIZE;
                    let lod = LodLevel::from_distance(distance);
                    
                    // Create chunk
                    let chunk = TerrainChunk::new(cx, cz, lod, CHUNK_SIZE);
                    self.chunk_cache.insert(chunk_key, chunk);
                    updates_this_frame += 1;
                    
                    // Add to active chunks if in range
                    let dist_sq = (dx * dx + dz * dz) as f32;
                    let view_dist_sq = (self.view_distance * self.view_distance) as f32;
                    if dist_sq <= view_dist_sq {
                        if !self.active_chunks.contains(&chunk_key) {
                            self.active_chunks.push(chunk_key);
                            self.batches_dirty = true;
                        }
                    }
                }
            }
            
            // Then handle regular chunks
            for &(cx, cz) in &sorted_chunks {
                let chunk_key = (cx, cz);
                
                if !self.chunk_cache.contains_key(&chunk_key) {
                    // Only create new chunks if we haven't hit the update limit
                    if updates_this_frame < MAX_UPDATES_PER_FRAME {
                        // Calculate distance for LOD
                        let dx = cx - chunk_x;
                        let dz = cz - chunk_z;
                        let distance = ((dx * dx + dz * dz) as f32).sqrt() * CHUNK_SIZE;
                        let lod = LodLevel::from_distance(distance);
                        
                        // Create new chunk
                        let chunk = TerrainChunk::new(cx, cz, lod, CHUNK_SIZE);
                        self.chunk_cache.insert(chunk_key, chunk);
                        self.batches_dirty = true;
                        updates_this_frame += 1;
                    }
                } else if needs_active_update {
                    // Only check LOD updates when player moves to new chunk
                    let dx = cx - chunk_x;
                    let dz = cz - chunk_z;
                    let distance = ((dx * dx + dz * dz) as f32).sqrt() * CHUNK_SIZE;
                    let new_lod = LodLevel::from_distance(distance);
                    
                    if let Some(chunk) = self.chunk_cache.get_mut(&chunk_key) {
                        // Only update if LOD level actually changed
                        let current_lod = chunk.lod;
                        if !matches!((current_lod, new_lod), (a, b) if a as u8 == b as u8) {
                            // Add hysteresis - only switch if we're well past the threshold
                            let should_update = match (current_lod, new_lod) {
                                (LodLevel::High, LodLevel::Medium) => distance > 500.0,   // 100 units past threshold
                                (LodLevel::Medium, LodLevel::High) => distance < 300.0,   // 100 units before threshold
                                (LodLevel::Medium, LodLevel::Low) => distance > 1400.0,   // 200 units past threshold
                                (LodLevel::Low, LodLevel::Medium) => distance < 1000.0,   // 200 units before threshold
                                _ => true,
                            };
                            
                            if should_update && updates_this_frame < MAX_UPDATES_PER_FRAME {
                                chunk.update_lod(new_lod, CHUNK_SIZE);
                                self.batches_dirty = true;
                                updates_this_frame += 1;
                            }
                        }
                    }
                }
            }
        }
        
        // No complex LOD blending needed - coordinate snapping handles alignment
        
        // Update cooldown
        if self.batch_rebuild_cooldown > 0 {
            self.batch_rebuild_cooldown -= 1;
        }
        
        // Rebuild batches if needed, but with longer cooldown to prevent stuttering
        if self.batches_dirty && self.batch_rebuild_cooldown == 0 {
            self.rebuild_batches(ctx);
            self.batches_dirty = false;
            self.batch_rebuild_cooldown = 30; // Wait 30 frames (0.5 sec) before next rebuild
        }
        
        // Remove chunks that are too far away
        if self.chunk_cache.len() > self.max_cached_chunks {
            let mut chunks_to_remove = Vec::new();
            
            for (key, _) in &self.chunk_cache {
                if !self.active_chunks.contains(key) {
                    chunks_to_remove.push(*key);
                }
            }
            
            // Sort by distance and remove the farthest ones
            chunks_to_remove.sort_by_key(|&(cx, cz)| {
                let dx = cx - chunk_x;
                let dz = cz - chunk_z;
                -(dx * dx + dz * dz) // Negative for reverse sort
            });
            
            let remove_count = self.chunk_cache.len() - self.max_cached_chunks;
            for key in chunks_to_remove.into_iter().take(remove_count) {
                self.chunk_cache.remove(&key);
            }
        }
        
        // Remove debug timing
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: [[f32; 4]; 4], color: [f32; 3], player_pos: glam::Vec3) -> i32 {
        static mut FRAME_COUNT: u32 = 0;
        static mut TOTAL_TIME: f64 = 0.0;
        let start_time = miniquad::date::now();
        
        let mvp_matrix = glam::Mat4::from_cols_array_2d(&mvp);
        
        // Draw each batch with GPU terrain generation
        let triangles_high = self.batch_high.draw(ctx, pipeline, mvp_matrix, color, CHUNK_SIZE, player_pos);
        let triangles_medium = self.batch_medium.draw(ctx, pipeline, mvp_matrix, color, CHUNK_SIZE, player_pos);
        let triangles_low = self.batch_low.draw(ctx, pipeline, mvp_matrix, color, CHUNK_SIZE, player_pos);
        
        let total_triangles = triangles_high + triangles_medium + triangles_low;
        let draw_time = miniquad::date::now() - start_time;
        
        unsafe {
            FRAME_COUNT += 1;
            TOTAL_TIME += draw_time;
            if FRAME_COUNT % 600 == 0 {  // Even less frequent logging
                let avg_time = TOTAL_TIME / 600.0;
                println!("GPU Terrain: {} triangles, avg: {:.1}ms", 
                    total_triangles, avg_time * 1000.0);
                TOTAL_TIME = 0.0;
            }
        }
        
        total_triangles
    }
    
    pub fn terrain_scale(&self) -> f32 {
        self.terrain_scale
    }
    
    pub fn active_chunks_count(&self) -> usize {
        self.active_chunks.len()
    }
    
    fn rebuild_batches(&mut self, ctx: &mut dyn RenderingBackend) {
        // Clear all batches
        self.batch_high.clear();
        self.batch_medium.clear();
        self.batch_low.clear();
        
        // Add chunks to appropriate batches
        for &(cx, cz) in &self.active_chunks {
            if let Some(chunk) = self.chunk_cache.get(&(cx, cz)) {
                // For now, no morphing between chunks (will be calculated in shader)
                let morph_factor = 1.0;
                match chunk.lod {
                    LodLevel::High => self.batch_high.add_chunk(chunk, morph_factor),
                    LodLevel::Medium => self.batch_medium.add_chunk(chunk, morph_factor),
                    LodLevel::Low => self.batch_low.add_chunk(chunk, morph_factor),
                }
            }
        }
        
        // Upload batches to GPU
        self.batch_high.upload_to_gpu(ctx);
        self.batch_medium.upload_to_gpu(ctx);
        self.batch_low.upload_to_gpu(ctx);
    }
    
    // Force generation of all initial chunks without update limits
    fn force_initial_generation(&mut self, ctx: &mut dyn RenderingBackend, player_x: f32, player_z: f32, player_rotation: f32) {
        let chunk_x = (player_x / CHUNK_SIZE).floor() as i32;
        let chunk_z = (player_z / CHUNK_SIZE).floor() as i32;
        
        self.current_center_x = chunk_x;
        self.current_center_z = chunk_z;
        
        // Generate all chunks within view distance
        for z in -self.view_distance..=self.view_distance {
            for x in -self.view_distance..=self.view_distance {
                let cx = chunk_x + x;
                let cz = chunk_z + z;
                
                // Use a slightly larger radius to ensure full coverage
                let dist_sq = (x * x + z * z) as f32;
                if dist_sq <= ((self.view_distance as f32 * 1.2) * (self.view_distance as f32 * 1.2)) {
                    let chunk_key = (cx, cz);
                    self.active_chunks.push(chunk_key);
                    
                    // Calculate distance for LOD
                    let dx = cx - chunk_x;
                    let dz = cz - chunk_z;
                    let distance = ((dx * dx + dz * dz) as f32).sqrt() * CHUNK_SIZE;
                    let lod = LodLevel::from_distance(distance);
                    
                    // Create chunk
                    let chunk = TerrainChunk::new(cx, cz, lod, CHUNK_SIZE);
                    self.chunk_cache.insert(chunk_key, chunk);
                    self.batches_dirty = true;
                }
            }
        }
        
    }
}