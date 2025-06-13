use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;
use crate::renderer::{Renderer, Drawable};

#[derive(Clone)]
pub enum ParticleType {
    Explosion,
    Spark,
    Debris,
    Shield,
}

#[derive(Clone)]
pub struct Particle {
    pub pos: Vec3,
    vel: Vec3,
    pub lifetime: f32,
    max_lifetime: f32,
    particle_type: ParticleType,
    rotation: f32,
    rotation_speed: f32,
    size: f32,
}

impl Particle {
    pub fn new(pos: Vec3) -> Self {
        Self::new_with_type(pos, ParticleType::Explosion)
    }
    
    pub fn new_with_type(pos: Vec3, particle_type: ParticleType) -> Self {
        let mut rng = thread_rng();
        
        let (speed_min, speed_max, lifetime_min, lifetime_max, size) = match particle_type {
            ParticleType::Explosion => (100.0, 400.0, 0.3, 0.6, 1.0),
            ParticleType::Spark => (200.0, 600.0, 0.2, 0.4, 0.5),
            ParticleType::Debris => (50.0, 200.0, 0.5, 1.0, 1.5),
            ParticleType::Shield => (50.0, 150.0, 0.2, 0.3, 0.8),
        };
        
        let angle_h = rng.gen_range(0.0..PI * 2.0);
        let angle_v = match particle_type {
            ParticleType::Explosion => rng.gen_range(-PI/3.0..PI/3.0),
            ParticleType::Spark => rng.gen_range(-PI/6.0..PI/6.0),
            ParticleType::Debris => rng.gen_range(-PI/4.0..PI/4.0),
            ParticleType::Shield => rng.gen_range(-PI/2.0..PI/2.0),
        };
        let speed = rng.gen_range(speed_min..speed_max);
        let lifetime = rng.gen_range(lifetime_min..lifetime_max);
        
        Self {
            pos,
            vel: Vec3::new(
                angle_h.cos() * angle_v.cos() * speed,
                angle_v.sin() * speed,
                angle_h.sin() * angle_v.cos() * speed
            ),
            lifetime,
            max_lifetime: lifetime,
            particle_type,
            rotation: rng.gen_range(0.0..PI * 2.0),
            rotation_speed: rng.gen_range(-10.0..10.0),
            size: size * rng.gen_range(0.8..1.2),
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.pos += self.vel * dt;
        
        // Apply gravity to some particle types
        match self.particle_type {
            ParticleType::Debris => {
                self.vel.y -= 200.0 * dt; // Gravity
            }
            ParticleType::Explosion => {
                self.vel *= 0.98; // Air resistance
            }
            _ => {}
        }
        
        self.rotation += self.rotation_speed * dt;
        self.lifetime -= dt;
        self.lifetime > 0.0
    }
}

impl Drawable for Particle {
    fn draw(&self, renderer: &mut Renderer) {
        let alpha = self.lifetime / self.max_lifetime;
        let size = self.size * (4.0 + alpha * 4.0); // Size based on lifetime
        
        match self.particle_type {
            ParticleType::Explosion | ParticleType::Spark => {
                // Draw as a small pyramid/tetrahedron for 3D effect
                let cos_r = self.rotation.cos();
                let sin_r = self.rotation.sin();
                
                // Base vertices
                let v1 = self.pos + Vec3::new(cos_r * size, -size * 0.5, sin_r * size);
                let v2 = self.pos + Vec3::new(cos_r * size - sin_r * size * 0.866, -size * 0.5, sin_r * size + cos_r * size * 0.866);
                let v3 = self.pos + Vec3::new(cos_r * size + sin_r * size * 0.866, -size * 0.5, sin_r * size - cos_r * size * 0.866);
                let v4 = self.pos + Vec3::new(0.0, size, 0.0); // Top
                
                // Draw tetrahedron faces
                renderer.draw_triangle(v1, v2, v3); // Base
                renderer.draw_triangle(v1, v2, v4); // Side 1
                renderer.draw_triangle(v2, v3, v4); // Side 2
                renderer.draw_triangle(v3, v1, v4); // Side 3
            }
            ParticleType::Debris => {
                // Draw as a small cube
                let half_size = size * 0.5;
                let cos_r = self.rotation.cos();
                let sin_r = self.rotation.sin();
                
                // Rotated offsets
                let x1 = cos_r * half_size - sin_r * half_size;
                let z1 = sin_r * half_size + cos_r * half_size;
                let x2 = cos_r * half_size + sin_r * half_size;
                let z2 = sin_r * half_size - cos_r * half_size;
                
                // Front face
                renderer.draw_triangle(
                    self.pos + Vec3::new(-x1, -half_size, -z1),
                    self.pos + Vec3::new(x1, -half_size, -z1),
                    self.pos + Vec3::new(x1, half_size, -z1)
                );
                renderer.draw_triangle(
                    self.pos + Vec3::new(-x1, -half_size, -z1),
                    self.pos + Vec3::new(x1, half_size, -z1),
                    self.pos + Vec3::new(-x1, half_size, -z1)
                );
                
                // Back face
                renderer.draw_triangle(
                    self.pos + Vec3::new(-x2, -half_size, z2),
                    self.pos + Vec3::new(x2, -half_size, z2),
                    self.pos + Vec3::new(x2, half_size, z2)
                );
                renderer.draw_triangle(
                    self.pos + Vec3::new(-x2, -half_size, z2),
                    self.pos + Vec3::new(x2, half_size, z2),
                    self.pos + Vec3::new(-x2, half_size, z2)
                );
            }
            ParticleType::Shield => {
                // Draw as an octahedron (diamond shape)
                let top = self.pos + Vec3::new(0.0, size, 0.0);
                let bottom = self.pos + Vec3::new(0.0, -size, 0.0);
                
                let cos_r = self.rotation.cos();
                let sin_r = self.rotation.sin();
                let radius = size * 0.7;
                
                let v1 = self.pos + Vec3::new(cos_r * radius, 0.0, sin_r * radius);
                let v2 = self.pos + Vec3::new(-sin_r * radius, 0.0, cos_r * radius);
                let v3 = self.pos + Vec3::new(-cos_r * radius, 0.0, -sin_r * radius);
                let v4 = self.pos + Vec3::new(sin_r * radius, 0.0, -cos_r * radius);
                
                // Top pyramid
                renderer.draw_triangle(v1, v2, top);
                renderer.draw_triangle(v2, v3, top);
                renderer.draw_triangle(v3, v4, top);
                renderer.draw_triangle(v4, v1, top);
                
                // Bottom pyramid
                renderer.draw_triangle(v1, v2, bottom);
                renderer.draw_triangle(v2, v3, bottom);
                renderer.draw_triangle(v3, v4, bottom);
                renderer.draw_triangle(v4, v1, bottom);
            }
        }
    }
}