use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use std::f32::consts::PI;

pub struct Crystal {
    pub pos: Vec3,
    pub rotation: f32,
    pub collected: bool,
    pub size: f32,
    pub glow_phase: f32,
}

impl Crystal {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            rotation: 0.0,
            collected: false,
            size: 15.0,
            glow_phase: rand::random::<f32>() * PI * 2.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        // Rotate the crystal
        self.rotation += dt * 1.5;
        
        // Pulsing glow effect
        self.glow_phase += dt * 3.0;
    }
    
    pub fn collect(&mut self) {
        self.collected = true;
    }
    
    pub fn is_collected(&self) -> bool {
        self.collected
    }
}

impl Drawable for Crystal {
    fn draw(&self, renderer: &mut Renderer) {
        if self.collected {
            return;
        }
        
        // Calculate pulsing size
        let pulse_scale = 1.0 + (self.glow_phase.sin() * 0.2);
        let size = self.size * pulse_scale;
        
        // Draw octahedron shape (double pyramid)
        let top = self.pos + Vec3::new(0.0, size, 0.0);
        let bottom = self.pos - Vec3::new(0.0, size, 0.0);
        
        // Middle vertices rotated
        let angle_offset = self.rotation;
        let vertices = [
            self.pos + Vec3::new(angle_offset.cos() * size, 0.0, angle_offset.sin() * size),
            self.pos + Vec3::new((angle_offset + PI * 0.5).cos() * size, 0.0, (angle_offset + PI * 0.5).sin() * size),
            self.pos + Vec3::new((angle_offset + PI).cos() * size, 0.0, (angle_offset + PI).sin() * size),
            self.pos + Vec3::new((angle_offset + PI * 1.5).cos() * size, 0.0, (angle_offset + PI * 1.5).sin() * size),
        ];
        
        // Top pyramid
        for i in 0..4 {
            let next = (i + 1) % 4;
            renderer.draw_triangle(top, vertices[i], vertices[next]);
        }
        
        // Bottom pyramid
        for i in 0..4 {
            let next = (i + 1) % 4;
            renderer.draw_triangle(bottom, vertices[next], vertices[i]);
        }
        
        // Middle band
        for i in 0..4 {
            let next = (i + 1) % 4;
            renderer.draw_triangle(vertices[i], vertices[next], self.pos);
        }
        
        // Draw glow lines emanating from crystal
        let glow_size = size * 2.0;
        for i in 0..8 {
            let angle = i as f32 * PI * 0.25 + self.rotation * 0.5;
            let outer = self.pos + Vec3::new(angle.cos() * glow_size, 0.0, angle.sin() * glow_size);
            renderer.draw_line(self.pos, outer);
        }
    }
}