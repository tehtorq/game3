use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;
use crate::renderer::{Renderer, Drawable};

#[derive(Clone)]
pub struct Particle {
    pub pos: Vec3,
    vel: Vec3,
    pub lifetime: f32,
}

impl Particle {
    pub fn new(pos: Vec3) -> Self {
        let mut rng = thread_rng();
        let angle_h = rng.gen_range(0.0..PI * 2.0);
        let angle_v = rng.gen_range(-PI/4.0..PI/4.0);
        let speed = rng.gen_range(50.0..200.0);
        
        Self {
            pos,
            vel: Vec3::new(
                angle_h.cos() * angle_v.cos() * speed,
                angle_v.sin() * speed,
                angle_h.sin() * angle_v.cos() * speed
            ),
            lifetime: rng.gen_range(0.3..0.8),
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.pos += self.vel * dt;
        self.lifetime -= dt;
        self.lifetime > 0.0
    }
}

impl Drawable for Particle {
    fn draw(&self, renderer: &mut Renderer) {
        let size = self.lifetime * 10.0;
        renderer.draw_line(
            self.pos - Vec3::new(size, 0.0, 0.0),
            self.pos + Vec3::new(size, 0.0, 0.0)
        );
        renderer.draw_line(
            self.pos - Vec3::new(0.0, size, 0.0),
            self.pos + Vec3::new(0.0, size, 0.0)
        );
    }
}