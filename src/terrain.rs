use miniquad::*;
use crate::vertex::Vertex;

pub struct TerrainChunk {
    pub x_offset: f32,
    pub z_offset: f32,
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
}

impl TerrainChunk {
    pub fn new(x_offset: f32, z_offset: f32, ctx: &mut dyn RenderingBackend) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate static mesh data
        let terrain_y_base = -30.0;
        let grid_size = 8;
        let cell_size = 20.0;
        let half_size = 80.0;
        
        // First, generate the grid positions
        let mut grid_positions = Vec::new();
        for i in 0..=grid_size {
            for j in 0..=grid_size {
                let x = x_offset - half_size + (i as f32) * cell_size;
                let z = z_offset - half_size + (j as f32) * cell_size;
                let y = Self::height_at_static(x, z) + terrain_y_base;
                grid_positions.push((x, y, z));
            }
        }
        
        // Generate triangles with proper barycentric coordinates
        for i in 0..grid_size {
            for j in 0..grid_size {
                let idx = i * (grid_size + 1) + j;
                let base_vertex_idx = vertices.len() as u32;
                
                // Get the four corners of this cell
                let p0 = grid_positions[idx];
                let p1 = grid_positions[idx + 1];
                let p2 = grid_positions[idx + grid_size + 1];
                let p3 = grid_positions[idx + grid_size + 2];
                
                // First triangle (p0, p2, p1)
                vertices.push(Vertex::with_barycentric(p0.0, p0.1, p0.2, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(p2.0, p2.1, p2.2, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(p1.0, p1.1, p1.2, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx);
                indices.push(base_vertex_idx + 1);
                indices.push(base_vertex_idx + 2);
                
                // Second triangle (p1, p2, p3)
                vertices.push(Vertex::with_barycentric(p1.0, p1.1, p1.2, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(p2.0, p2.1, p2.2, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(p3.0, p3.1, p3.2, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx + 3);
                indices.push(base_vertex_idx + 4);
                indices.push(base_vertex_idx + 5);
            }
        }
        
        let index_count = indices.len() as i32;
        
        // Create GPU buffers
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        Self {
            x_offset,
            z_offset,
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
    
    fn height_at_static(x: f32, z: f32) -> f32 {
        let scale1 = 0.002;
        let scale2 = 0.007;
        let scale3 = 0.015;
        let height_scale = 60.0;
        
        let h1 = (x * scale1).sin() * (z * scale1).cos() * height_scale;
        let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * height_scale * 0.5;
        let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * height_scale * 0.25;
        
        h1 + h2 + h3
    }
    
    pub fn vertex_buffer(&self) -> BufferId {
        self.vertex_buffer
    }
    
    pub fn index_buffer(&self) -> BufferId {
        self.index_buffer
    }
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
}

