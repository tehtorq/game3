use miniquad::*;
use crate::vertex::Vertex;
use crate::biome::{BiomeMap, Biome};
use crate::constants::*;

// Smoothstep function (matches GLSL smoothstep)
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub struct InstancedTerrain {
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

impl InstancedTerrain {
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
        use crate::terrain_cache::TerrainCache;
        
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
        
        let mut count = 0;
        for x in -view_distance..=view_distance {
            for z in -view_distance..=view_distance {
                instance_data.push(x as f32 * chunk_size);
                instance_data.push(z as f32 * chunk_size);
                count += 1;
                
                // Debug first few instances
                if count <= 5 {
                    println!("Instance {}: offset ({}, {})", count - 1, x as f32 * chunk_size, z as f32 * chunk_size);
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
        let mut total_height = 0.0;
        
        // Calculate base height from biomes
        for (biome, weight) in biome_weights {
            let params = biome.height_params();
            let height = Self::biome_height(x, z, biome, &params);
            total_height += height * weight;
        }
        
        // Add fractal noise for natural terrain variation
        let mut fractal_noise = 0.0;
        let mut amplitude = 40.0;
        let mut frequency = 0.0005;
        for i in 0..5 {
            fractal_noise += (x * frequency).sin() * (z * frequency).cos() * amplitude;
            fractal_noise += (x * frequency * 1.7 + 100.0).sin() * (z * frequency * 1.7 + 100.0).cos() * amplitude * 0.7;
            amplitude *= 0.5;
            frequency *= 2.2;
        }
        
        // Add large-scale terrain features
        let continent_scale = 0.0002;
        let continental = ((x * continent_scale).sin() * (z * continent_scale * 0.8).cos() + 
                          (x * continent_scale * 0.3).cos() * (z * continent_scale * 1.2).sin()) * 80.0;
        
        // Erosion simulation - smooth out steep areas
        let slope_factor = ((x * 0.005).sin() - (x * 0.005 + 1.0).sin()).abs() + 
                          ((z * 0.005).sin() - (z * 0.005 + 1.0).sin()).abs();
        let erosion = slope_factor.min(1.0) * 0.3;
        
        // Terracing effect - make it much more subtle
        let terrace_height = 100.0; // Increased from 40 to make terraces less frequent
        let terraced = if total_height > 0.0 {
            let terrace_level = (total_height / terrace_height).floor();
            let terrace_blend = (total_height / terrace_height).fract();
            // Smooth the terrace transitions
            let smooth_blend = terrace_blend * terrace_blend * (3.0 - 2.0 * terrace_blend);
            terrace_level * terrace_height + smooth_blend * terrace_height
        } else {
            total_height
        };
        
        // Mix terraced and smooth terrain - reduce terrace influence significantly
        let terrace_influence = ((x * 0.001 + z * 0.0008).sin() * 0.5 + 0.5).clamp(0.0, 1.0) * 0.2; // Max 20% terrace influence
        let height = terraced * terrace_influence + total_height * (1.0 - terrace_influence);
        
        height + fractal_noise + continental * (1.0 - erosion)
    }
    
    fn biome_height(x: f32, z: f32, biome: &Biome, params: &crate::biome::BiomeHeightParams) -> f32 {
        match biome {
            Biome::Plains => {
                // Original terrain generation for plains
                let scale1 = 0.002 * params.frequency_multiplier;
                let scale2 = 0.007 * params.frequency_multiplier;
                let scale3 = 0.015 * params.frequency_multiplier;
                
                let h1 = (x * scale1).sin() * (z * scale1).cos() * params.base_amplitude;
                let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * params.base_amplitude * 0.5;
                let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * params.base_amplitude * 0.25;
                
                h1 + h2 + h3
            },
            Biome::Canyon => {
                // Complex canyon system with varied depths and slopes
                let scale1 = 0.003 * params.frequency_multiplier;
                let scale2 = 0.002 * params.frequency_multiplier;
                let scale3 = 0.005 * params.frequency_multiplier;
                let scale4 = 0.0008 * params.frequency_multiplier;
                
                // Main canyon with varying width
                let width_var = 0.4 + ((x * 0.0005 + z * 0.0003).sin() * 0.3);
                let canyon_main = ((x * scale1).sin() + (z * scale1 * 0.7).cos()) * width_var;
                
                // Tributary canyons
                let canyon_branch = ((x * scale2 * 1.3 - z * scale2 * 0.4).sin() + 
                                   (x * scale2 * 0.5 + z * scale2 * 1.1).cos()) * 0.3;
                
                // Slot canyons (narrow deep cuts)
                let slot = ((x * scale3 + z * scale3 * 0.5).sin() * 
                           (x * scale3 * 0.7 - z * scale3).cos()).abs().powf(5.0) * 0.5;
                
                let combined = canyon_main + canyon_branch - slot;
                
                // Variable slope instead of vertical walls
                let slope_var = 1.2 + ((x * 0.001 + z * 0.0007).sin() * 0.8);
                let depth = combined.abs().powf(slope_var);
                
                // Stepped canyon walls
                let steps = ((depth * 8.0).floor() / 8.0).max(0.0);
                let smooth_depth = depth * 0.3 + steps * 0.7;
                
                // Mesa tops with erosion
                let mesa_top = ((x * scale4).sin().powi(2) + (z * scale4).cos().powi(2)).sqrt();
                let erosion = (x * 0.01).sin() * (z * 0.01).cos() * 15.0 * (1.0 - smooth_depth);
                
                params.min_height + (1.0 - smooth_depth) * (params.max_height - params.min_height) + 
                mesa_top * 20.0 + erosion
            },
            Biome::Plateau => {
                // Layered plateau system with varied elevations
                let scale1 = 0.001 * params.frequency_multiplier;
                let scale2 = 0.0008 * params.frequency_multiplier;
                let scale3 = 0.003 * params.frequency_multiplier;
                let scale4 = 0.0004 * params.frequency_multiplier;
                
                // Base plateau shape with multiple tiers
                let tier1 = ((x * scale1).sin() * (z * scale1).cos()).clamp(-1.0, 1.0);
                let tier2 = ((x * scale2 + 50.0).sin() * (z * scale2 - 30.0).cos()).clamp(-1.0, 1.0);
                let tier3 = ((x * scale4 - 100.0).cos() * (z * scale4 + 70.0).sin()).clamp(-1.0, 1.0);
                
                // Create distinct elevation levels
                let level1 = if tier1.abs() > 0.3 { 1.0 } else { tier1.abs() / 0.3 };
                let level2 = if tier2.abs() > 0.5 { 1.0 } else { tier2.abs() / 0.5 };
                let level3 = if tier3.abs() > 0.7 { 1.0 } else { tier3.abs() / 0.7 };
                
                // Smooth transitions between levels
                let smooth1 = level1 * level1 * (3.0 - 2.0 * level1);
                let smooth2 = level2 * level2 * (3.0 - 2.0 * level2);
                let smooth3 = level3 * level3 * (3.0 - 2.0 * level3);
                
                // Stack the plateaus
                let base_height = params.min_height;
                let tier_height = (params.max_height - params.min_height) / 3.0;
                
                let h1 = base_height + smooth1 * tier_height;
                let h2 = h1 + smooth2 * tier_height * 0.8;
                let h3 = h2 + smooth3 * tier_height * 0.6;
                
                // Surface weathering and details
                let weathering = (x * scale3).sin() * (z * scale3).cos() * 8.0;
                let cracks = ((x * 0.02).sin() * (z * 0.02).cos()).abs().powf(3.0) * -5.0;
                
                // Blend the tiers based on position
                let blend = ((x * 0.0002 + z * 0.0003).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                let height = h1 * (1.0 - blend) + h3 * blend + h2 * 0.3;
                
                height + weathering + cracks
            },
            Biome::Crystalline => {
                // Varied spiky crystal formations
                let scale1 = 0.01 * params.frequency_multiplier;
                let scale2 = 0.02 * params.frequency_multiplier;
                let scale3 = 0.05 * params.frequency_multiplier;
                let scale4 = 0.007 * params.frequency_multiplier;
                
                // Vary spike sharpness based on position
                let sharpness1 = 0.8 + ((x * 0.001).sin() * (z * 0.001).cos() * 0.4);
                let sharpness2 = 1.2 + ((x * 0.002 + 100.0).sin() * (z * 0.002).cos() * 0.6);
                
                // Different crystal cluster patterns
                let spike1 = ((x * scale1).sin() * (z * scale1).cos()).abs().powf(sharpness1) * params.base_amplitude;
                let spike2 = ((x * scale2 + 50.0).cos() * (z * scale2 - 30.0).sin()).abs().powf(sharpness2) * params.base_amplitude * 0.7;
                let spike3 = ((x * scale3 - 20.0).sin() * (z * scale3 + 40.0).cos()).abs() * params.base_amplitude * 0.4;
                
                // Add larger crystal formations
                let large_crystal = ((x * scale4).sin().powi(2) + (z * scale4).cos().powi(2)).sqrt();
                let crystal_height = (1.0 - large_crystal).max(0.0).powf(1.5) * params.base_amplitude * 0.8;
                
                // Base elevation variation
                let base_variation = (x * 0.003).sin() * (z * 0.003).cos() * 15.0;
                
                let height = spike1 + spike2 + spike3 + crystal_height + base_variation;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Volcanic => {
                // Rough terrain with crater-like formations
                let scale1 = 0.004 * params.frequency_multiplier;
                let scale2 = 0.008 * params.frequency_multiplier;
                
                let crater = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
                let rough = (x * scale2).sin() * (z * scale2).cos() * params.base_amplitude * params.roughness;
                
                let base_height = (1.0 - crater) * params.base_amplitude + rough;
                base_height.clamp(params.min_height, params.max_height)
            },
            Biome::Mountains => {
                // Realistic mountain ranges with varied slopes and heights
                let scale1 = 0.001 * params.frequency_multiplier;
                let scale2 = 0.003 * params.frequency_multiplier;
                let scale3 = 0.0005 * params.frequency_multiplier;
                let scale4 = 0.008 * params.frequency_multiplier;
                let scale5 = 0.0002 * params.frequency_multiplier;
                
                // Continental divide - major ridge
                let divide = ((x * scale5).sin() - (z * scale5 * 0.6).cos()).abs();
                let divide_height = divide.powf(0.5) * params.base_amplitude * 1.2;
                
                // Multiple mountain peaks at different elevations
                let peak1 = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
                let peak2 = ((x * scale2 + 100.0).sin().powi(2) + (z * scale2 - 50.0).cos().powi(2)).sqrt();
                let peak3 = ((x * scale3 - 200.0).sin().powi(2) + (z * scale3 + 150.0).cos().powi(2)).sqrt();
                
                // Vary peak heights and shapes
                let h1 = (1.0 - peak1).max(0.0).powf(1.5) * params.base_amplitude * 0.8;
                let h2 = (1.0 - peak2).max(0.0).powf(2.0) * params.base_amplitude * 0.6;
                let h3 = (1.0 - peak3).max(0.0).powf(1.2) * params.base_amplitude * 0.7;
                
                // Saddles and valleys between peaks
                let valley1 = ((x * scale2 * 0.7 + z * scale2 * 0.5).sin() * 0.5 + 0.5).powf(2.0) * -40.0;
                let valley2 = ((x * scale1 * 1.3 - z * scale1 * 0.8).cos() * 0.5 + 0.5).powf(2.0) * -30.0;
                
                // Foothills with gradual slope
                let distance_from_peak = ((x * 0.001).sin().powi(2) + (z * 0.001).cos().powi(2)).sqrt();
                let foothill_factor = (1.0 - distance_from_peak).max(0.0);
                let foothills = foothill_factor * (x * scale4).sin() * (z * scale4).cos() * 30.0;
                
                // Glacial carving
                let glacial = ((x * 0.005).sin() * (z * 0.005).cos()).abs().powf(0.3) * -20.0;
                
                let height = divide_height + h1 + h2 + h3 + valley1 + valley2 + foothills + glacial;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Desert => {
                // Sand dune formations
                let scale1 = 0.005 * params.frequency_multiplier;
                let scale2 = 0.01 * params.frequency_multiplier;
                let scale3 = 0.03 * params.frequency_multiplier;
                
                // Large dunes
                let dunes = ((x * scale1).sin() * (z * scale1 * 1.2).cos()).abs() * params.base_amplitude;
                // Ripples
                let ripples = (x * scale3).sin() * (z * scale3).cos() * params.base_amplitude * 0.1;
                // Medium variation
                let medium = ((x * scale2 + 30.0).cos() * (z * scale2 - 20.0).sin()) * params.base_amplitude * 0.3;
                
                let height = params.min_height + dunes + ripples + medium;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Arctic => {
                // Varied icy formations
                let scale1 = 0.008 * params.frequency_multiplier;
                let scale2 = 0.02 * params.frequency_multiplier;
                let scale3 = 0.04 * params.frequency_multiplier;
                let scale4 = 0.003 * params.frequency_multiplier;
                
                // Glacial base with undulations
                let glacial = ((x * scale1).sin() + (z * scale1 * 0.9).cos()) * params.base_amplitude * 0.5;
                
                // Ice spikes with varying heights and sharpness
                let spike_var = 1.5 + ((x * 0.002).sin() * (z * 0.002).cos() * 1.0);
                let spikes = ((x * scale2).sin() * (z * scale2).cos()).abs().powf(spike_var) * params.base_amplitude * 0.8;
                
                // Smaller ice formations
                let small_spikes = ((x * scale3).sin() * (z * scale3).cos()).abs() * params.base_amplitude * 0.3;
                
                // Ice sheets and smooth areas
                let ice_sheets = ((x * scale4).sin().powi(2) + (z * scale4).cos().powi(2)).powf(0.3) * params.base_amplitude * 0.4;
                
                // Crevasses with varying depths
                let crevasse_depth = ((x * 0.001 + z * 0.0007).sin() * 0.5 + 0.5) * 0.4;
                let crevasses = ((x * scale3 + z * scale3 * 0.7).sin()).abs() * params.base_amplitude * crevasse_depth;
                
                let height = params.min_height + glacial + spikes + small_spikes + ice_sheets - crevasses;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Badlands => {
                // Complex eroded terrain with varied formations
                let scale1 = 0.006 * params.frequency_multiplier;
                let scale2 = 0.015 * params.frequency_multiplier;
                let scale3 = 0.04 * params.frequency_multiplier;
                let scale4 = 0.002 * params.frequency_multiplier;
                
                // Mesa tops with varying heights
                let mesa_height_var = 0.2 + ((x * 0.001).sin() * (z * 0.0008).cos() * 0.3).abs();
                let mesas = ((x * scale1).sin() * (z * scale1).cos()).abs().powf(mesa_height_var) * params.base_amplitude;
                
                // Complex erosion patterns
                let erosion1 = ((x * scale2 + z * scale2 * 0.5).sin() + (x * scale2 * 0.7 - z * scale2).cos()) * params.base_amplitude * 0.4;
                let erosion2 = ((x * scale2 * 1.3).sin() - (z * scale2 * 0.8).cos()) * params.base_amplitude * 0.2;
                
                // Hoodoos and pillars with varying heights
                let pillar_power = 2.5 + ((x * 0.003 + z * 0.002).sin() * 1.0);
                let pillars = ((x * scale3).sin() * (z * scale3).cos()).abs().powf(pillar_power) * params.base_amplitude * 0.5;
                
                // Layered sediment effect
                let layers = ((z * scale4).sin() * 0.5 + 0.5) * 8.0;
                
                let height = mesas + erosion1.abs() + erosion2.abs() + pillars + layers;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Floating => {
                // Suspended islands
                let scale1 = 0.002 * params.frequency_multiplier;
                let scale2 = 0.008 * params.frequency_multiplier;
                
                // Island shapes
                let islands = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
                let shape = (1.0 - islands).max(0.0).powf(2.0) * params.base_amplitude;
                // Small variations
                let detail = (x * scale2).sin() * (z * scale2).cos() * params.base_amplitude * 0.1;
                
                let height = params.min_height + shape + detail;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Caverns => {
                // Pockmarked terrain with holes
                let scale1 = 0.01 * params.frequency_multiplier;
                let scale2 = 0.025 * params.frequency_multiplier;
                let scale3 = 0.05 * params.frequency_multiplier;
                
                // Base rocky terrain
                let base = (x * scale1).sin() * (z * scale1).cos() * params.base_amplitude * 0.5;
                // Cave holes (inverted peaks)
                let holes1 = ((x * scale2).sin() * (z * scale2).cos()).abs().powf(4.0) * params.base_amplitude;
                let holes2 = ((x * scale3 + 50.0).cos() * (z * scale3 - 30.0).sin()).abs().powf(4.0) * params.base_amplitude * 0.7;
                
                let height = base - holes1 - holes2;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Swamp => {
                // Low, undulating wetlands
                let scale1 = 0.02 * params.frequency_multiplier;
                let scale2 = 0.04 * params.frequency_multiplier;
                let scale3 = 0.08 * params.frequency_multiplier;
                
                // Gentle undulations
                let undulation = (x * scale1).sin() * (z * scale1).cos() * params.base_amplitude * 0.3;
                // Small pools
                let pools = ((x * scale2).sin() + (z * scale2).cos()) * params.base_amplitude * 0.2;
                // Muddy bumps
                let bumps = (x * scale3).sin() * (z * scale3).sin() * params.base_amplitude * 0.1;
                
                let height = undulation + pools + bumps;
                height.clamp(params.min_height, params.max_height)
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
        Self::get_blended_biome_height_shader(x, z)
    }
    
    pub fn height_at(x: f32, z: f32) -> f32 {
        // Static version for use without instance
        Self::get_blended_biome_height_shader(x, z)
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
    
    fn get_plains_height_shader(x: f32, z: f32) -> f32 {
        // Very low frequency for gentle rolling plains
        let scale1 = 0.0002;
        let scale2 = 0.0004;
        
        // Gentle rolling hills
        let h1 = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 25.0;
        let h2 = (x * scale2 + 1.0).sin() * (z * scale2 * 1.2).cos() * 15.0;
        
        // Subtle undulations
        let detail = (x * 0.001).sin() * (z * 0.0008).sin() * 8.0;
        
        h1 + h2 + detail
    }
    
    fn get_canyon_height_shader(x: f32, z: f32) -> f32 {
        // Lower frequency for wider features
        let scale1 = 0.0008;
        let scale2 = 0.0004;
        
        // Smooth rolling canyon
        let base = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 60.0;
        let secondary = (x * scale2 + 1.0).sin() * (z * scale2 * 1.2).cos() * 40.0;
        
        // Add some gentle valleys
        let mut valley = 0.0;
        valley += smoothstep(0.0, 1.0, (x * 0.001).sin()) * 30.0;
        valley += smoothstep(0.0, 1.0, (z * 0.0008).cos()) * 20.0;
        
        // Gentle undulations instead of cliffs
        let detail = (x * 0.005).sin() * (z * 0.004).cos() * 10.0;
        
        base + secondary - valley + detail
    }
    
    fn get_plateau_height_shader(x: f32, z: f32) -> f32 {
        // Very low frequency for large, smooth plateaus
        let scale1 = 0.0002;
        let scale2 = 0.0003;
        
        // Smooth raised areas with tapered edges
        let raise1 = smoothstep(0.2, 0.8, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5);
        let raise2 = smoothstep(0.3, 0.7, (x * scale2 + 1.5).sin() * (z * scale2 - 0.8).cos() * 0.5 + 0.5);
        
        // Gradual height changes
        let h1 = raise1 * 60.0;
        let h2 = raise2 * 40.0;
        
        // Rolling surface
        let surface = (x * 0.001).sin() * (z * 0.0008).cos() * 15.0;
        
        // Gentle blend between heights
        let blend = (x * 0.0001 + z * 0.00015).sin() * 0.5 + 0.5;
        
        h1 * (1.0 - blend) + h2 * blend + surface
    }
    
    fn get_crystalline_height_shader(x: f32, z: f32) -> f32 {
        // Lower frequency for larger crystal formations
        let scale1 = 0.002;
        let scale2 = 0.004;
        
        // Smooth crystal clusters instead of sharp spikes
        let cluster1 = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5);
        let cluster2 = smoothstep(0.4, 0.6, (x * scale2 + 1.0).cos() * (z * scale2 - 0.5).sin() * 0.5 + 0.5);
        
        // Varied heights with smooth transitions
        let h1 = cluster1 * 50.0;
        let h2 = cluster2 * 35.0;
        
        // Gentle crystalline texture
        let texture = ((x * 0.01).sin() * (z * 0.008).cos()).abs() * 20.0;
        
        // Smooth base elevation
        let base = (x * 0.0005).sin() * (z * 0.0004).cos() * 25.0;
        
        h1 + h2 + texture + base
    }
    
    fn get_volcanic_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for broad volcanic features
        let scale1 = 0.0002;
        let scale2 = 0.0005;
        
        // Smooth volcanic cone with gentle slopes
        let dist = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
        let cone = smoothstep(1.0, 0.0, dist) * 80.0;
        
        // Rolling lava fields
        let fields = (x * scale2).sin() * (z * scale2 * 0.9).cos() * 25.0;
        
        // Gentle surface texture
        let texture = (x * 0.002).sin() * (z * 0.0018).cos() * 10.0;
        
        cone + fields + texture
    }
    
    fn get_mountains_height_shader(x: f32, z: f32) -> f32 {
        // Much lower frequency for broader mountains
        let scale1 = 0.0002;
        let scale2 = 0.0004;
        
        // Smooth gaussian-like peaks instead of sharp ones
        let dist1 = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
        let dist2 = ((x * scale2 + 1.0).sin().powi(2) + (z * scale2 - 0.5).cos().powi(2)).sqrt();
        
        // Use gaussian falloff for smooth peaks
        let h1 = (-dist1 * dist1 * 2.0).exp() * 120.0;
        let h2 = (-dist2 * dist2 * 3.0).exp() * 80.0;
        
        // Rolling foothills
        let mut foothills = 0.0;
        foothills += (x * 0.0008).sin() * (z * 0.0007).cos() * 30.0;
        foothills += (x * 0.0012 + 0.5).sin() * (z * 0.001).sin() * 20.0;
        
        // Gentle valleys between peaks
        let valley = (x * 0.0003 + z * 0.0002).sin() * 15.0;
        
        h1 + h2 + foothills + valley
    }
    
    fn get_desert_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for large dune fields
        let scale1 = 0.0003;
        let scale2 = 0.0006;
        
        // Smooth, rolling dunes
        let dunes = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1 * 1.2).cos() * 0.5 + 0.5) * 30.0;
        let secondary = (x * scale2 + 0.5).sin() * (z * scale2 * 0.8).cos() * 20.0;
        
        // Gentle ripples
        let ripples = (x * 0.003).sin() * (z * 0.0025).cos() * 5.0;
        
        dunes + secondary + ripples
    }
    
    fn get_arctic_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for smooth, rolling ice sheets
        let scale1 = 0.0003;
        let scale2 = 0.0006;
        
        // Smooth rolling glacial terrain
        let glacial = (x * scale1).sin() * (z * scale1 * 0.9).cos() * 40.0;
        let sheets = (x * scale2 + 0.5).cos() * (z * scale2 * 1.1).sin() * 30.0;
        
        // Gentle ice dunes
        let dunes = smoothstep(0.2, 0.8, (x * 0.001).sin() * (z * 0.0008).cos() * 0.5 + 0.5) * 25.0;
        
        // Subtle surface texture
        let texture = (x * 0.008).sin() * (z * 0.007).cos() * 10.0;
        
        // Very gentle crevasses
        let crevasse = smoothstep(0.4, 0.6, (x * 0.002 + z * 0.0015).sin()) * -15.0;
        
        glacial + sheets + dunes + texture + crevasse
    }
    
    fn get_badlands_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for wider, smoother features
        let scale1 = 0.0004;
        let scale2 = 0.0008;
        
        // Smooth eroded hills
        let hills = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 50.0;
        let erosion = smoothstep(0.3, 0.7, (x * scale2 + 0.7).cos() * (z * scale2 * 1.2).sin() * 0.5 + 0.5) * 30.0;
        
        // Gentle mesas with sloped sides
        let mesa = smoothstep(0.2, 0.6, (x * 0.0003).sin() * (z * 0.00025).cos() * 0.5 + 0.5) * 40.0;
        
        // Rolling badland texture
        let texture = (x * 0.002).sin() * (z * 0.0018).cos() * 15.0;
        
        hills + erosion + mesa + texture
    }
    
    fn get_floating_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for large floating islands
        let scale1 = 0.0002;
        let scale2 = 0.0004;
        
        // Smooth floating plateaus
        let island1 = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5) * 60.0;
        let island2 = smoothstep(0.4, 0.6, (x * scale2 + 0.8).cos() * (z * scale2 * 0.9).sin() * 0.5 + 0.5) * 40.0;
        
        // Gentle surface
        let surface = (x * 0.001).sin() * (z * 0.0008).cos() * 10.0;
        
        80.0 + island1 + island2 + surface
    }
    
    fn get_caverns_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for larger cavern systems
        let scale1 = 0.0004;
        let scale2 = 0.0008;
        
        // Rolling base terrain
        let base = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 40.0;
        
        // Smooth depressions instead of sharp holes
        let depression1 = smoothstep(0.6, 0.3, (x * scale2).sin() * (z * scale2).cos() * 0.5 + 0.5) * -30.0;
        let depression2 = smoothstep(0.5, 0.2, (x * scale2 * 1.3 + 1.0).cos() * (z * scale2 * 0.9).sin() * 0.5 + 0.5) * -20.0;
        
        // Gentle undulations
        let detail = (x * 0.002).sin() * (z * 0.0015).cos() * 10.0;
        
        base + depression1 + depression2 + detail
    }
    
    fn get_swamp_height_shader(x: f32, z: f32) -> f32 {
        // Low frequency for gentle swamp terrain
        let scale1 = 0.0005;
        let scale2 = 0.001;
        
        // Very gentle undulations
        let undulation = (x * scale1).sin() * (z * scale1 * 0.9).cos() * 15.0;
        let pools = smoothstep(0.4, 0.6, (x * scale2).sin() * (z * scale2 * 1.1).cos() * 0.5 + 0.5) * -10.0;
        
        // Subtle surface variation
        let surface = (x * 0.003).sin() * (z * 0.0025).cos() * 5.0;
        
        undulation + pools + surface
    }
    
    // Enhanced biome selection with more variety and smaller regions (matches shader exactly)
    fn get_biome_height_shader(x: f32, z: f32) -> f32 {
        // Multi-scale noise for more organic biome distribution
        let noise1 = (x * 0.0003).sin() * (z * 0.0003).cos();
        let noise2 = (x * 0.0007 + 1.3).sin() * (z * 0.0006 - 0.7).sin();
        let noise3 = (x * 0.0013 - 2.1).cos() * (z * 0.0011 + 1.9).sin();
        
        // Combine noises for complex patterns
        let mut biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
        
        // Add local variation for sub-biomes
        let local_var = (x * 0.01).sin() * (z * 0.01).cos() * 0.1;
        biome_noise += local_var;
        
        // 12 biome types distributed across the noise range
        if biome_noise < -0.7 {
            Self::get_canyon_height_shader(x, z)
        } else if biome_noise < -0.5 {
            Self::get_caverns_height_shader(x, z)
        } else if biome_noise < -0.3 {
            Self::get_badlands_height_shader(x, z)
        } else if biome_noise < -0.1 {
            Self::get_plateau_height_shader(x, z)
        } else if biome_noise < 0.1 {
            Self::get_plains_height_shader(x, z)
        } else if biome_noise < 0.25 {
            Self::get_desert_height_shader(x, z)
        } else if biome_noise < 0.4 {
            Self::get_swamp_height_shader(x, z)
        } else if biome_noise < 0.5 {
            Self::get_crystalline_height_shader(x, z)
        } else if biome_noise < 0.6 {
            Self::get_volcanic_height_shader(x, z)
        } else if biome_noise < 0.7 {
            Self::get_arctic_height_shader(x, z)
        } else if biome_noise < 0.8 {
            Self::get_floating_height_shader(x, z)
        } else {
            Self::get_mountains_height_shader(x, z)
        }
    }
    
    // Smooth blending between biomes (matches shader exactly)
    fn get_blended_biome_height_shader(x: f32, z: f32) -> f32 {
        // Sample multiple nearby points for smoother transitions
        let sample_dist = 50.0;
        let h_center = Self::get_biome_height_shader(x, z);
        let h_north = Self::get_biome_height_shader(x, z + sample_dist);
        let h_south = Self::get_biome_height_shader(x, z - sample_dist);
        let h_east = Self::get_biome_height_shader(x + sample_dist, z);
        let h_west = Self::get_biome_height_shader(x - sample_dist, z);
        
        // Average nearby samples for smoother terrain
        let primary_height = (h_center * 2.0 + h_north + h_south + h_east + h_west) / 6.0;
        
        // Add rolling hills with lower frequency
        let mut hills = 0.0;
        hills += (x * 0.0001).sin() * (z * 0.00012).cos() * 60.0;
        hills += (x * 0.00018 + 1.5).sin() * (z * 0.00015 - 0.7).cos() * 40.0;
        hills += (x * 0.00025 - 0.3).sin() * (z * 0.0003 + 1.2).cos() * 25.0;
        
        // Add gentle undulations
        let mut undulation = 0.0;
        undulation += (x * 0.0004).sin() * (z * 0.0004).cos() * 15.0;
        undulation += (x * 0.0008 + 2.1).sin() * (z * 0.0007 - 1.3).sin() * 10.0;
        
        // Very gentle large scale features
        let continent_scale = 0.00005;
        let continental = (x * continent_scale).sin() * (z * continent_scale).cos() * 30.0;
        
        // Smooth everything together
        let height = primary_height * 0.7 + hills * 0.2 + undulation * 0.1;
        
        height + continental
    }
}