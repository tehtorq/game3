use miniquad::*;
use glam::{Vec2, Vec3};
use crate::vertex::Vertex;
use crate::shader::UniformsTerrainGPU;

const CLIPMAP_SIZE: usize = 127; // Must be (2^n - 1) for proper nesting
const CLIPMAP_LEVELS: usize = 8; // Number of clipmap levels

pub struct ClipRing {
    level: usize,
    scale: f32,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    // Ring is made of 4 L-shaped pieces and a center patch
    l_shapes: [LShape; 4],
    center_patch: Option<CenterPatch>,
    trim_regions: Vec<TrimRegion>,
}

struct LShape {
    vertices_start: usize,
    indices_start: usize,
    indices_count: usize,
}

struct CenterPatch {
    vertices_start: usize,
    indices_start: usize,
    indices_count: usize,
}

struct TrimRegion {
    vertices_start: usize,
    indices_start: usize,
    indices_count: usize,
}

pub struct TerrainClipmap {
    rings: Vec<ClipRing>,
    ring_vertex_buffer: BufferId,
    ring_index_buffer: BufferId,
    grid_size: usize,
    base_scale: f32,
}

impl TerrainClipmap {
    pub fn new(ctx: &mut dyn RenderingBackend, base_scale: f32) -> Self {
        let grid_size = CLIPMAP_SIZE;
        
        // Generate base grid vertices
        let (vertices, indices) = Self::generate_grid(grid_size);
        
        // Create buffers
        let ring_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let ring_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create rings
        let mut rings = Vec::new();
        for level in 0..CLIPMAP_LEVELS {
            let scale = base_scale * (1 << level) as f32;
            rings.push(Self::create_ring(level, scale, &vertices, &indices));
        }
        
        Self {
            rings,
            ring_vertex_buffer,
            ring_index_buffer,
            grid_size,
            base_scale,
        }
    }
    
    fn generate_grid(size: usize) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate vertices for a regular grid
        for y in 0..=size {
            for x in 0..=size {
                let fx = x as f32 / size as f32 - 0.5;
                let fy = y as f32 / size as f32 - 0.5;
                
                vertices.push(Vertex::new(fx, 0.0, fy));
            }
        }
        
        // Generate indices
        for y in 0..size {
            for x in 0..size {
                let idx = (y * (size + 1) + x) as u32;
                
                // First triangle
                indices.push(idx);
                indices.push(idx + 1);
                indices.push(idx + size as u32 + 1);
                
                // Second triangle
                indices.push(idx + 1);
                indices.push(idx + size as u32 + 2);
                indices.push(idx + size as u32 + 1);
            }
        }
        
        (vertices, indices)
    }
    
    fn create_ring(level: usize, scale: f32, vertices: &[Vertex], indices: &[u32]) -> ClipRing {
        // For now, create a simple full grid ring
        // In a full implementation, this would create L-shapes and trim regions
        let l_shapes = [
            LShape { vertices_start: 0, indices_start: 0, indices_count: indices.len() / 4 },
            LShape { vertices_start: 0, indices_start: indices.len() / 4, indices_count: indices.len() / 4 },
            LShape { vertices_start: 0, indices_start: indices.len() / 2, indices_count: indices.len() / 4 },
            LShape { vertices_start: 0, indices_start: 3 * indices.len() / 4, indices_count: indices.len() / 4 },
        ];
        
        ClipRing {
            level,
            scale,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            l_shapes,
            center_patch: None,
            trim_regions: Vec::new(),
        }
    }
    
    pub fn update(&mut self, player_pos: Vec3) {
        // Update ring positions based on player position
        // Snap to grid positions to prevent swimming
        for ring in &mut self.rings {
            let grid_size = ring.scale;
            let snapped_x = (player_pos.x / grid_size).floor() * grid_size;
            let snapped_z = (player_pos.z / grid_size).floor() * grid_size;
            
            // Update ring center position
            // In full implementation, would update L-shape positions
        }
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: glam::Mat4, player_pos: Vec3) -> i32 {
        let mut total_triangles = 0;
        
        ctx.apply_pipeline(pipeline);
        
        let bindings = Bindings {
            vertex_buffers: vec![self.ring_vertex_buffer],
            index_buffer: self.ring_index_buffer,
            images: vec![],
        };
        
        ctx.apply_bindings(&bindings);
        
        // Draw each ring from finest to coarsest
        for (i, ring) in self.rings.iter().enumerate() {
            // Calculate ring center position (snapped to grid)
            let grid_size = ring.scale * 2.0; // Each vertex covers 2x the scale
            let ring_center_x = (player_pos.x / grid_size).floor() * grid_size;
            let ring_center_z = (player_pos.z / grid_size).floor() * grid_size;
            
            // Draw the ring with proper offset and scale
            let chunk_offset = [ring_center_x, ring_center_z];
            
            // Morph factor for blending with next level
            let morph_factor = if i < self.rings.len() - 1 {
                // Calculate morph factor based on position within ring
                let next_ring = &self.rings[i + 1];
                let transition_start = ring.scale * CLIPMAP_SIZE as f32 * 0.4;
                let transition_end = ring.scale * CLIPMAP_SIZE as f32 * 0.5;
                
                let dist_from_center = ((player_pos.x - ring_center_x).powi(2) + 
                                       (player_pos.z - ring_center_z).powi(2)).sqrt();
                
                ((dist_from_center - transition_start) / (transition_end - transition_start)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            
            let uniforms = UniformsTerrainGPU::new(
                mvp,
                [0.2, 0.8, 0.2], // Green terrain
                morph_factor,
                chunk_offset,
                ring.scale,
                player_pos
            );
            
            ctx.apply_uniforms(UniformsSource::table(&uniforms));
            
            // Draw the ring
            let triangle_count = ring.indices.len() as i32 / 3;
            ctx.draw(0, ring.indices.len() as i32, 1);
            total_triangles += triangle_count;
        }
        
        total_triangles
    }
}