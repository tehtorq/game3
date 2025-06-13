use miniquad::*;
use glam::Vec3;
use crate::vertex::Vertex;
use crate::bullet::{Bullet, BulletType};

pub struct BulletInstancingSystem {
    cube_vertex_buffer: BufferId,
    cube_index_buffer: BufferId,
    instance_buffer: BufferId,
    index_count: i32,
    max_bullets: usize,
}

impl BulletInstancingSystem {
    pub fn new(ctx: &mut dyn RenderingBackend, max_bullets: usize) -> Self {
        // Create a small cube mesh for bullets
        let (vertices, indices) = Self::create_bullet_cube();
        
        let cube_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let cube_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance buffer for bullet positions and scales
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<BulletInstanceData>(max_bullets)
        );
        
        Self {
            cube_vertex_buffer,
            cube_index_buffer,
            instance_buffer,
            index_count: indices.len() as i32,
            max_bullets,
        }
    }
    
    fn create_bullet_cube() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Create a cube with size 1.0 (will be scaled per bullet type)
        let size = 0.5; // Half size for -0.5 to 0.5 range
        
        // Define the 8 vertices of the cube
        let positions = [
            [-size, -size, -size], // 0
            [ size, -size, -size], // 1
            [ size,  size, -size], // 2
            [-size,  size, -size], // 3
            [-size, -size,  size], // 4
            [ size, -size,  size], // 5
            [ size,  size,  size], // 6
            [-size,  size,  size], // 7
        ];
        
        // Define the 12 triangles (2 per face, 6 faces)
        let face_indices = [
            // Front face
            [0, 1, 2], [0, 2, 3],
            // Back face
            [5, 4, 7], [5, 7, 6],
            // Left face
            [4, 0, 3], [4, 3, 7],
            // Right face
            [1, 5, 6], [1, 6, 2],
            // Top face
            [3, 2, 6], [3, 6, 7],
            // Bottom face
            [4, 5, 1], [4, 1, 0],
        ];
        
        // Add vertices and indices for each triangle
        for triangle in &face_indices {
            let base_idx = vertices.len() as u32;
            
            // Add three vertices for this triangle
            for (i, &idx) in triangle.iter().enumerate() {
                let pos = positions[idx];
                
                // Set up barycentric coordinates for wireframe effect
                let barycentric = match i {
                    0 => [1.0, 0.0, 0.0],
                    1 => [0.0, 1.0, 0.0],
                    2 => [0.0, 0.0, 1.0],
                    _ => [0.0, 0.0, 0.0],
                };
                
                vertices.push(Vertex::with_barycentric(
                    pos[0], pos[1], pos[2],
                    barycentric[0], barycentric[1], barycentric[2]
                ));
            }
            
            // Add indices for this triangle
            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
        }
        
        (vertices, indices)
    }
    
    pub fn update_instances(&mut self, ctx: &mut dyn RenderingBackend, bullets: &[Bullet]) -> i32 {
        let mut instance_data = Vec::new();
        
        for bullet in bullets.iter().take(self.max_bullets) {
            // Scale based on bullet type
            let scale = match bullet.bullet_type {
                BulletType::Player => 5.0,      // Base size
                BulletType::Enemy => 5.0,       // Same as player
                BulletType::HeavyTurret => 2.5, // Half of player size
            };
            
            instance_data.push(BulletInstanceData {
                position: [bullet.pos.x, bullet.pos.y, bullet.pos.z],
                scale,
            });
        }
        
        let instance_count = instance_data.len() as i32;
        
        if instance_count > 0 {
            ctx.buffer_update(self.instance_buffer, BufferSource::slice(&instance_data));
        }
        
        instance_count
    }
    
    pub fn vertex_buffer(&self) -> BufferId {
        self.cube_vertex_buffer
    }
    
    pub fn instance_buffer(&self) -> BufferId {
        self.instance_buffer
    }
    
    pub fn index_buffer(&self) -> BufferId {
        self.cube_index_buffer
    }
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct BulletInstanceData {
    position: [f32; 3],
    scale: f32,
}