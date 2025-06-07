use miniquad::*;
use crate::vertex::Vertex;

pub struct InstancedTerrain {
    base_mesh_vertex_buffer: BufferId,
    base_mesh_index_buffer: BufferId,
    instance_buffer: BufferId,
    height_texture: TextureId,
    index_count: i32,
    instance_count: i32,
    terrain_scale: f32,
}

impl InstancedTerrain {
    pub fn new(ctx: &mut dyn RenderingBackend, view_distance: i32) -> Self {
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
        
        // Create height texture
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * 160.0;
        let texture_size = 2048; // Use power of 2 for better GPU performance
        let height_texture = Self::create_height_texture(ctx, texture_size, terrain_scale);
        
        println!("Created instanced terrain:");
        println!("  Base mesh: {} vertices, {} indices", vertices.len(), indices.len());
        println!("  Instances: {}", instance_count);
        println!("  Terrain scale: {}", terrain_scale);
        println!("  Texture size: {}x{}", texture_size, texture_size);
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
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
    
    fn create_height_texture(ctx: &mut dyn RenderingBackend, size: u32, terrain_scale: f32) -> TextureId {
        let mut height_data = vec![0u8; (size * size * 4) as usize]; // RGBA format
        
        // Generate height values
        for y in 0..size {
            for x in 0..size {
                // Convert texture coordinates to world coordinates
                let u = x as f32 / (size - 1) as f32;
                let v = y as f32 / (size - 1) as f32;
                
                let world_x = (u - 0.5) * terrain_scale;
                let world_z = (v - 0.5) * terrain_scale;
                
                let height = Self::height_at(world_x, world_z);
                
                // Normalize height to 0-255 range
                // Assuming height ranges from -100 to 100
                let normalized = ((height + 100.0) / 200.0 * 255.0).clamp(0.0, 255.0) as u8;
                
                let idx = ((y * size + x) * 4) as usize;
                height_data[idx] = normalized;     // R
                height_data[idx + 1] = normalized; // G
                height_data[idx + 2] = normalized; // B
                height_data[idx + 3] = 255;       // A
            }
        }
        
        let texture = ctx.new_texture(
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
        
        texture
    }
    
    fn height_at(x: f32, z: f32) -> f32 {
        let scale1 = 0.002;
        let scale2 = 0.007;
        let scale3 = 0.015;
        let height_scale = 60.0;
        
        let h1 = (x * scale1).sin() * (z * scale1).cos() * height_scale;
        let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * height_scale * 0.5;
        let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * height_scale * 0.25;
        
        h1 + h2 + h3
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
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
    
    pub fn instance_count(&self) -> i32 {
        self.instance_count
    }
    
    pub fn terrain_scale(&self) -> f32 {
        self.terrain_scale
    }
}