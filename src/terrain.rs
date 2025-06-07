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
        let terrain_y_base = -30.0; // Base terrain height
        
        // Draw a simple grid square for each chunk (same as before)
        let size = 40.0;
        let x1 = self.x_offset - size;
        let x2 = self.x_offset + size;
        let z1 = self.z_offset - size;
        let z2 = self.z_offset + size;
        
        // Draw outline of chunk with height
        let y1z1 = self.height_at(x1, z1) + terrain_y_base;
        let y2z1 = self.height_at(x2, z1) + terrain_y_base;
        let y2z2 = self.height_at(x2, z2) + terrain_y_base;
        let y1z2 = self.height_at(x1, z2) + terrain_y_base;
        
        renderer.draw_line(Vec3::new(x1, y1z1, z1), Vec3::new(x2, y2z1, z1));
        renderer.draw_line(Vec3::new(x2, y2z1, z1), Vec3::new(x2, y2z2, z2));
        renderer.draw_line(Vec3::new(x2, y2z2, z2), Vec3::new(x1, y1z2, z2));
        renderer.draw_line(Vec3::new(x1, y1z2, z2), Vec3::new(x1, y1z1, z1));
        
        // Draw cross in middle with height
        let ymid = self.height_at(self.x_offset, self.z_offset) + terrain_y_base;
        let y1mid = self.height_at(x1, self.z_offset) + terrain_y_base;
        let y2mid = self.height_at(x2, self.z_offset) + terrain_y_base;
        let ymid1 = self.height_at(self.x_offset, z1) + terrain_y_base;
        let ymid2 = self.height_at(self.x_offset, z2) + terrain_y_base;
        
        renderer.draw_line(Vec3::new(x1, y1mid, self.z_offset), Vec3::new(x2, y2mid, self.z_offset));
        renderer.draw_line(Vec3::new(self.x_offset, ymid1, z1), Vec3::new(self.x_offset, ymid2, z2));
    }
}