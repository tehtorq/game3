use glam::Vec3;
use crate::renderer::{Renderer, Drawable};

#[derive(Clone, Copy)]
pub struct TerrainChunk {
    pub x_offset: f32,
    pub z_offset: f32,
    grid_size: usize,
    spacing: f32,
}

impl TerrainChunk {
    pub fn new(x_offset: f32, z_offset: f32) -> Self {
        Self {
            x_offset,
            z_offset,
            grid_size: 25,
            spacing: 25.0,
        }
    }
    
    fn height_at(&self, x: f32, z: f32) -> f32 {
        let scale1 = 0.002;  // Larger scale = gentler hills
        let scale2 = 0.007;
        let scale3 = 0.015;
        let height_scale = 60.0;  // Increased height variation
        
        // Multiple octaves of noise for more interesting terrain
        let h1 = (x * scale1).sin() * (z * scale1).cos() * height_scale;
        let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * height_scale * 0.5;
        let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * height_scale * 0.25;
        
        h1 + h2 + h3
    }
}

impl Drawable for TerrainChunk {
    fn draw(&self, renderer: &mut Renderer) {
        let terrain_y_base = -30.0;
        let grid_size = 4; // 4x4 grid of quads per chunk
        let cell_size = 20.0;
        let half_size = 40.0;
        
        // Generate grid of triangles
        for i in 0..grid_size {
            for j in 0..grid_size {
                let x1 = self.x_offset - half_size + (i as f32) * cell_size;
                let x2 = x1 + cell_size;
                let z1 = self.z_offset - half_size + (j as f32) * cell_size;
                let z2 = z1 + cell_size;
                
                // Get heights at corners
                let y11 = self.height_at(x1, z1) + terrain_y_base;
                let y21 = self.height_at(x2, z1) + terrain_y_base;
                let y12 = self.height_at(x1, z2) + terrain_y_base;
                let y22 = self.height_at(x2, z2) + terrain_y_base;
                
                // Create vertices
                let v1 = Vec3::new(x1, y11, z1);
                let v2 = Vec3::new(x2, y21, z1);
                let v3 = Vec3::new(x1, y12, z2);
                let v4 = Vec3::new(x2, y22, z2);
                
                // Draw two triangles to form a quad
                renderer.draw_triangle(v1, v2, v3);
                renderer.draw_triangle(v2, v4, v3);
            }
        }
    }
}