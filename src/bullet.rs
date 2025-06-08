use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::player::Player;
use crate::enemy::Enemy;

#[derive(Clone)]
pub enum BulletType {
    Player,
    Enemy,
}

#[derive(Clone)]
pub struct Bullet {
    pub pos: Vec3,
    pub vel: Vec3,
    pub bullet_type: BulletType,
}

impl Bullet {
    pub fn new(player: &Player) -> Self {
        // Fire bullet in the direction the player is facing
        let forward = Vec3::new(
            -player.rotation.sin(),
            0.0,
            -player.rotation.cos()
        );
        Self {
            pos: player.pos + forward * 20.0 + Vec3::new(0.0, -5.0, 0.0),
            vel: forward * 1200.0,
            bullet_type: BulletType::Player,
        }
    }
    
    pub fn new_enemy(enemy: &Enemy, direction: Vec3) -> Self {
        Self {
            pos: enemy.pos + direction * 30.0,
            vel: direction * 600.0, // Enemy bullets are slower
            bullet_type: BulletType::Enemy,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }
}

impl Drawable for Bullet {
    fn draw(&self, renderer: &mut Renderer) {
        // Draw bullet as a small 3D cross
        let size = match self.bullet_type {
            BulletType::Player => 5.0,
            BulletType::Enemy => 7.0, // Enemy bullets are larger
        };
        renderer.draw_line(
            self.pos - Vec3::new(size, 0.0, 0.0),
            self.pos + Vec3::new(size, 0.0, 0.0)
        );
        renderer.draw_line(
            self.pos - Vec3::new(0.0, size, 0.0),
            self.pos + Vec3::new(0.0, size, 0.0)
        );
        renderer.draw_line(
            self.pos - Vec3::new(0.0, 0.0, size),
            self.pos + Vec3::new(0.0, 0.0, size)
        );
    }
}