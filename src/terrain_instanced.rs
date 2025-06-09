use miniquad::*;
use crate::vertex::Vertex;
use crate::biome::{BiomeMap, Biome};

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
            BufferUsage::Immutable,
            BufferSource::slice(&instance_data)
        );
        
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * 160.0;
        
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
            BufferUsage::Immutable,
            BufferSource::slice(&instance_data)
        );
        
        // Create biome map
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * 160.0;
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Create height and biome textures
        let texture_size = 512; // Reduced for faster generation
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
            BufferUsage::Immutable,
            BufferSource::slice(&instance_data)
        );
        
        // Create biome map
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * 160.0;
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Create height and biome textures
        let texture_size = 512; // Reduced for faster generation, still good quality
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
        }
    }
    
    fn create_base_mesh() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let grid_size = 8;
        let cell_size = 20.0;
        let half_size = 80.0;
        
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
        let chunk_size = 160.0;
        
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
                
                // Store height in height texture
                let normalized_height = ((height + 500.0) / 1000.0 * 255.0).clamp(0.0, 255.0) as u8;
                let height_idx = ((y * size + x) * 4) as usize;
                height_data[height_idx] = normalized_height;     // R
                height_data[height_idx + 1] = normalized_height; // G
                height_data[height_idx + 2] = normalized_height; // B
                height_data[height_idx + 3] = 255;              // A
                
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
        
        for (biome, weight) in biome_weights {
            let params = biome.height_params();
            let height = Self::biome_height(x, z, biome, &params);
            total_height += height * weight;
        }
        
        total_height
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
                // Deep cuts with steep walls
                let scale = 0.003 * params.frequency_multiplier;
                let canyon_cut = ((x * scale).sin() + (z * scale * 0.7).cos()) * 0.5;
                let depth = canyon_cut.abs().powf(3.0); // Sharp canyon edges
                
                params.min_height + (1.0 - depth) * (params.max_height - params.min_height)
            },
            Biome::Plateau => {
                // Flat elevated areas with cliff edges
                let scale = 0.001 * params.frequency_multiplier;
                let plateau_shape = ((x * scale).sin() * (z * scale).cos()).clamp(-1.0, 1.0);
                let flatness = plateau_shape.abs().powf(0.2); // Very flat top
                
                params.min_height + flatness * (params.max_height - params.min_height)
            },
            Biome::Crystalline => {
                // Spiky crystal formations
                let scale1 = 0.01 * params.frequency_multiplier;
                let scale2 = 0.02 * params.frequency_multiplier;
                let scale3 = 0.05 * params.frequency_multiplier;
                
                let spike1 = ((x * scale1).sin() * (z * scale1).cos()).abs() * params.base_amplitude;
                let spike2 = ((x * scale2 + 50.0).cos() * (z * scale2 - 30.0).sin()).abs() * params.base_amplitude * 0.7;
                let spike3 = ((x * scale3 - 20.0).sin() * (z * scale3 + 40.0).cos()).abs() * params.base_amplitude * 0.4;
                
                let height = spike1 + spike2 + spike3;
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
                // Tall peaks and valleys
                let scale1 = 0.001 * params.frequency_multiplier;
                let scale2 = 0.003 * params.frequency_multiplier;
                
                let ridge = ((x * scale1).sin() - (z * scale1 * 0.8).cos()).abs() * params.base_amplitude;
                let peaks = ((x * scale2 + 100.0).sin() * (z * scale2 - 50.0).cos()).abs() * params.base_amplitude * 0.6;
                
                let height = ridge + peaks;
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
                // Icy spikes and glacial formations
                let scale1 = 0.008 * params.frequency_multiplier;
                let scale2 = 0.02 * params.frequency_multiplier;
                let scale3 = 0.04 * params.frequency_multiplier;
                
                // Glacial base
                let glacial = ((x * scale1).sin() + (z * scale1 * 0.9).cos()) * params.base_amplitude * 0.5;
                // Ice spikes
                let spikes = ((x * scale2).sin() * (z * scale2).cos()).abs().powf(2.0) * params.base_amplitude;
                // Crevasses
                let crevasses = ((x * scale3 + z * scale3 * 0.7).sin()).abs() * params.base_amplitude * 0.3;
                
                let height = params.min_height + glacial + spikes - crevasses;
                height.clamp(params.min_height, params.max_height)
            },
            Biome::Badlands => {
                // Eroded pillars and mesas
                let scale1 = 0.006 * params.frequency_multiplier;
                let scale2 = 0.015 * params.frequency_multiplier;
                let scale3 = 0.04 * params.frequency_multiplier;
                
                // Mesa tops
                let mesas = ((x * scale1).sin() * (z * scale1).cos()).abs().powf(0.3) * params.base_amplitude;
                // Erosion channels
                let erosion = ((x * scale2 + z * scale2 * 0.5).sin() + (x * scale2 * 0.7 - z * scale2).cos()) * params.base_amplitude * 0.4;
                // Pillars
                let pillars = ((x * scale3).sin() * (z * scale3).cos()).abs().powf(3.0) * params.base_amplitude * 0.5;
                
                let height = mesas + erosion.abs() + pillars;
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
        let biome_weights = self.biome_map.get_biome_weights(x, z);
        Self::height_at_with_biomes(x, z, &biome_weights)
    }
    
    pub fn get_biome_at(&self, x: f32, z: f32) -> Biome {
        self.biome_map.get_biome_at(x, z)
    }
}