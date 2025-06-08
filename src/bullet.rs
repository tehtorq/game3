use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::player::Player;
use crate::enemy::Enemy;

#[derive(Clone)]
pub enum BulletType {
    Player,
    Enemy,
    HeavyTurret, // Ground turret bullets - more damage
}

#[derive(Clone)]
pub struct Bullet {
    pub pos: Vec3,
    pub vel: Vec3,
    pub bullet_type: BulletType,
}

impl Bullet {
    pub fn new(player: &Player) -> Self {
        // Fire bullet in the horizontal direction the player is facing
        // Only use yaw rotation, ignore pitch and banking for bullets
        let forward = Vec3::new(-player.rotation.sin(), 0.0, -player.rotation.cos());
        
        // Spawn bullet at player height, in front
        let spawn_offset = forward * 30.0;
        let spawn_pos = Vec3::new(
            player.pos.x + spawn_offset.x,
            player.pos.y,  // Same height as player
            player.pos.z + spawn_offset.z
        );
        
        Self {
            pos: spawn_pos,
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
    
    pub fn new_at_position(pos: Vec3, direction: Vec3, bullet_type: BulletType) -> Self {
        Self {
            pos,
            vel: direction * 600.0,
            bullet_type,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }
}

impl Drawable for Bullet {
    fn draw(&self, renderer: &mut Renderer) {
        // Draw all bullets as 3D crosses
        let size = match self.bullet_type {
            BulletType::Player => 5.0,
            BulletType::Enemy => 7.0,
            BulletType::HeavyTurret => 4.0, // Smaller but visible
        };
        
        // Draw 3D cross
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