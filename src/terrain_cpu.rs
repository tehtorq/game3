use glam::Vec2;
use miniquad::*;
use std::collections::HashMap;
use crate::terrain_chunk::{TerrainChunk, LodLevel};
use crate::terrain_generation::get_blended_biome_height;
use crate::terrain_batch::TerrainBatch;

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
    batch_high: TerrainBatch,
    batch_medium: TerrainBatch,
    batch_low: TerrainBatch,
    batches_dirty: bool,
    frame_count: u32,
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
            max_cached_chunks: 2000, // Keep up to 2000 chunks in memory for larger view distance
            chunks_to_update: Vec::new(),
            update_index: 0,
            batch_high: TerrainBatch::new(LodLevel::High),
            batch_medium: TerrainBatch::new(LodLevel::Medium),
            batch_low: TerrainBatch::new(LodLevel::Low),
            batches_dirty: true,
            frame_count: 0,
        };
        
        // Generate initial chunks around origin
        terrain.update_for_player_position(ctx, 0.0, 0.0, 0.0);
        
        terrain
    }
    
    pub fn update_for_player_position(&mut self, ctx: &mut dyn RenderingBackend, player_x: f32, player_z: f32, player_rotation: f32) {
        self.frame_count += 1;
        let update_start = miniquad::date::now();
        
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
            
            // Generate all chunks within view distance CIRCLE (not square!)
            let view_dist_sq = (self.view_distance * self.view_distance) as f32;
            for z in -self.view_distance..=self.view_distance {
                for x in -self.view_distance..=self.view_distance {
                    // Only include chunks within circular radius
                    let dist_sq = (x * x + z * z) as f32;
                    if dist_sq <= view_dist_sq {
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
        
        // Only iterate through chunks if we need to generate or update
        if need_generation || needs_active_update {
            // Generate/update chunks as needed
            let mut updates_this_frame = 0;
            const MAX_UPDATES_PER_FRAME: i32 = 50; // Allow more on first frames
            
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
                                (LodLevel::High, LodLevel::Medium) => distance > 880.0, // 10% past threshold
                                (LodLevel::Medium, LodLevel::High) => distance < 720.0, // 10% before threshold
                                (LodLevel::Medium, LodLevel::Low) => distance > 1760.0,
                                (LodLevel::Low, LodLevel::Medium) => distance < 1440.0,
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
        
        // Rebuild batches if needed
        if self.batches_dirty {
            // Remove debug logging
            self.rebuild_batches(ctx);
            self.batches_dirty = false;
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
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: [[f32; 4]; 4], color: [f32; 3]) {
        static mut FRAME_COUNT: u32 = 0;
        static mut TOTAL_TIME: f64 = 0.0;
        let start_time = miniquad::date::now();
        
        // Create uniforms once
        let uniforms = crate::shader::Uniforms::new(
            glam::Mat4::from_cols_array_2d(&mvp),
            color
        );
        
        // Draw each batch with a single draw call
        let triangles_high = self.batch_high.draw(ctx, pipeline, &uniforms);
        let triangles_medium = self.batch_medium.draw(ctx, pipeline, &uniforms);
        let triangles_low = self.batch_low.draw(ctx, pipeline, &uniforms);
        
        let total_triangles = triangles_high + triangles_medium + triangles_low;
        let draw_time = miniquad::date::now() - start_time;
        
        unsafe {
            FRAME_COUNT += 1;
            TOTAL_TIME += draw_time;
            if FRAME_COUNT % 300 == 0 {  // Less frequent logging
                let avg_time = TOTAL_TIME / 300.0;
                println!("Terrain: {} triangles in 3 draw calls, avg: {:.1}ms", 
                    total_triangles, avg_time * 1000.0);
                TOTAL_TIME = 0.0;
            }
        }
    }
    
    pub fn terrain_scale(&self) -> f32 {
        self.terrain_scale
    }
    
    fn rebuild_batches(&mut self, ctx: &mut dyn RenderingBackend) {
        let start_time = miniquad::date::now();
        
        // Clear all batches
        self.batch_high.clear();
        self.batch_medium.clear();
        self.batch_low.clear();
        
        // Add chunks to appropriate batches
        for &(cx, cz) in &self.active_chunks {
            if let Some(chunk) = self.chunk_cache.get(&(cx, cz)) {
                match chunk.lod {
                    LodLevel::High => self.batch_high.add_chunk(&chunk.vertices, &chunk.indices),
                    LodLevel::Medium => self.batch_medium.add_chunk(&chunk.vertices, &chunk.indices),
                    LodLevel::Low => self.batch_low.add_chunk(&chunk.vertices, &chunk.indices),
                }
            }
        }
        
        let batch_time = miniquad::date::now() - start_time;
        
        // Upload batches to GPU
        self.batch_high.upload_to_gpu(ctx);
        self.batch_medium.upload_to_gpu(ctx);
        self.batch_low.upload_to_gpu(ctx);
        
        let upload_time = miniquad::date::now() - start_time - batch_time;
        
        if self.frame_count % 600 == 0 {  // Very infrequent
            println!("Batch rebuild: {:.1}ms", (batch_time + upload_time) * 1000.0);
        }
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