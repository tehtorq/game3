use miniquad::*;
use glam::{Vec3, Mat4};
use crate::vertex::Vertex;
use crate::shader::UniformsTerrainGPU;

// Ring-based GPU terrain using concentric rings instead of chunks
pub struct TerrainGPURings {
    rings: Vec<TerrainRing>,
    player_pos: Vec3,
}

struct TerrainRing {
    radius_inner: f32,
    radius_outer: f32,
    segments: usize,
    rings: usize,
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
    lod_level: u8,
}

impl TerrainGPURings {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        let mut rings = Vec::new();
        
        // Create concentric rings with decreasing detail
        // Ring 0: 0-500m, high detail
        rings.push(TerrainRing::new(ctx, 0.0, 500.0, 64, 32, 0));
        
        // Ring 1: 500-1200m, medium detail  
        rings.push(TerrainRing::new(ctx, 500.0, 1200.0, 48, 24, 1));
        
        // Ring 2: 1200-2500m, low detail
        rings.push(TerrainRing::new(ctx, 1200.0, 2500.0, 32, 16, 2));
        
        // Ring 3: 2500-5000m, very low detail
        rings.push(TerrainRing::new(ctx, 2500.0, 5000.0, 24, 12, 3));
        
        // Ring 4: 5000-10000m, ultra low detail
        rings.push(TerrainRing::new(ctx, 5000.0, 10000.0, 16, 8, 4));
        
        Self {
            rings,
            player_pos: Vec3::ZERO,
        }
    }
    
    pub fn update_player_position(&mut self, player_pos: Vec3) {
        self.player_pos = player_pos;
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: Mat4, color: [f32; 3]) -> i32 {
        let mut total_triangles = 0;
        
        ctx.apply_pipeline(pipeline);
        
        // Draw each ring
        for (i, ring) in self.rings.iter().enumerate() {
            let bindings = Bindings {
                vertex_buffers: vec![ring.vertex_buffer],
                index_buffer: ring.index_buffer,
                images: vec![],
            };
            
            ctx.apply_bindings(&bindings);
            
            // Calculate morph factor for smooth transitions
            let player_dist = (self.player_pos.x * self.player_pos.x + self.player_pos.z * self.player_pos.z).sqrt();
            let morph_factor = if i < self.rings.len() - 1 {
                let transition_start = ring.radius_outer * 0.8;
                let transition_end = ring.radius_outer;
                ((player_dist - transition_start) / (transition_end - transition_start)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            
            // Use player position as chunk offset for GPU terrain generation
            let chunk_offset = [self.player_pos.x, self.player_pos.z];
            let lod_scale = (ring.radius_outer - ring.radius_inner) / ring.segments as f32;
            
            let uniforms = UniformsTerrainGPU::new(
                mvp,
                color,
                morph_factor,
                chunk_offset,
                lod_scale,
                self.player_pos
            );
            
            ctx.apply_uniforms(UniformsSource::table(&uniforms));
            ctx.draw(0, ring.index_count, 1);
            
            total_triangles += ring.index_count / 3;
        }
        
        total_triangles
    }
}

impl TerrainRing {
    fn new(ctx: &mut dyn RenderingBackend, radius_inner: f32, radius_outer: f32, segments: usize, rings: usize, lod_level: u8) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate ring vertices
        for r in 0..=rings {
            let t = r as f32 / rings as f32;
            let radius = radius_inner + (radius_outer - radius_inner) * t;
            
            for s in 0..=segments {
                let angle = s as f32 * std::f32::consts::TAU / segments as f32;
                let x = angle.cos() * radius;
                let z = angle.sin() * radius;
                
                // Store relative position, GPU will add player offset
                vertices.push(Vertex::new(x, 0.0, z));
            }
        }
        
        // Generate indices
        for r in 0..rings {
            for s in 0..segments {
                let current = r * (segments + 1) + s;
                let next = current + segments + 1;
                
                // First triangle
                indices.push(current as u32);
                indices.push((current + 1) as u32);
                indices.push(next as u32);
                
                // Second triangle
                indices.push((current + 1) as u32);
                indices.push((next + 1) as u32);
                indices.push(next as u32);
            }
        }
        
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
            radius_inner,
            radius_outer,
            segments,
            rings,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as i32,
            lod_level,
        }
    }
}