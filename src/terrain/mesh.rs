use miniquad::*;
use glam::Vec2;
use crate::vertex::Vertex;
use crate::biome::{BiomeMap, Biome};
use super::generation::height_at;
use super::biome_heights::smoothstep;
use crate::constants::*;


pub struct Terrain {
    base_mesh_vertex_buffer: BufferId,
    base_mesh_index_buffer: BufferId,
    instance_buffer: BufferId,
    height_texture: TextureId,
    biome_texture: TextureId,
    index_count: i32,
    instance_count: i32,
    terrain_scale: f32,
    biome_map: BiomeMap,
    view_distance: i32,
}

impl Terrain {
    pub fn new(ctx: &mut dyn RenderingBackend, view_distance: i32) -> Self {
        Self::new_with_seed(ctx, view_distance, 12345)
    }
    
    pub fn new_from_cache(
        ctx: &mut dyn RenderingBackend, 
        view_distance: i32, 
        height_data: Vec<u8>, 
        biome_data: Vec<u8>,
        texture_size: u32
    ) -> Self {
        // Create base mesh (single chunk template)
        let (vertices, indices) = Self::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance data (chunk positions)
        let instance_data = Self::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2;
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        
        // Create textures from cached data
        let height_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&height_data),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: texture_size,
                height: texture_size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        let biome_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&biome_data),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: texture_size,
                height: texture_size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        // Create a dummy biome map (not needed when loading from cache)
        let biome_map = BiomeMap::new(terrain_scale, 0);
        
        println!("Loaded terrain from cache");
        println!("  Instances: {}", instance_count);
        println!("  Terrain scale: {}", terrain_scale);
        println!("  Texture size: {}x{}", texture_size, texture_size);
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    pub fn new_with_seed_and_save(
        ctx: &mut dyn RenderingBackend, 
        view_distance: i32, 
        seed: u32,
        save_name: Option<&str>
    ) -> Self {
        use super::cache::TerrainCache;
        
        // Create base mesh (single chunk template)
        let (vertices, indices) = Self::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance data (chunk positions)
        let instance_data = Self::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2;
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        // Create biome map
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Create height and biome textures
        let texture_size = TERRAIN_TEXTURE_SIZE;
        let (height_texture, biome_texture, height_data, biome_data) = Self::create_terrain_textures(ctx, texture_size, terrain_scale, &biome_map);
        
        // Save textures if requested
        if let Some(name) = save_name {
            if let Err(e) = TerrainCache::save_terrain_textures(name, &height_data, &biome_data, texture_size) {
                eprintln!("Failed to save terrain textures: {}", e);
            }
        }
        
        println!("Created instanced terrain:");
        println!("  Base mesh: {} vertices, {} indices", vertices.len(), indices.len());
        println!("  Instances: {}", instance_count);
        println!("  Terrain scale: {}", terrain_scale);
        println!("  Texture size: {}x{}", texture_size, texture_size);
        println!("  Biome regions created");
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    pub fn new_with_seed(ctx: &mut dyn RenderingBackend, view_distance: i32, seed: u32) -> Self {
        // Create base mesh (single chunk template)
        let (vertices, indices) = Self::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance data (chunk positions)
        let instance_data = Self::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2; // 2 floats per instance
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        // Create biome map
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Create height and biome textures
        let texture_size = TERRAIN_TEXTURE_SIZE;
        let (height_texture, biome_texture, height_data, biome_data) = Self::create_terrain_textures(ctx, texture_size, terrain_scale, &biome_map);
        
        println!("Created instanced terrain:");
        println!("  Base mesh: {} vertices, {} indices", vertices.len(), indices.len());
        println!("  Instances: {}", instance_count);
        println!("  Terrain scale: {}", terrain_scale);
        println!("  Texture size: {}x{}", texture_size, texture_size);
        println!("  Biome regions created");
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    fn create_base_mesh() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let grid_size = CHUNK_GRID_SIZE;
        let cell_size = CHUNK_SIZE / grid_size as f32;
        let half_size = CHUNK_SIZE / 2.0;
        
        // Generate triangles with proper barycentric coordinates
        for i in 0..grid_size {
            for j in 0..grid_size {
                let base_vertex_idx = vertices.len() as u32;
                
                // Calculate positions for this cell
                let x0 = -half_size + (i as f32) * cell_size;
                let x1 = -half_size + ((i + 1) as f32) * cell_size;
                let z0 = -half_size + (j as f32) * cell_size;
                let z1 = -half_size + ((j + 1) as f32) * cell_size;
                let y = 0.0; // Ground level
                
                // Debug first cell
                if i == 0 && j == 0 {
                    println!("First cell corners: ({}, {}, {}) to ({}, {}, {})", x0, y, z0, x1, y, z1);
                }
                
                // First triangle - counter-clockwise when viewed from above
                vertices.push(Vertex::with_barycentric(x0, y, z0, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(x1, y, z0, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(x0, y, z1, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx);
                indices.push(base_vertex_idx + 1);
                indices.push(base_vertex_idx + 2);
                
                // Second triangle - counter-clockwise when viewed from above
                vertices.push(Vertex::with_barycentric(x1, y, z0, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(x1, y, z1, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(x0, y, z1, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx + 3);
                indices.push(base_vertex_idx + 4);
                indices.push(base_vertex_idx + 5);
            }
        }
        
        println!("Base mesh created: {} vertices, {} indices", vertices.len(), indices.len());
        
        (vertices, indices)
    }
    
    fn create_instance_data(view_distance: i32) -> Vec<f32> {
        let mut instance_data = Vec::new();
        let chunk_size = CHUNK_SIZE;
        let max_distance_chunks = view_distance as f32 + 0.5;
        
        let mut count = 0;
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                // Check if chunk is within circular distance
                let chunk_dist_sq = (x * x + z * z) as f32;
                if chunk_dist_sq <= max_distance_chunks * max_distance_chunks {
                    instance_data.push(x as f32 * chunk_size);
                    instance_data.push(z as f32 * chunk_size);
                    count += 1;
                    
                    // Debug first few instances
                    if count <= 5 {
                        println!("Instance {}: offset ({}, {})", count - 1, x as f32 * chunk_size, z as f32 * chunk_size);
                    }
                }
            }
        }
        
        println!("Total instances: {}", count);
        instance_data
    }
    
    pub fn get_texture_data(&self) -> (Vec<u8>, Vec<u8>, u32) {
        // This would require storing the texture data in the struct
        // For now, we'll handle this differently
        (vec![], vec![], 2048)
    }
    
    fn create_terrain_textures(ctx: &mut dyn RenderingBackend, size: u32, terrain_scale: f32, biome_map: &BiomeMap) -> (TextureId, TextureId, Vec<u8>, Vec<u8>) {
        let mut height_data = vec![0u8; (size * size * 4) as usize]; // RGBA format
        let mut biome_data = vec![0u8; (size * size * 4) as usize]; // RGBA format for biome info
        
        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;
        
        // Generate height and biome values
        println!("Generating terrain textures...");
        let total_pixels = size * size;
        let mut pixels_processed = 0u32;
        
        for y in 0..size {
            if y % 100 == 0 {
                let progress = (y as f32 / size as f32 * 100.0) as u32;
                println!("  Progress: {}%", progress);
            }
            
            for x in 0..size {
                // Convert texture coordinates to world coordinates
                let u = x as f32 / (size - 1) as f32;
                let v = y as f32 / (size - 1) as f32;
                
                let world_x = (u - 0.5) * terrain_scale;
                let world_z = (v - 0.5) * terrain_scale;
                
                // Get biome weights for smooth transitions
                let biome_weights = biome_map.get_biome_weights(world_x, world_z);
                
                // Calculate blended height using biome weights
                let height = Self::height_at_with_biomes(world_x, world_z, &biome_weights);
                min_height = min_height.min(height);
                max_height = max_height.max(height);
                
                // Store height in height texture with better precision
                // Use all color channels for better precision (24-bit instead of 8-bit)
                let normalized_height = ((height + 500.0) / 1000.0).clamp(0.0, 1.0);
                let height_24bit = (normalized_height * 16777215.0) as u32; // 2^24 - 1
                
                let height_idx = ((y * size + x) * 4) as usize;
                height_data[height_idx] = ((height_24bit >> 16) & 0xFF) as u8;     // R - high byte
                height_data[height_idx + 1] = ((height_24bit >> 8) & 0xFF) as u8;  // G - middle byte
                height_data[height_idx + 2] = (height_24bit & 0xFF) as u8;         // B - low byte
                height_data[height_idx + 3] = 255;                                 // A
                
                // Store biome color blend in biome texture
                let mut color = [0.0, 0.0, 0.0];
                for (biome, weight) in &biome_weights {
                    let biome_color = biome.color_tint();
                    color[0] += biome_color[0] * weight;
                    color[1] += biome_color[1] * weight;
                    color[2] += biome_color[2] * weight;
                }
                
                let biome_idx = ((y * size + x) * 4) as usize;
                biome_data[biome_idx] = (color[0] * 255.0) as u8;     // R
                biome_data[biome_idx + 1] = (color[1] * 255.0) as u8; // G
                biome_data[biome_idx + 2] = (color[2] * 255.0) as u8; // B
                biome_data[biome_idx + 3] = 255;                      // A
                
                pixels_processed += 1;
            }
        }
        
        println!("  Progress: 100%");
        println!("Height texture stats: min={:.2}, max={:.2}", min_height, max_height);
        
        // Debug: sample some points to verify texture data
        for i in 0..5 {
            let idx = i * 100 * 4; // Sample every 100 pixels
            if idx < height_data.len() {
                let r = height_data[idx] as u32;
                let g = height_data[idx + 1] as u32;
                let b = height_data[idx + 2] as u32;
                let height_24bit = (r << 16) | (g << 8) | b;
                let normalized = height_24bit as f32 / 16777215.0;
                let world_height = normalized * 1000.0 - 500.0;
                println!("  Height sample {}: RGB({},{},{}) = {:.2}", i, r, g, b, world_height);
            }
        }
        
        let height_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&height_data),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: size,
                height: size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        let biome_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&biome_data),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: size,
                height: size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        (height_texture, biome_texture, height_data, biome_data)
    }
    
    fn height_at_with_biomes(x: f32, z: f32, biome_weights: &[(Biome, f32)]) -> f32 {
        // Use the shader-matched implementation for perfect CPU/GPU parity
        Self::get_blended_biome_height(Vec2::new(x, z))
    }
    
    fn biome_height(x: f32, z: f32, biome: &Biome, params: &crate::biome::BiomeHeightParams) -> f32 {
        match biome {
            Biome::Plains => {
                // Rolling plains with more variation
                let scale1 = 0.002;
                let scale2 = 0.007;
                let scale3 = 0.015;
                
                // Larger rolling hills
                let h1 = (x * scale1).sin() * (z * scale1).cos() * 40.0;
                let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * 20.0;
                let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * 10.0;
                
                // Add some occasional low ridges
                let ridge = ((x * 0.0005 + z * 0.0003).sin()).abs().powf(3.0) * 30.0;
                
                // Gentle valleys and depressions
                let depression = smoothstep(0.6, 0.3, ((x * 0.0008).sin() * (z * 0.0006).cos()).abs()) * -20.0;
                
                h1 + h2 + h3 + ridge + depression
            },
            Biome::Canyon => {
                // River-like canyon systems with dramatic depth variations
                let scale1 = 0.0015; // Main river course
                let scale2 = 0.003;  // Tributaries
                let scale3 = 0.006;  // Rapids and falls
                let scale4 = 0.0005; // Canyon width variation
                
                // Main river channel - continuous flowing pattern
                let river_flow = (x * scale1).sin() * 0.7 + (z * scale1 * 0.8).cos() * 0.5;
                let river_meander = ((x * scale1 * 0.5 + z * scale1 * 0.3).sin() + 
                                    (x * scale1 * 0.3 - z * scale1 * 0.4).cos()) * 0.4;
                
                // Tributary channels joining the main river
                let tributary1 = ((x * scale2 - z * scale2 * 0.6).sin() + 
                                 (x * scale2 * 0.4 + z * scale2).cos()) * 0.3;
                let tributary2 = ((x * scale2 * 1.2 + z * scale2 * 0.5).sin() * 
                                 (x * scale2 * 0.8 - z * scale2 * 0.7).cos()) * 0.25;
                
                // River confluence points - deeper where rivers meet
                let confluence = ((tributary1 * river_flow).abs() + (tributary2 * river_flow).abs()) * 0.5;
                
                // Canyon width varies like a real river
                let width_pattern = (x * scale4 + z * scale4 * 0.7).sin();
                let canyon_width = 0.3 + width_pattern.abs() * 0.7 + confluence * 0.3;
                
                // River depth with pools and rapids
                let pool_pattern = ((x * scale3).sin() * (z * scale3 * 1.2).cos()).abs();
                let rapids = ((x * scale3 * 2.0 + z * scale3 * 1.5).sin()).abs().powf(3.0) * 0.3;
                
                // Combine all river features
                let river_depth = (river_flow + river_meander).abs() * canyon_width + 
                                 tributary1.abs() * 0.5 + tributary2.abs() * 0.5 + 
                                 confluence + pool_pattern * 0.4 - rapids;
                
                // Create dramatic canyon walls with overhangs
                let wall_slope = 1.5 + width_pattern * 0.5;
                let canyon_cut = river_depth.abs().powf(wall_slope) * 2.0; // Deeper canyons
                
                // Terraced canyon walls
                let terraces = ((canyon_cut * 6.0).floor() / 6.0).max(0.0);
                let final_depth = canyon_cut * 0.4 + terraces * 0.6;
                
                // High mesas between canyons
                let mesa_height = ((x * scale4 * 0.5).sin().powi(2) + (z * scale4 * 0.5).cos().powi(2)) * 40.0;
                let plateau_base = 200.0; // Base height for dramatic effect
                
                // Create dramatic height difference
                plateau_base + mesa_height - final_depth * 160.0
            },
            Biome::Plateau => {
                // Dramatic mesa and plateau formations with sheer cliffs
                let scale1 = 0.0008;
                let scale2 = 0.0005;
                let scale3 = 0.002;
                let scale4 = 0.0003;
                
                // Create distinct mesa formations
                let mesa1 = ((x * scale1).sin() * (z * scale1 * 0.9).cos()).abs();
                let mesa2 = ((x * scale2 + 200.0).sin() * (z * scale2 - 150.0).cos()).abs();
                let mesa3 = ((x * scale4 * 1.3).cos() * (z * scale4 + 100.0).sin()).abs();
                
                // Sharp cliff edges
                let cliff_sharpness = 8.0; // Very sharp transitions
                let mesa_top1 = if mesa1 > 0.4 { 1.0 } else { (mesa1 / 0.4).powf(cliff_sharpness) };
                let mesa_top2 = if mesa2 > 0.5 { 1.0 } else { (mesa2 / 0.5).powf(cliff_sharpness) };
                let mesa_top3 = if mesa3 > 0.6 { 1.0 } else { (mesa3 / 0.6).powf(cliff_sharpness) };
                
                // Dramatic height differences between plateau levels
                let base_elevation = -50.0;
                let tier1_height = 120.0;
                let tier2_height = 180.0;
                let tier3_height = 250.0;
                
                // Calculate mesa heights
                let h1 = base_elevation + mesa_top1 * tier1_height;
                let h2 = base_elevation + mesa_top2 * tier2_height;
                let h3 = base_elevation + mesa_top3 * tier3_height;
                
                // Natural bridges and arches
                let arch_pattern = ((x * scale3 + z * scale3 * 0.7).sin() * 
                                   (x * scale3 * 1.2 - z * scale3 * 0.5).cos()).abs();
                let arch_cut = if arch_pattern > 0.7 { arch_pattern.powf(4.0) * -50.0 } else { 0.0 };
                
                // Rock spires and hoodoos
                let spire_pattern = ((x * scale3 * 2.0).sin() * (z * scale3 * 2.0).cos()).abs();
                let spires = spire_pattern.powf(6.0) * 40.0;
                
                // Weathering and erosion patterns
                let erosion = ((x * 0.01).sin() + (z * 0.01).cos()) * 10.0 * (1.0 - mesa_top1.max(mesa_top2).max(mesa_top3));
                
                // Combine all plateau features
                let height = h1.max(h2).max(h3) + spires + arch_cut + erosion;
                
                // Add dramatic vertical relief
                height.clamp(-100.0, 350.0)
            },
            Biome::Crystalline => {
                // Varied spiky crystal formations
                let scale1 = 0.01;
                let scale2 = 0.02;
                let scale3 = 0.05;
                let scale4 = 0.007;
                
                // Vary spike sharpness based on position
                let sharpness1 = 0.8 + ((x * 0.001).sin() * (z * 0.001).cos() * 0.4);
                let sharpness2 = 1.2 + ((x * 0.002 + 100.0).sin() * (z * 0.002).cos() * 0.6);
                
                // Different crystal cluster patterns
                let spike1 = ((x * scale1).sin() * (z * scale1).cos()).abs().powf(sharpness1) * 150.0;
                let spike2 = ((x * scale2 + 50.0).cos() * (z * scale2 - 30.0).sin()).abs().powf(sharpness2) * 105.0;
                let spike3 = ((x * scale3 - 20.0).sin() * (z * scale3 + 40.0).cos()).abs() * 60.0;
                
                // Add larger crystal formations
                let large_crystal = ((x * scale4).sin().powi(2) + (z * scale4).cos().powi(2)).sqrt();
                let crystal_height = (1.0 - large_crystal).max(0.0).powf(1.5) * 120.0;
                
                // Base elevation variation
                let base_variation = (x * 0.003).sin() * (z * 0.003).cos() * 15.0;
                
                let height = spike1 + spike2 + spike3 + crystal_height + base_variation;
                height.clamp(-100.0, 400.0)
            },
            Biome::Volcanic => {
                // Rough terrain with crater-like formations
                let scale1 = 0.004;
                let scale2 = 0.008;
                let scale3 = 0.002;
                let scale4 = 0.001;
                
                // Multiple volcanic craters with varying sizes
                let crater1 = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
                let crater2 = ((x * scale3 + 100.0).sin().powi(2) + (z * scale3 - 50.0).cos().powi(2)).sqrt();
                let crater3 = ((x * scale4 * 1.5).sin().powi(2) + (z * scale4 * 1.2).cos().powi(2)).sqrt();
                
                // Dramatic volcanic cones
                let h1 = (1.0 - crater1) * 180.0;
                let h2 = (1.0 - crater2) * 120.0;
                let h3 = (1.0 - crater3) * 250.0; // Main massive volcano
                
                // Rough lava flows and volcanic debris
                let rough = (x * scale2).sin() * (z * scale2).cos() * 80.0;
                let lava_flow = ((x * 0.003 + z * 0.002).sin()).abs() * 40.0;
                
                // Caldera formations
                let caldera = if crater1 < 0.3 { -60.0 } else { 0.0 };
                let caldera2 = if crater3 < 0.4 { -80.0 } else { 0.0 };
                
                // Volcanic ridges and fissures
                let ridge = ((x * 0.005 - z * 0.003).sin()).abs().powf(2.0) * 60.0;
                
                let base_height = h1.max(h2).max(h3) + rough + lava_flow + ridge + caldera + caldera2;
                base_height.clamp(-150.0, 350.0)
            },
            Biome::Mountains => {
                // Dramatic mountain ranges with connected peaks and ridgelines
                let scale1 = 0.0008;  // Major range direction
                let scale2 = 0.0015;  // Individual peaks
                let scale3 = 0.0003;  // Range backbone
                let scale4 = 0.004;   // Rocky details
                let scale5 = 0.0001;  // Continental scale
                
                // Major mountain range ridgeline - continuous spine
                let range_angle: f32 = 0.4; // Northwest to southeast trend
                let ridge_main = ((x * scale3 * range_angle.cos() + z * scale3 * range_angle.sin()).sin() * 0.5 + 0.5).powf(3.0);
                let ridge_secondary = ((x * scale3 * 1.2 - z * scale3 * 0.7).cos() * 0.5 + 0.5).powf(2.5);
                
                // Connected peak system along the ridges
                let peak_spacing = 0.0012;
                let peak_line1 = ((x * peak_spacing * range_angle.cos() + z * peak_spacing * range_angle.sin()).sin().powi(2) + 
                                 (x * peak_spacing * range_angle.sin() - z * peak_spacing * range_angle.cos()).cos().powi(2)).sqrt();
                let peak_line2 = ((x * peak_spacing * 1.3 + 100.0).sin().powi(2) + 
                                 (z * peak_spacing * 1.3 - 50.0).cos().powi(2)).sqrt();
                
                // Create dramatic pointed peaks
                let peak_sharpness = 2.5; // Higher = sharper peaks
                let h1 = (1.0 - peak_line1).max(0.0).powf(peak_sharpness) * 270.0;
                let h2 = (1.0 - peak_line2).max(0.0).powf(peak_sharpness * 0.8) * 225.0;
                
                // Ridge height variations - peaks are higher along the ridge
                let ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
                
                // Deep valleys between ridges
                let valley_pattern = (x * scale2 + z * scale2 * 0.6).sin() + 
                                    (x * scale2 * 0.8 - z * scale2 * 0.5).cos();
                let valley_depth = valley_pattern.abs().powf(2.0) * -60.0;
                
                // Dramatic cliffs and rock faces
                let cliff_pattern = ((x * scale4).sin() * (z * scale4 * 1.2).cos()).abs();
                let cliffs = cliff_pattern.powf(4.0) * 80.0;
                
                // Snow fields and glacial valleys
                let glacial_valley = ((x * scale2 * 0.5 + z * scale2 * 0.7).sin()).abs().powf(0.5) * -40.0;
                let snow_cap = (h1 + h2 + ridge_height).max(225.0) * 0.2;
                
                // Foothills that gradually rise to meet the mountains
                let distance_to_ridge = ((ridge_main - 0.5).abs() + (ridge_secondary - 0.5).abs()).min(1.0);
                let foothill_height = (1.0 - distance_to_ridge).powf(0.5) * 60.0;
                
                // Continental mountain building
                let tectonic = ((x * scale5).sin() + (z * scale5 * 0.8).cos()) * 45.0;
                
                let height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                            glacial_valley + snow_cap + foothill_height + tectonic;
                            
                // Ensure dramatic height variations
                height.clamp(-250.0, 450.0)
            },
            Biome::Desert => {
                // Sand dune formations with dramatic heights
                let scale1 = 0.005;
                let scale2 = 0.01;
                let scale3 = 0.03;
                
                // Large dramatic dunes
                let dunes = ((x * scale1).sin() * (z * scale1 * 1.2).cos()).abs() * 80.0;
                
                // Secondary dune fields
                let secondary = (x * scale2 + 30.0).cos() * (z * scale2 - 20.0).sin() * 30.0;
                
                // Sand ripples and waves
                let ripples = (x * scale3).sin() * (z * scale3).cos() * 10.0;
                
                // Occasional rock outcroppings
                let rocks = ((x * 0.002).sin() * (z * 0.002).cos()).abs().powf(4.0) * 60.0;
                
                // Wind-carved hollows
                let hollows = smoothstep(0.7, 0.5, ((x * 0.004 + z * 0.003).sin()).abs()) * -30.0;
                
                let height = -20.0 + dunes + secondary + ripples + rocks + hollows;
                height.clamp(-100.0, 200.0)
            },
            Biome::Arctic => {
                // Dramatic glacial formations with towering ice
                let scale1 = 0.005;
                let scale2 = 0.015;
                let scale3 = 0.03;
                let scale4 = 0.002;
                let scale5 = 0.0008;
                
                // Massive glacial sheets with dramatic elevation
                let glacier_flow = ((x * scale5).sin() + (z * scale5 * 0.7).cos()) * 120.0;
                let glacier_thickness = ((x * scale5 * 0.5).sin().powi(2) + (z * scale5 * 0.5).cos().powi(2)) * 90.0;
                
                // Towering ice spires and seracs
                let serac_sharpness = 3.0 + ((x * 0.001).sin() * (z * 0.001).cos() * 2.0);
                let seracs = ((x * scale2).sin() * (z * scale2).cos()).abs().powf(serac_sharpness) * 225.0;
                
                // Massive icebergs and pressure ridges
                let pressure_ridge1 = ((x * scale1 + z * scale1 * 0.5).sin()).abs().powf(2.0) * 180.0;
                let pressure_ridge2 = ((x * scale1 * 0.8 - z * scale1 * 0.6).cos()).abs().powf(2.0) * 135.0;
                
                // Deep crevasses and moulins
                let crevasse_pattern = (x * scale3).sin() + (z * scale3 * 1.2).cos();
                let crevasse_depth = crevasse_pattern.abs().powf(4.0) * 120.0;
                let moulin = ((x * scale2 * 2.0 + z * scale2 * 1.5).sin() * 
                             (x * scale2 * 1.5 - z * scale2 * 2.0).cos()).abs().powf(6.0) * -60.0;
                
                // Ice caverns and tunnels
                let cave_pattern = ((x * scale4).sin() * (z * scale4 * 0.8).cos()).abs();
                let ice_caves = if cave_pattern > 0.6 { cave_pattern.powf(3.0) * -40.0 } else { 0.0 };
                
                // Frozen waterfalls and ice walls
                let ice_wall = ((x * scale1 * 0.3 + z * scale1 * 0.9).sin()).abs().powf(5.0) * 105.0;
                
                let height = -20.0 + glacier_flow + glacier_thickness + 
                            seracs + pressure_ridge1 + pressure_ridge2 + ice_wall - 
                            crevasse_depth + moulin + ice_caves;
                            
                height.clamp(-150.0, 420.0)
            },
            Biome::Badlands => {
                // Dramatic eroded landscape with towering formations
                let scale1 = 0.004;
                let scale2 = 0.01;
                let scale3 = 0.025;
                let scale4 = 0.0015;
                let scale5 = 0.0006;
                
                // Massive mesa formations with sheer cliffs
                let mesa_pattern = ((x * scale1).sin() * (z * scale1 * 0.8).cos()).abs();
                let mesa_height = if mesa_pattern > 0.3 { 
                    mesa_pattern.powf(0.2) * 300.0 
                } else { 
                    mesa_pattern * 75.0 
                };
                
                // Deep erosion channels and slot canyons
                let erosion_main = (x * scale2).sin() + (z * scale2 * 1.2).cos();
                let erosion_branch = (x * scale2 * 1.5 - z * scale2 * 0.7).sin() * 
                                    (x * scale2 * 0.8 + z * scale2 * 1.3).cos();
                let slot_canyon = erosion_main.abs().powf(3.0) * 80.0 + 
                                 erosion_branch.abs().powf(4.0) * 60.0;
                
                // Towering hoodoos and rock spires
                let hoodoo_field = ((x * scale3).sin() * (z * scale3).cos()).abs();
                let hoodoo_height = hoodoo_field.powf(5.0) * 270.0;
                let spire_cluster = ((x * scale3 * 1.5 + 100.0).sin() * 
                                    (z * scale3 * 1.5 - 100.0).cos()).abs().powf(6.0) * 225.0;
                
                // Natural arches and bridges
                let arch_base = ((x * scale4 + z * scale4 * 0.6).sin() * 
                                (x * scale4 * 0.7 - z * scale4).cos()).abs();
                let arch_void = if arch_base > 0.7 && mesa_pattern > 0.5 { 
                    arch_base.powf(3.0) * -60.0 
                } else { 
                    0.0 
                };
                
                // Dramatic layered rock strata
                let strata_tilt = (x * 0.0001 + z * 0.00015).sin() * 0.3;
                let strata = (z * scale4 + x * strata_tilt).sin() * 0.5 + 0.5;
                let layer_height = (strata * 12.0).floor() * 10.0;
                
                // Scree slopes and talus fields
                let scree = (x * scale5).sin() * (z * scale5 * 1.1).cos() * 20.0 * (1.0 - mesa_pattern);
                
                let height = mesa_height + hoodoo_height + spire_cluster + 
                            layer_height + arch_void - slot_canyon + scree;
                            
                height.clamp(-250.0, 480.0)
            },
            Biome::Floating => {
                // Large floating island formations at extreme heights
                let scale1 = 0.0008;
                let scale2 = 0.0015;
                let scale3 = 0.003;
                let scale4 = 0.0002;
                
                // Main floating continents
                let continent1 = ((x * scale4).sin() * (z * scale4).cos()).abs().powf(0.5) * 200.0;
                let continent2 = ((x * scale4 * 1.3 + 100.0).cos() * (z * scale4 * 0.9 - 50.0).sin()).abs().powf(0.6) * 150.0;
                
                // Individual floating islands
                let island1 = smoothstep(0.3, 0.8, ((x * scale1).sin() * (z * scale1).cos()).abs()) * 120.0;
                let island2 = smoothstep(0.4, 0.7, ((x * scale2 + 0.8).cos() * (z * scale2 * 0.9).sin()).abs()) * 90.0;
                
                // Rocky spires on the islands
                let spires = ((x * scale3).sin() * (z * scale3 * 1.2).cos()).abs().powf(4.0) * 80.0;
                
                // Hanging gardens and waterfalls (negative values for overhangs)
                let overhang = smoothstep(0.7, 0.9, ((x * scale2 * 2.0 + z * scale2).sin()).abs()) * -40.0;
                
                // Crystal formations on underside
                let crystals = ((x * 0.01).sin() * (z * 0.01).cos()).abs().powf(3.0) * 60.0;
                
                // Base altitude for floating effect
                let base_altitude = 250.0;
                
                // Combine all features
                let height = base_altitude + continent1.max(continent2) + 
                            island1.max(island2) + spires + crystals + overhang;
                
                height.clamp(150.0, 500.0)
            },
            Biome::Caverns => {
                // Extensive underground cavern networks
                let scale1 = 0.004;
                let scale2 = 0.008;
                let scale3 = 0.02;
                let scale4 = 0.001;
                
                // Rolling karst terrain base
                let base = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 60.0;
                
                // Major sinkholes and cave entrances
                let sinkhole1 = smoothstep(0.7, 0.2, ((x * scale2).sin() * (z * scale2).cos()).abs()) * -120.0;
                let sinkhole2 = smoothstep(0.6, 0.15, ((x * scale2 * 1.3 + 1.0).cos() * (z * scale2 * 0.9).sin()).abs()) * -100.0;
                let sinkhole3 = smoothstep(0.8, 0.3, ((x * scale4 + z * scale4 * 0.5).sin()).abs()) * -150.0;
                
                // Collapsed cavern ceilings
                let collapse_pattern = ((x * scale3).sin() * (z * scale3 * 1.2).cos()).abs();
                let collapsed = if collapse_pattern > 0.6 { collapse_pattern.powf(2.0) * -80.0 } else { 0.0 };
                
                // Underground rivers and channels
                let river_channel = ((x * 0.003 + z * 0.002).sin()).abs().powf(3.0) * -40.0;
                
                // Stalactite and stalagmite fields (surface roughness)
                let formations = ((x * 0.05).sin() * (z * 0.05).cos()).abs() * 30.0;
                
                // Natural bridges over caverns
                let bridge = smoothstep(0.8, 0.95, ((x * scale2 * 0.7 - z * scale2 * 0.5).sin()).abs()) * 60.0;
                
                let height = base + sinkhole1 + sinkhole2 + sinkhole3 + 
                            collapsed + river_channel + formations + bridge;
                
                height.clamp(-300.0, 150.0)
            },
            Biome::Swamp => {
                // Murky swamp terrain with varied water features
                let scale1 = 0.005;
                let scale2 = 0.01;
                let scale3 = 0.03;
                let scale4 = 0.002;
                
                // Gentle base undulations
                let undulation = (x * scale1).sin() * (z * scale1 * 0.9).cos() * 25.0;
                
                // Deep water channels and pools
                let pools = smoothstep(0.4, 0.7, ((x * scale2).sin() * (z * scale2 * 1.1).cos()).abs()) * -40.0;
                let channels = ((x * scale4 + z * scale4 * 0.7).sin()).abs().powf(2.0) * -30.0;
                
                // Raised hummocks and dry land
                let hummocks = ((x * scale2 * 1.5).sin() * (z * scale2 * 1.3).cos()).abs().powf(3.0) * 35.0;
                
                // Dead trees and root systems (small bumps)
                let roots = ((x * scale3).sin() * (z * scale3 * 1.2).cos()).abs() * 15.0;
                
                // Bog pits and quicksand
                let bog_pattern = ((x * 0.008 - z * 0.006).sin() * (x * 0.007 + z * 0.009).cos()).abs();
                let bog_pits = if bog_pattern > 0.7 { bog_pattern.powf(2.0) * -25.0 } else { 0.0 };
                
                // Thick vegetation mounds
                let vegetation = smoothstep(0.3, 0.6, ((x * scale1 * 2.0).sin() * (z * scale1 * 1.8).cos()).abs()) * 20.0;
                
                let height = -10.0 + undulation + pools + channels + hummocks + 
                            roots + bog_pits + vegetation;
                
                height.clamp(-100.0, 80.0)
            },
        }
    }
    
    pub fn base_vertex_buffer(&self) -> BufferId {
        self.base_mesh_vertex_buffer
    }
    
    pub fn instance_buffer(&self) -> BufferId {
        self.instance_buffer
    }
    
    pub fn index_buffer(&self) -> BufferId {
        self.base_mesh_index_buffer
    }
    
    pub fn height_texture(&self) -> TextureId {
        self.height_texture
    }
    
    pub fn biome_texture(&self) -> TextureId {
        self.biome_texture
    }
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
    
    pub fn instance_count(&self) -> i32 {
        self.instance_count
    }
    
    pub fn terrain_scale(&self) -> f32 {
        self.terrain_scale
    }
    
    pub fn get_height_at(&self, x: f32, z: f32) -> f32 {
        // Use shader-matched implementation for perfect parity
        Self::get_blended_biome_height(glam::Vec2::new(x, z))
    }
    
    pub fn height_at(x: f32, z: f32) -> f32 {
        // Static version for use without instance
        Self::get_blended_biome_height(glam::Vec2::new(x, z))
    }
    
    pub fn get_biome_at(&self, x: f32, z: f32) -> Biome {
        self.biome_map.get_biome_at(x, z)
    }
    
    /// Update instance positions to be centered around the player
    /// Returns the number of visible chunks
    pub fn update_for_player_position(&mut self, ctx: &mut dyn RenderingBackend, player_x: f32, player_z: f32, player_rotation: f32) -> i32 {
        // Calculate which chunk the player is in
        let player_chunk_x = (player_x / CHUNK_SIZE).floor() as i32;
        let player_chunk_z = (player_z / CHUNK_SIZE).floor() as i32;
        
        // Generate new instance data centered on player's chunk
        let mut instance_data = Vec::new();
        
        // Calculate view direction
        let view_dir_x = -player_rotation.sin();
        let view_dir_z = -player_rotation.cos();
        
        let mut visible_chunks = 0;
        
        for x in -self.view_distance..=self.view_distance {
            for z in -self.view_distance..=self.view_distance {
                let chunk_x = player_chunk_x + x;
                let chunk_z = player_chunk_z + z;
                
                // Calculate chunk center position relative to player
                let chunk_center_x = chunk_x as f32 * CHUNK_SIZE - player_x;
                let chunk_center_z = chunk_z as f32 * CHUNK_SIZE - player_z;
                
                // Simple frustum culling: check if chunk is in front of player
                // Dot product with view direction
                let dot = chunk_center_x * view_dir_x + chunk_center_z * view_dir_z;
                
                // Only include chunks that are in front or to the sides (dot > -CHUNK_SIZE)
                if dot > -CHUNK_SIZE * 2.0 {
                    instance_data.push(chunk_x as f32 * CHUNK_SIZE);
                    instance_data.push(chunk_z as f32 * CHUNK_SIZE);
                    visible_chunks += 1;
                }
            }
        }
        
        // Update the instance buffer
        ctx.buffer_update(self.instance_buffer, BufferSource::slice(&instance_data));
        
        // Update instance count
        self.instance_count = visible_chunks;
        
        visible_chunks
    }
    
    // ===== SHADER-MATCHED HEIGHT FUNCTIONS =====
    // These functions exactly match the GLSL shader implementation for perfect CPU/GPU parity
    
    fn plains_height(p: glam::Vec2) -> f32 {
        // Rolling plains with more variation
        let scale1 = 0.002;
        let scale2 = 0.007;
        let scale3 = 0.015;
        
        // Larger rolling hills
        let h1 = (p.x * scale1).sin() * (p.y * scale1).cos() * 40.0;
        let h2 = (p.x * scale2 + 100.0).sin() * (p.y * scale2 + 100.0).sin() * 20.0;
        let h3 = (p.x * scale3 + 200.0).cos() * (p.y * scale3 + 200.0).cos() * 10.0;
        
        // Add some occasional low ridges
        let ridge = ((p.x * 0.0005 + p.y * 0.0003).sin()).abs().powf(3.0) * 30.0;
        
        // Gentle valleys and depressions
        let depression = smoothstep(0.6, 0.3, ((p.x * 0.0008).sin() * (p.y * 0.0006).cos()).abs()) * -20.0;
        
        h1 + h2 + h3 + ridge + depression
    }
    
    fn canyon_height(p: glam::Vec2) -> f32 {
        // River-like canyon systems with dramatic depth variations
        let scale1 = 0.0015; // Main river course
        let scale2 = 0.003;  // Tributaries
        let scale3 = 0.006;  // Rapids and falls
        let scale4 = 0.0005; // Canyon width variation
        
        // Main river channel - continuous flowing pattern
        let river_flow = (p.x * scale1).sin() * 0.7 + (p.y * scale1 * 0.8).cos() * 0.5;
        let river_meander = ((p.x * scale1 * 0.5 + p.y * scale1 * 0.3).sin() + 
                            (p.x * scale1 * 0.3 - p.y * scale1 * 0.4).cos()) * 0.4;
        
        // Tributary channels joining the main river
        let tributary1 = ((p.x * scale2 - p.y * scale2 * 0.6).sin() + 
                         (p.x * scale2 * 0.4 + p.y * scale2).cos()) * 0.3;
        let tributary2 = ((p.x * scale2 * 1.2 + p.y * scale2 * 0.5).sin() * 
                         (p.x * scale2 * 0.8 - p.y * scale2 * 0.7).cos()) * 0.25;
        
        // River confluence points - deeper where rivers meet
        let confluence = ((tributary1 * river_flow).abs() + (tributary2 * river_flow).abs()) * 0.5;
        
        // Canyon width varies like a real river
        let width_pattern = (p.x * scale4 + p.y * scale4 * 0.7).sin();
        let canyon_width = 0.3 + width_pattern.abs() * 0.7 + confluence * 0.3;
        
        // River depth with pools and rapids
        let pool_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs();
        let rapids = ((p.x * scale3 * 2.0 + p.y * scale3 * 1.5).sin()).abs().powf(3.0) * 0.3;
        
        // Combine all river features
        let river_depth = (river_flow + river_meander).abs() * canyon_width + 
                         tributary1.abs() * 0.5 + tributary2.abs() * 0.5 + 
                         confluence + pool_pattern * 0.4 - rapids;
        
        // Create dramatic canyon walls with overhangs
        let wall_slope = 1.5 + width_pattern * 0.5;
        let canyon_cut = river_depth.abs().powf(wall_slope) * 2.0; // Deeper canyons
        
        // Terraced canyon walls
        let terraces = ((canyon_cut * 6.0).floor() / 6.0).max(0.0);
        let final_depth = canyon_cut * 0.4 + terraces * 0.6;
        
        // High mesas between canyons
        let mesa_height = ((p.x * scale4 * 0.5).sin().powi(2) + (p.y * scale4 * 0.5).cos().powi(2)) * 40.0;
        let plateau_base = 200.0; // Base height for dramatic effect
        
        // Create dramatic height difference
        plateau_base + mesa_height - final_depth * 160.0
    }
    
    fn plateau_height(p: glam::Vec2) -> f32 {
        // Dramatic mesa and plateau formations with sheer cliffs
        let scale1 = 0.0008;
        let scale2 = 0.0005;
        let scale3 = 0.002;
        let scale4 = 0.0003;
        
        // Create distinct mesa formations
        let mesa1 = ((p.x * scale1).sin() * (p.y * scale1 * 0.9).cos()).abs();
        let mesa2 = ((p.x * scale2 + 200.0).sin() * (p.y * scale2 - 150.0).cos()).abs();
        let mesa3 = ((p.x * scale4 * 1.3).cos() * (p.y * scale4 + 100.0).sin()).abs();
        
        // Sharp cliff edges
        let cliff_sharpness = 8.0; // Very sharp transitions
        let mesa_top1 = if mesa1 > 0.4 { 1.0 } else { (mesa1 / 0.4).powf(cliff_sharpness) };
        let mesa_top2 = if mesa2 > 0.5 { 1.0 } else { (mesa2 / 0.5).powf(cliff_sharpness) };
        let mesa_top3 = if mesa3 > 0.6 { 1.0 } else { (mesa3 / 0.6).powf(cliff_sharpness) };
        
        // Dramatic height differences between plateau levels
        let base_elevation = -50.0;
        let tier1_height = 120.0;
        let tier2_height = 180.0;
        let tier3_height = 250.0;
        
        // Calculate mesa heights
        let h1 = base_elevation + mesa_top1 * tier1_height;
        let h2 = base_elevation + mesa_top2 * tier2_height;
        let h3 = base_elevation + mesa_top3 * tier3_height;
        
        // Natural bridges and arches
        let arch_pattern = ((p.x * scale3 + p.y * scale3 * 0.7).sin() * 
                           (p.x * scale3 * 1.2 - p.y * scale3 * 0.5).cos()).abs();
        let arch_cut = if arch_pattern > 0.7 { arch_pattern.powf(4.0) * -50.0 } else { 0.0 };
        
        // Rock spires and hoodoos
        let spire_pattern = ((p.x * scale3 * 2.0).sin() * (p.y * scale3 * 2.0).cos()).abs();
        let spires = spire_pattern.powf(6.0) * 40.0;
        
        // Weathering and erosion patterns
        let erosion = ((p.x * 0.01).sin() + (p.y * 0.01).cos()) * 10.0 * (1.0 - mesa_top1.max(mesa_top2).max(mesa_top3));
        
        // Combine all plateau features
        let height = h1.max(h2).max(h3) + spires + arch_cut + erosion;
        
        // Add dramatic vertical relief
        height.clamp(-100.0, 350.0)
    }
    
    fn crystalline_height(p: glam::Vec2) -> f32 {
        // Varied spiky crystal formations
        let scale1 = 0.01;
        let scale2 = 0.02;
        let scale3 = 0.05;
        let scale4 = 0.007;
        
        // Vary spike sharpness based on position
        let sharpness1 = 0.8 + ((p.x * 0.001).sin() * (p.y * 0.001).cos() * 0.4);
        let sharpness2 = 1.2 + ((p.x * 0.002 + 100.0).sin() * (p.y * 0.002).cos() * 0.6);
        
        // Different crystal cluster patterns
        let spike1 = ((p.x * scale1).sin() * (p.y * scale1).cos()).abs().powf(sharpness1) * 150.0;
        let spike2 = ((p.x * scale2 + 50.0).cos() * (p.y * scale2 - 30.0).sin()).abs().powf(sharpness2) * 105.0;
        let spike3 = ((p.x * scale3 - 20.0).sin() * (p.y * scale3 + 40.0).cos()).abs() * 60.0;
        
        // Add larger crystal formations
        let large_crystal = ((p.x * scale4).sin().powi(2) + (p.y * scale4).cos().powi(2)).sqrt();
        let crystal_height = (1.0 - large_crystal).max(0.0).powf(1.5) * 120.0;
        
        // Base elevation variation
        let base_variation = (p.x * 0.003).sin() * (p.y * 0.003).cos() * 15.0;
        
        let height = spike1 + spike2 + spike3 + crystal_height + base_variation;
        height.clamp(-100.0, 400.0)
    }
    
    fn volcanic_height(p: glam::Vec2) -> f32 {
        // Rough terrain with crater-like formations
        let scale1 = 0.004;
        let scale2 = 0.008;
        let scale3 = 0.002;
        let scale4 = 0.001;
        
        // Multiple volcanic craters with varying sizes
        let crater1 = ((p.x * scale1).sin().powi(2) + (p.y * scale1).cos().powi(2)).sqrt();
        let crater2 = ((p.x * scale3 + 100.0).sin().powi(2) + (p.y * scale3 - 50.0).cos().powi(2)).sqrt();
        let crater3 = ((p.x * scale4 * 1.5).sin().powi(2) + (p.y * scale4 * 1.2).cos().powi(2)).sqrt();
        
        // Dramatic volcanic cones
        let h1 = (1.0 - crater1) * 180.0;
        let h2 = (1.0 - crater2) * 120.0;
        let h3 = (1.0 - crater3) * 250.0; // Main massive volcano
        
        // Rough lava flows and volcanic debris
        let rough = (p.x * scale2).sin() * (p.y * scale2).cos() * 80.0;
        let lava_flow = ((p.x * 0.003 + p.y * 0.002).sin()).abs() * 40.0;
        
        // Caldera formations
        let caldera = if crater1 < 0.3 { -60.0 } else { 0.0 };
        let caldera2 = if crater3 < 0.4 { -80.0 } else { 0.0 };
        
        // Volcanic ridges and fissures
        let ridge = ((p.x * 0.005 - p.y * 0.003).sin()).abs().powf(2.0) * 60.0;
        
        let base_height = h1.max(h2).max(h3) + rough + lava_flow + ridge + caldera + caldera2;
        base_height.clamp(-150.0, 350.0)
    }
    
    fn mountain_height(p: glam::Vec2) -> f32 {
        // Dramatic mountain ranges with connected peaks and ridgelines
        let scale1 = 0.0008;  // Major range direction
        let scale2 = 0.0015;  // Individual peaks
        let scale3 = 0.0003;  // Range backbone
        let scale4 = 0.004;   // Rocky details
        let scale5 = 0.0001;  // Continental scale
        
        // Major mountain range ridgeline - continuous spine
        let range_angle: f32 = 0.4; // Northwest to southeast trend
        let ridge_main = ((p.x * scale3 * range_angle.cos() + p.y * scale3 * range_angle.sin()).sin() * 0.5 + 0.5).powf(3.0);
        let ridge_secondary = ((p.x * scale3 * 1.2 - p.y * scale3 * 0.7).cos() * 0.5 + 0.5).powf(2.5);
        
        // Connected peak system along the ridges
        let peak_spacing = 0.0012;
        let peak_line1 = ((p.x * peak_spacing * range_angle.cos() + p.y * peak_spacing * range_angle.sin()).sin().powi(2) + 
                         (p.x * peak_spacing * range_angle.sin() - p.y * peak_spacing * range_angle.cos()).cos().powi(2)).sqrt();
        let peak_line2 = ((p.x * peak_spacing * 1.3 + 100.0).sin().powi(2) + 
                         (p.y * peak_spacing * 1.3 - 50.0).cos().powi(2)).sqrt();
        
        // Create dramatic pointed peaks
        let peak_sharpness = 2.5; // Higher = sharper peaks
        let h1 = (1.0 - peak_line1).max(0.0).powf(peak_sharpness) * 270.0;
        let h2 = (1.0 - peak_line2).max(0.0).powf(peak_sharpness * 0.8) * 225.0;
        
        // Ridge height variations - peaks are higher along the ridge
        let ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
        
        // Deep valleys between ridges
        let valley_pattern = (p.x * scale2 + p.y * scale2 * 0.6).sin() + 
                            (p.x * scale2 * 0.8 - p.y * scale2 * 0.5).cos();
        let valley_depth = valley_pattern.abs().powf(2.0) * -60.0;
        
        // Dramatic cliffs and rock faces
        let cliff_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 1.2).cos()).abs();
        let cliffs = cliff_pattern.powf(4.0) * 80.0;
        
        // Snow fields and glacial valleys
        let glacial_valley = ((p.x * scale2 * 0.5 + p.y * scale2 * 0.7).sin()).abs().powf(0.5) * -40.0;
        let snow_cap = (h1 + h2 + ridge_height).max(225.0) * 0.2;
        
        // Foothills that gradually rise to meet the mountains
        let distance_to_ridge = ((ridge_main - 0.5).abs() + (ridge_secondary - 0.5).abs()).min(1.0);
        let foothill_height = (1.0 - distance_to_ridge).powf(0.5) * 60.0;
        
        // Continental mountain building
        let tectonic = ((p.x * scale5).sin() + (p.y * scale5 * 0.8).cos()) * 45.0;
        
        let height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                    glacial_valley + snow_cap + foothill_height + tectonic;
                    
        // Ensure dramatic height variations
        height.clamp(-250.0, 450.0)
    }
    
    fn desert_height(p: glam::Vec2) -> f32 {
        // Sand dune formations with dramatic heights
        let scale1 = 0.005;
        let scale2 = 0.01;
        let scale3 = 0.03;
        
        // Large dramatic dunes
        let dunes = ((p.x * scale1).sin() * (p.y * scale1 * 1.2).cos()).abs() * 80.0;
        
        // Secondary dune fields
        let secondary = (p.x * scale2 + 30.0).cos() * (p.y * scale2 - 20.0).sin() * 30.0;
        
        // Sand ripples and waves
        let ripples = (p.x * scale3).sin() * (p.y * scale3).cos() * 10.0;
        
        // Occasional rock outcroppings
        let rocks = ((p.x * 0.002).sin() * (p.y * 0.002).cos()).abs().powf(4.0) * 60.0;
        
        // Wind-carved hollows
        let hollows = smoothstep(0.7, 0.5, ((p.x * 0.004 + p.y * 0.003).sin()).abs()) * -30.0;
        
        let height = -20.0 + dunes + secondary + ripples + rocks + hollows;
        height.clamp(-100.0, 200.0)
    }
    
    fn arctic_height(p: glam::Vec2) -> f32 {
        // Dramatic glacial formations with towering ice
        let scale1 = 0.005;
        let scale2 = 0.015;
        let scale3 = 0.03;
        let scale4 = 0.002;
        let scale5 = 0.0008;
        
        // Massive glacial sheets with dramatic elevation
        let glacier_flow = ((p.x * scale5).sin() + (p.y * scale5 * 0.7).cos()) * 120.0;
        let glacier_thickness = ((p.x * scale5 * 0.5).sin().powi(2) + (p.y * scale5 * 0.5).cos().powi(2)) * 90.0;
        
        // Towering ice spires and seracs
        let serac_sharpness = 3.0 + ((p.x * 0.001).sin() * (p.y * 0.001).cos() * 2.0);
        let seracs = ((p.x * scale2).sin() * (p.y * scale2).cos()).abs().powf(serac_sharpness) * 225.0;
        
        // Massive icebergs and pressure ridges
        let pressure_ridge1 = ((p.x * scale1 + p.y * scale1 * 0.5).sin()).abs().powf(2.0) * 180.0;
        let pressure_ridge2 = ((p.x * scale1 * 0.8 - p.y * scale1 * 0.6).cos()).abs().powf(2.0) * 135.0;
        
        // Deep crevasses and moulins
        let crevasse_pattern = (p.x * scale3).sin() + (p.y * scale3 * 1.2).cos();
        let crevasse_depth = crevasse_pattern.abs().powf(4.0) * 120.0;
        let moulin = ((p.x * scale2 * 2.0 + p.y * scale2 * 1.5).sin() * 
                     (p.x * scale2 * 1.5 - p.y * scale2 * 2.0).cos()).abs().powf(6.0) * -60.0;
        
        // Ice caverns and tunnels
        let cave_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 0.8).cos()).abs();
        let ice_caves = if cave_pattern > 0.6 { cave_pattern.powf(3.0) * -40.0 } else { 0.0 };
        
        // Frozen waterfalls and ice walls
        let ice_wall = ((p.x * scale1 * 0.3 + p.y * scale1 * 0.9).sin()).abs().powf(5.0) * 105.0;
        
        let height = -20.0 + glacier_flow + glacier_thickness + 
                    seracs + pressure_ridge1 + pressure_ridge2 + ice_wall - 
                    crevasse_depth + moulin + ice_caves;
                    
        height.clamp(-150.0, 420.0)
    }
    
    fn badlands_height(p: glam::Vec2) -> f32 {
        // Dramatic eroded landscape with towering formations
        let scale1 = 0.004;
        let scale2 = 0.01;
        let scale3 = 0.025;
        let scale4 = 0.0015;
        let scale5 = 0.0006;
        
        // Massive mesa formations with sheer cliffs
        let mesa_pattern = ((p.x * scale1).sin() * (p.y * scale1 * 0.8).cos()).abs();
        let mesa_height = if mesa_pattern > 0.3 { 
            mesa_pattern.powf(0.2) * 300.0 
        } else { 
            mesa_pattern * 75.0 
        };
        
        // Deep erosion channels and slot canyons
        let erosion_main = (p.x * scale2).sin() + (p.y * scale2 * 1.2).cos();
        let erosion_branch = (p.x * scale2 * 1.5 - p.y * scale2 * 0.7).sin() * 
                            (p.x * scale2 * 0.8 + p.y * scale2 * 1.3).cos();
        let slot_canyon = erosion_main.abs().powf(3.0) * 80.0 + 
                         erosion_branch.abs().powf(4.0) * 60.0;
        
        // Towering hoodoos and rock spires
        let hoodoo_field = ((p.x * scale3).sin() * (p.y * scale3).cos()).abs();
        let hoodoo_height = hoodoo_field.powf(5.0) * 270.0;
        let spire_cluster = ((p.x * scale3 * 1.5 + 100.0).sin() * 
                            (p.y * scale3 * 1.5 - 100.0).cos()).abs().powf(6.0) * 225.0;
        
        // Natural arches and bridges
        let arch_base = ((p.x * scale4 + p.y * scale4 * 0.6).sin() * 
                        (p.x * scale4 * 0.7 - p.y * scale4).cos()).abs();
        let arch_void = if arch_base > 0.7 && mesa_pattern > 0.5 { 
            arch_base.powf(3.0) * -60.0 
        } else { 
            0.0 
        };
        
        // Dramatic layered rock strata
        let strata_tilt = (p.x * 0.0001 + p.y * 0.00015).sin() * 0.3;
        let strata = (p.y * scale4 + p.x * strata_tilt).sin() * 0.5 + 0.5;
        let layer_height = (strata * 12.0).floor() * 10.0;
        
        // Scree slopes and talus fields
        let scree = (p.x * scale5).sin() * (p.y * scale5 * 1.1).cos() * 20.0 * (1.0 - mesa_pattern);
        
        let height = mesa_height + hoodoo_height + spire_cluster + 
                    layer_height + arch_void - slot_canyon + scree;
                    
        height.clamp(-250.0, 480.0)
    }
    
    fn floating_height(p: glam::Vec2) -> f32 {
        // Large floating island formations at extreme heights
        let scale1 = 0.0008;
        let scale2 = 0.0015;
        let scale3 = 0.003;
        let scale4 = 0.0002;
        
        // Main floating continents
        let continent1 = ((p.x * scale4).sin() * (p.y * scale4).cos()).abs().powf(0.5) * 200.0;
        let continent2 = ((p.x * scale4 * 1.3 + 100.0).cos() * (p.y * scale4 * 0.9 - 50.0).sin()).abs().powf(0.6) * 150.0;
        
        // Individual floating islands
        let island1 = smoothstep(0.3, 0.8, ((p.x * scale1).sin() * (p.y * scale1).cos()).abs()) * 120.0;
        let island2 = smoothstep(0.4, 0.7, ((p.x * scale2 + 0.8).cos() * (p.y * scale2 * 0.9).sin()).abs()) * 90.0;
        
        // Rocky spires on the islands
        let spires = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs().powf(4.0) * 80.0;
        
        // Hanging gardens and waterfalls (negative values for overhangs)
        let overhang = smoothstep(0.7, 0.9, ((p.x * scale2 * 2.0 + p.y * scale2).sin()).abs()) * -40.0;
        
        // Crystal formations on underside
        let crystals = ((p.x * 0.01).sin() * (p.y * 0.01).cos()).abs().powf(3.0) * 60.0;
        
        // Base altitude for floating effect
        let base_altitude = 250.0;
        
        // Combine all features
        let height = base_altitude + continent1.max(continent2) + 
                    island1.max(island2) + spires + crystals + overhang;
        
        height.clamp(150.0, 500.0)
    }
    
    fn caverns_height(p: glam::Vec2) -> f32 {
        // Extensive underground cavern networks
        let scale1 = 0.004;
        let scale2 = 0.008;
        let scale3 = 0.02;
        let scale4 = 0.001;
        
        // Rolling karst terrain base
        let base = (p.x * scale1).sin() * (p.y * scale1 * 0.8).cos() * 60.0;
        
        // Major sinkholes and cave entrances
        let sinkhole1 = smoothstep(0.7, 0.2, ((p.x * scale2).sin() * (p.y * scale2).cos()).abs()) * -120.0;
        let sinkhole2 = smoothstep(0.6, 0.15, ((p.x * scale2 * 1.3 + 1.0).cos() * (p.y * scale2 * 0.9).sin()).abs()) * -100.0;
        let sinkhole3 = smoothstep(0.8, 0.3, ((p.x * scale4 + p.y * scale4 * 0.5).sin()).abs()) * -150.0;
        
        // Collapsed cavern ceilings
        let collapse_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs();
        let collapsed = if collapse_pattern > 0.6 { collapse_pattern.powf(2.0) * -80.0 } else { 0.0 };
        
        // Underground rivers and channels
        let river_channel = ((p.x * 0.003 + p.y * 0.002).sin()).abs().powf(3.0) * -40.0;
        
        // Stalactite and stalagmite fields (surface roughness)
        let formations = ((p.x * 0.05).sin() * (p.y * 0.05).cos()).abs() * 30.0;
        
        // Natural bridges over caverns
        let bridge = smoothstep(0.8, 0.95, ((p.x * scale2 * 0.7 - p.y * scale2 * 0.5).sin()).abs()) * 60.0;
        
        let height = base + sinkhole1 + sinkhole2 + sinkhole3 + 
                    collapsed + river_channel + formations + bridge;
        
        height.clamp(-300.0, 150.0)
    }
    
    fn swamp_height(p: glam::Vec2) -> f32 {
        // Murky swamp terrain with varied water features
        let scale1 = 0.005;
        let scale2 = 0.01;
        let scale3 = 0.03;
        let scale4 = 0.002;
        
        // Gentle base undulations
        let undulation = (p.x * scale1).sin() * (p.y * scale1 * 0.9).cos() * 25.0;
        
        // Deep water channels and pools
        let pools = smoothstep(0.4, 0.7, ((p.x * scale2).sin() * (p.y * scale2 * 1.1).cos()).abs()) * -40.0;
        let channels = ((p.x * scale4 + p.y * scale4 * 0.7).sin()).abs().powf(2.0) * -30.0;
        
        // Raised hummocks and dry land
        let hummocks = ((p.x * scale2 * 1.5).sin() * (p.y * scale2 * 1.3).cos()).abs().powf(3.0) * 35.0;
        
        // Dead trees and root systems (small bumps)
        let roots = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs() * 15.0;
        
        // Bog pits and quicksand
        let bog_pattern = ((p.x * 0.008 - p.y * 0.006).sin() * (p.x * 0.007 + p.y * 0.009).cos()).abs();
        let bog_pits = if bog_pattern > 0.7 { bog_pattern.powf(2.0) * -25.0 } else { 0.0 };
        
        // Thick vegetation mounds
        let vegetation = smoothstep(0.3, 0.6, ((p.x * scale1 * 2.0).sin() * (p.y * scale1 * 1.8).cos()).abs()) * 20.0;
        
        let height = -10.0 + undulation + pools + channels + hummocks + 
                    roots + bog_pits + vegetation;
        
        height.clamp(-100.0, 80.0)
    }
    
    // Enhanced biome selection with more variety and smaller regions (matches shader exactly)
    fn get_biome_height(p: glam::Vec2) -> f32 {
        // Multi-scale noise for more organic biome distribution
        let noise1 = (p.x * 0.0003).sin() * (p.y * 0.0003).cos();
        let noise2 = (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin();
        let noise3 = (p.x * 0.0013 - 2.1).cos() * (p.y * 0.0011 + 1.9).sin();
        
        // Combine noises for complex patterns
        let mut biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
        
        // Add local variation for sub-biomes
        let local_var = (p.x * 0.01).sin() * (p.y * 0.01).cos() * 0.1;
        biome_noise += local_var;
        
        // 12 biome types distributed across the noise range
        if biome_noise < -0.7 {
            Self::canyon_height(p)
        } else if biome_noise < -0.5 {
            Self::caverns_height(p)
        } else if biome_noise < -0.3 {
            Self::badlands_height(p)
        } else if biome_noise < -0.1 {
            Self::plateau_height(p)
        } else if biome_noise < 0.1 {
            Self::plains_height(p)
        } else if biome_noise < 0.25 {
            Self::desert_height(p)
        } else if biome_noise < 0.4 {
            Self::swamp_height(p)
        } else if biome_noise < 0.5 {
            Self::crystalline_height(p)
        } else if biome_noise < 0.6 {
            Self::volcanic_height(p)
        } else if biome_noise < 0.7 {
            Self::arctic_height(p)
        } else if biome_noise < 0.8 {
            Self::floating_height(p)
        } else {
            Self::mountain_height(p)
        }
    }
    
    // Smooth blending between biomes (matches shader exactly)
    fn get_blended_biome_height(p: glam::Vec2) -> f32 {
        // Sample multiple nearby points for smoother transitions
        let sample_dist = 100.0; // Increased for larger features
        let h_center = Self::get_biome_height(p);
        let h_north = Self::get_biome_height(glam::Vec2::new(p.x, p.y + sample_dist));
        let h_south = Self::get_biome_height(glam::Vec2::new(p.x, p.y - sample_dist));
        let h_east = Self::get_biome_height(glam::Vec2::new(p.x + sample_dist, p.y));
        let h_west = Self::get_biome_height(glam::Vec2::new(p.x - sample_dist, p.y));
        
        // Weighted average for smoother transitions
        let primary_height = (h_center * 3.0 + h_north + h_south + h_east + h_west) / 7.0;
        
        // Enhanced fractal noise with more octaves for detail
        let mut fractal_noise = 0.0;
        let mut amplitude = 60.0; // Increased base amplitude
        let mut frequency = 0.0005;
        for i in 0..7 { // More octaves for finer detail
            fractal_noise += (p.x * frequency).sin() * (p.y * frequency).cos() * amplitude;
            fractal_noise += (p.x * frequency * 1.7 + 100.0).sin() * (p.y * frequency * 1.7 + 100.0).cos() * amplitude * 0.7;
            amplitude *= 0.45; // Slower falloff for more influence from each octave
            frequency *= 2.3;
        }
        
        // Larger scale continental features
        let continent_scale = 0.0001; // Even larger scale
        let continental = ((p.x * continent_scale).sin() * (p.y * continent_scale * 0.8).cos() + 
                          (p.x * continent_scale * 0.3).cos() * (p.y * continent_scale * 1.2).sin()) * 120.0; // More dramatic
        
        // Erosion simulation - smooth out steep areas
        let slope_factor = ((p.x * 0.005).sin() - (p.x * 0.005 + 1.0).sin()).abs() + 
                          ((p.y * 0.005).sin() - (p.y * 0.005 + 1.0).sin()).abs();
        let erosion = slope_factor.min(1.0) * 0.3;
        
        // Terracing effect - make it much more subtle
        let terrace_height = 100.0; // Increased from 40 to make terraces less frequent
        let terraced = if primary_height > 0.0 {
            let terrace_level = (primary_height / terrace_height).floor();
            let terrace_fract = (primary_height / terrace_height).fract();
            terrace_level * terrace_height + terrace_fract.powf(2.0) * terrace_height
        } else {
            primary_height
        };
        
        // Mix terraced and smooth terrain - reduce terrace influence significantly
        let terrace_influence = ((p.x * 0.001 + p.y * 0.0008).sin() * 0.5 + 0.5).clamp(0.0, 1.0) * 0.2; // Max 20% terrace influence
        let height = terraced * terrace_influence + primary_height * (1.0 - terrace_influence);
        
        height + fractal_noise + continental * (1.0 - erosion)
    }
}