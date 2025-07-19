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
    pub lifetime: f32,
    pub bullet_type: BulletType,
}

impl Bullet {
    pub fn new(player: &Player) -> Self {
        // Fire bullet in the direction the player is facing, including pitch
        let forward = player.v_forward();
        
        // Spawn bullet slightly in front of the player
        let spawn_offset = forward * 30.0;
        let spawn_pos = player.pos + spawn_offset;
        
        
        Self {
            pos: spawn_pos,
            vel: forward * 1200.0,
            lifetime: 5.0,
            bullet_type: BulletType::Player,
        }
    }
    
    pub fn new_enemy(enemy: &Enemy, direction: Vec3) -> Self {
        Self {
            pos: enemy.pos + direction * 30.0,
            vel: direction * 600.0, // Enemy bullets are slower
            lifetime: 5.0,
            bullet_type: BulletType::Enemy,
        }
    }
    
    pub fn new_at_position(pos: Vec3, direction: Vec3, bullet_type: BulletType) -> Self {
        Self {
            pos,
            vel: direction * 600.0,
            lifetime: 5.0,
            bullet_type,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.lifetime -= dt;
    }
    
    pub fn get_debug_info(&self) -> String {
        format!("pos: ({:.1}, {:.1}, {:.1}), vel: ({:.1}, {:.1}, {:.1})", 
                self.pos.x, self.pos.y, self.pos.z,
                self.vel.x, self.vel.y, self.vel.z)
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