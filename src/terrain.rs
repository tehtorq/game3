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
        let scale1 = 0.003;
        let scale2 = 0.007;
        let scale3 = 0.015;
        let height_scale = 40.0;
        
        // Multiple octaves of noise for more interesting terrain
        let h1 = (x * scale1).sin() * (z * scale1).cos() * height_scale;
        let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * height_scale * 0.5;
        let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * height_scale * 0.25;
        
        h1 + h2 + h3
    }
}

impl Drawable for TerrainChunk {
    fn draw(&self, renderer: &mut Renderer) {
        let terrain_y = 0.0; // Put terrain at y=0 for debugging
        
        // Draw a simple grid square for each chunk
        let size = 300.0;
        let x1 = self.x_offset - size;
        let x2 = self.x_offset + size;
        let z1 = self.z_offset - size;
        let z2 = self.z_offset + size;
        
        // Draw outline of chunk
        renderer.draw_line(Vec3::new(x1, terrain_y, z1), Vec3::new(x2, terrain_y, z1));
        renderer.draw_line(Vec3::new(x2, terrain_y, z1), Vec3::new(x2, terrain_y, z2));
        renderer.draw_line(Vec3::new(x2, terrain_y, z2), Vec3::new(x1, terrain_y, z2));
        renderer.draw_line(Vec3::new(x1, terrain_y, z2), Vec3::new(x1, terrain_y, z1));
        
        // Draw cross in middle
        renderer.draw_line(Vec3::new(x1, terrain_y, self.z_offset), Vec3::new(x2, terrain_y, self.z_offset));
        renderer.draw_line(Vec3::new(self.x_offset, terrain_y, z1), Vec3::new(self.x_offset, terrain_y, z2));
    }
}