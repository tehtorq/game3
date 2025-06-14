use miniquad::*;
use glam::{Vec3, Mat4};
use crate::vertex::Vertex;
use crate::shader::UniformsTerrainGPU;

// Simple grid-based GPU terrain
pub struct TerrainGPUGrid {
    grid_size: usize,
    grid_spacing: f32,
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
}

impl TerrainGPUGrid {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        // Create a very large grid that covers the view distance with high detail
        let grid_size = 1024; // 1024x1024 grid
        let grid_spacing = 12.0; // 12 units between vertices for high detail
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate grid vertices centered at origin
        let half_size = grid_size as f32 * 0.5;
        for z in 0..=grid_size {
            for x in 0..=grid_size {
                let fx = (x as f32 - half_size) * grid_spacing;
                let fz = (z as f32 - half_size) * grid_spacing;
                
                // Store position, height will be calculated on GPU
                vertices.push(Vertex::new(fx, 0.0, fz));
            }
        }
        
        // Generate indices with LOD in mind
        let vertex_row_size = grid_size + 1;
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let idx = (z * vertex_row_size + x) as u32;
                
                // First triangle
                indices.push(idx);
                indices.push(idx + 1);
                indices.push(idx + vertex_row_size as u32);
                
                // Second triangle
                indices.push(idx + 1);
                indices.push(idx + vertex_row_size as u32 + 1);
                indices.push(idx + vertex_row_size as u32);
            }
        }
        
        println!("GPU Grid terrain: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
        
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
            grid_size,
            grid_spacing,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as i32,
        }
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: Mat4, color: [f32; 3], player_pos: Vec3) -> i32 {
        ctx.apply_pipeline(pipeline);
        
        let bindings = Bindings {
            vertex_buffers: vec![self.vertex_buffer],
            index_buffer: self.index_buffer,
            images: vec![],
        };
        
        ctx.apply_bindings(&bindings);
        
        // Use player position as offset for the grid
        // This makes the grid follow the player
        let chunk_offset = [player_pos.x, player_pos.z];
        
        let uniforms = UniformsTerrainGPU::new(
            mvp,
            color,
            0.0, // No morphing needed for single grid
            chunk_offset,
            self.grid_spacing,
            player_pos
        );
        
        ctx.apply_uniforms(UniformsSource::table(&uniforms));
        ctx.draw(0, self.index_count, 1);
        
        self.index_count / 3
    }
}