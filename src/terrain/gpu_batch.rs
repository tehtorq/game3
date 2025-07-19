use miniquad::*;
use crate::vertex::Vertex;
use crate::terrain_chunk::{TerrainChunk, LodLevel};
use crate::shader::UniformsTerrainGPU;

pub struct TerrainGPUBatch {
    pub lod: LodLevel,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub chunk_data: Vec<(i32, i32, f32)>, // chunk_x, chunk_z, morph_factor
    vertex_buffer: Option<BufferId>,
    index_buffer: Option<BufferId>,
}

impl TerrainGPUBatch {
    pub fn new(lod: LodLevel) -> Self {
        // Adjusted for aggressive sparse loading
        let expected_chunks = match lod {
            LodLevel::High => 400,     // Only near chunks are high detail
            LodLevel::Medium => 800,   // Medium ring with 50% sparse
            LodLevel::Low => 3000,    // Far chunks with 75% sparse
        };
        
        let vertices_per_chunk = match lod {
            LodLevel::High => 81,     // 9x9 vertices
            LodLevel::Medium => 25,   // 5x5 vertices
            LodLevel::Low => 9,       // 3x3 vertices
        };
        
        let indices_per_chunk = match lod {
            LodLevel::High => 128 * 3,
            LodLevel::Medium => 32 * 3,
            LodLevel::Low => 8 * 3,
        };
        
        Self {
            lod,
            vertices: Vec::with_capacity(expected_chunks * vertices_per_chunk),
            indices: Vec::with_capacity(expected_chunks * indices_per_chunk),
            chunk_data: Vec::with_capacity(expected_chunks),
            vertex_buffer: None,
            index_buffer: None,
        }
    }
    
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.chunk_data.clear();
    }
    
    pub fn add_chunk(&mut self, chunk: &TerrainChunk, morph_factor: f32) {
        let vertex_offset = self.vertices.len() as u32;
        
        // Store chunk position and morph factor
        self.chunk_data.push((chunk.chunk_x, chunk.chunk_z, morph_factor));
        
        // Add vertices
        self.vertices.extend_from_slice(&chunk.vertices);
        
        // Add indices with offset
        for &idx in &chunk.indices {
            self.indices.push(idx + vertex_offset);
        }
    }
    
    pub fn upload_to_gpu(&mut self, ctx: &mut dyn RenderingBackend) {
        if self.vertices.is_empty() {
            return;
        }
        
        // Delete old buffers if they exist
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
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: glam::Mat4, color: [f32; 3], chunk_size: f32, player_pos: glam::Vec3) -> i32 {
        if let (Some(vb), Some(ib)) = (self.vertex_buffer, self.index_buffer) {
            let bindings = Bindings {
                vertex_buffers: vec![vb],
                index_buffer: ib,
                images: vec![],
            };
            
            ctx.apply_pipeline(pipeline);
            ctx.apply_bindings(&bindings);
            
            // Calculate LOD scale
            let lod_scale = chunk_size / self.lod.grid_size() as f32;
            
            let mut triangles = 0;
            let mut index_offset = 0;
            let vertices_per_chunk = match self.lod {
                LodLevel::High => 81,
                LodLevel::Medium => 25,
                LodLevel::Low => 9,
            };
            let indices_per_chunk = match self.lod {
                LodLevel::High => 128 * 3,
                LodLevel::Medium => 32 * 3,
                LodLevel::Low => 8 * 3,
            };
            
            // Draw each chunk with its own uniforms
            for &(chunk_x, chunk_z, base_morph) in &self.chunk_data {
                let chunk_offset = [chunk_x as f32 * chunk_size, chunk_z as f32 * chunk_size];
                
                // Calculate distance-based morph factor for geomorphing
                let chunk_center_x = chunk_offset[0] + chunk_size * 0.5;
                let chunk_center_z = chunk_offset[1] + chunk_size * 0.5;
                let dist_to_player = ((chunk_center_x - player_pos.x).powi(2) + 
                                     (chunk_center_z - player_pos.z).powi(2)).sqrt();
                
                // Smooth morph factor based on distance - updated for new LOD distances
                let morph_factor = match self.lod {
                    LodLevel::High => {
                        // Morph to medium LOD between 350-450 units
                        ((dist_to_player - 350.0) / 100.0).clamp(0.0, 1.0)
                    },
                    LodLevel::Medium => {
                        // Morph to low LOD between 1100-1300 units
                        ((dist_to_player - 1100.0) / 200.0).clamp(0.0, 1.0)
                    },
                    LodLevel::Low => 0.0, // No morphing for lowest LOD
                };
                
                let time = (miniquad::date::now() as f32) * 0.001;
                let uniforms = UniformsTerrainGPU::new(
                    mvp,
                    color,
                    morph_factor * base_morph,
                    chunk_offset,
                    lod_scale,
                    player_pos,
                    time
                );
                
                ctx.apply_uniforms(UniformsSource::table(&uniforms));
                ctx.draw(index_offset, indices_per_chunk, 1);
                
                triangles += indices_per_chunk / 3;
                index_offset += indices_per_chunk;
            }
            
            triangles
        } else {
            0
        }
    }
}