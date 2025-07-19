use miniquad::*;
use crate::vertex::Vertex;
use super::chunk::LodLevel;

pub struct TerrainBatch {
    pub lod: LodLevel,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    vertex_buffer: Option<BufferId>,
    index_buffer: Option<BufferId>,
}

impl TerrainBatch {
    pub fn new(lod: LodLevel) -> Self {
        // Pre-allocate reasonable capacity based on expected chunks
        let expected_chunks = match lod {
            LodLevel::High => 200,    // ~11x11 area
            LodLevel::Medium => 500,  // Ring around high LOD
            LodLevel::Low => 3500,    // Most chunks
        };
        
        let vertices_per_chunk = match lod {
            LodLevel::High => 81,     // 9x9 vertices
            LodLevel::Medium => 25,   // 5x5 vertices
            LodLevel::Low => 9,       // 3x3 vertices
        };
        
        let indices_per_chunk = match lod {
            LodLevel::High => 128 * 3,   // 64 cells * 2 triangles * 3 indices
            LodLevel::Medium => 32 * 3,  // 16 cells * 2 triangles * 3 indices
            LodLevel::Low => 8 * 3,      // 4 cells * 2 triangles * 3 indices
        };
        
        Self {
            lod,
            vertices: Vec::with_capacity(expected_chunks * vertices_per_chunk),
            indices: Vec::with_capacity(expected_chunks * indices_per_chunk),
            vertex_buffer: None,
            index_buffer: None,
        }
    }
    
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }
    
    pub fn add_chunk(&mut self, chunk_vertices: &[Vertex], chunk_indices: &[u32]) {
        let vertex_offset = self.vertices.len() as u32;
        
        // Add vertices directly (they're already in world space)
        self.vertices.extend_from_slice(chunk_vertices);
        
        // Add indices with offset
        for &idx in chunk_indices {
            self.indices.push(idx + vertex_offset);
        }
    }
    
    pub fn upload_to_gpu(&mut self, ctx: &mut dyn RenderingBackend) {
        if self.vertices.is_empty() {
            return;
        }
        
        // Delete old buffers if they exist - simpler and more reliable
        if let Some(vb) = self.vertex_buffer.take() {
            ctx.delete_buffer(vb);
        }
        if let Some(ib) = self.index_buffer.take() {
            ctx.delete_buffer(ib);
        }
        
        // Create new buffers
        self.vertex_buffer = Some(ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&self.vertices)
        ));
        
        self.index_buffer = Some(ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&self.indices)
        ));
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, uniforms: &crate::shader::Uniforms) -> i32 {
        if let (Some(vb), Some(ib)) = (self.vertex_buffer, self.index_buffer) {
            let bindings = Bindings {
                vertex_buffers: vec![vb],
                index_buffer: ib,
                images: vec![],
            };
            
            ctx.apply_pipeline(pipeline);
            ctx.apply_bindings(&bindings);
            ctx.apply_uniforms(UniformsSource::table(uniforms));
            
            let triangle_count = self.indices.len() as i32 / 3;
            ctx.draw(0, self.indices.len() as i32, 1);
            
            triangle_count
        } else {
            0
        }
    }
}