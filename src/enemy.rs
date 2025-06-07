use glam::Vec3;
use rand::prelude::*;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;

#[derive(Clone, Copy)]
pub enum EnemyType {
    Cube,
    Pyramid,
    Spinner,
}

#[derive(Clone)]
pub struct Enemy {
    pub pos: Vec3,
    pub vel: Vec3,
    pub enemy_type: EnemyType,
    rotation: Vec3,
    rotation_speed: Vec3,
}

impl Enemy {
    pub fn new(x: f32, z: f32, enemy_type: EnemyType) -> Self {
        let mut rng = thread_rng();
        Self {
            pos: Vec3::new(x, rng.gen_range(40.0..80.0), z),
            vel: Vec3::new(
                rng.gen_range(-30.0..30.0),
                rng.gen_range(-10.0..10.0),
                rng.gen_range(-30.0..30.0)
            ),
            enemy_type,
            rotation: Vec3::ZERO,
            rotation_speed: Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.rotation += self.rotation_speed * dt;
    }
}

impl Drawable for Enemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.rotation.y, self.rotation.x, self.rotation.z);
        
        match self.enemy_type {
            EnemyType::Cube => {
                renderer.draw_cube(self.pos, 40.0, rotation);
            }
            EnemyType::Pyramid => {
                renderer.draw_pyramid(self.pos, 45.0, rotation);
            }
            EnemyType::Spinner => {
                renderer.draw_cube(self.pos, 30.0, rotation);
                let rotation2 = rotation_matrix(
                    -self.rotation.y * 2.0,
                    -self.rotation.x * 2.0,
                    self.rotation.z
                );
                renderer.draw_pyramid(self.pos, 35.0, rotation2);
            }
        }
    }
}