use glam::Vec3;
use crate::bullet::{Bullet, BulletType};
use crate::enemy::Enemy;
use crate::player::Player;
use crate::base::Base;

pub struct CombatSystem;

impl CombatSystem {
    /// Create a player bullet
    pub fn create_player_bullet(player: &Player) -> Bullet {
        Bullet::new(player)
    }
    
    /// Create turret bullets from bases
    pub fn process_base_turret_shots(bases: &mut [Base], player_pos: Vec3, dt: f32) -> Vec<Bullet> {
        let mut turret_bullets = Vec::new();
        
        for base in bases.iter_mut() {
            let shots = base.update(player_pos, dt);
            // Convert shot tuples to actual bullets
            for (start_pos, direction, is_heavy) in shots {
                let bullet_type = if is_heavy { 
                    BulletType::HeavyTurret 
                } else { 
                    BulletType::Enemy 
                };
                turret_bullets.push(Bullet::new_at_position(start_pos, direction, bullet_type));
            }
        }
        
        turret_bullets
    }
    
    /// Process enemy shooting behavior
    pub fn process_enemy_shots(enemies: &mut [Enemy]) -> Vec<Bullet> {
        let mut enemy_bullets = Vec::new();
        
        for enemy in enemies.iter_mut() {
            if let Some(bullet) = enemy.try_shoot() {
                enemy_bullets.push(bullet);
            }
        }
        
        enemy_bullets
    }
    
    /// Create reflected bullets
    pub fn create_reflected_bullet(enemy_pos: Vec3, direction: Vec3) -> Bullet {
        let mut bullet = Bullet {
            pos: enemy_pos,
            vel: direction * 600.0,
            lifetime: 3.0,
            bullet_type: BulletType::Player,
        };
        bullet.pos.y += 10.0;
        bullet
    }
    
    /// Alert enemies to sound at a position
    pub fn alert_enemies_to_sound(enemies: &mut [Enemy], sound_pos: Vec3, alert_radius: f32) {
        for enemy in enemies.iter_mut() {
            let dist = (enemy.pos - sound_pos).length();
            if dist < alert_radius {
                enemy.alert_to_sound(sound_pos);
            }
        }
    }
}