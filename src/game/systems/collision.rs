use glam::Vec3;
use crate::bullet::{Bullet, BulletType};
use crate::enemy::{Enemy, EnemyType};
use crate::particle::{Particle, ParticleType};
use crate::mine::Mine;
use crate::base::Base;
use crate::player::Player;

pub struct CollisionSystem;

impl CollisionSystem {
    /// Check bullet-base collisions and handle damage
    pub fn check_bullet_base_collisions(
        bullets: &[Bullet],
        bases: &mut Vec<Base>,
        particles: &mut Vec<Particle>,
        score: &mut u32,
    ) -> (Vec<usize>, Vec<(usize, usize, Vec3, Vec3)>) {
        let mut bullets_to_remove = Vec::new();
        let mut turret_hits = Vec::new();
        
        for (bi, bullet) in bullets.iter().enumerate() {
            if matches!(bullet.bullet_type, BulletType::Player) {
                let mut hit_something = false;
                
                // Check main base collision
                for base in bases.iter_mut() {
                    if base.is_active {
                        let dist = (bullet.pos - base.pos).length();
                        if dist < 50.0 {
                            bullets_to_remove.push(bi);
                            base.take_damage(25.0);
                            hit_something = true;
                            
                            // Create impact particles
                            for _ in 0..10 {
                                particles.push(Particle::new_with_type(bullet.pos, ParticleType::Spark));
                            }
                            
                            if !base.is_active {
                                // Base destroyed - big explosion
                                for _ in 0..50 {
                                    particles.push(Particle::new_with_type(base.pos, ParticleType::Debris));
                                }
                                *score += 500;
                            }
                            break;
                        }
                    }
                }
                
                // Check ground turret collisions
                if !hit_something {
                    for (base_idx, base) in bases.iter().enumerate() {
                        if base.is_active {
                            let turret_positions = base.get_ground_turret_positions();
                            for (turret_idx, turret_pos, is_active) in turret_positions {
                                if is_active {
                                    let dist = (bullet.pos - turret_pos).length();
                                    if dist < 20.0 {
                                        bullets_to_remove.push(bi);
                                        turret_hits.push((base_idx, turret_idx, bullet.pos, turret_pos));
                                        hit_something = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if hit_something {
                            break;
                        }
                    }
                }
            }
        }
        
        (bullets_to_remove, turret_hits)
    }
    
    /// Process turret hits after collision detection
    pub fn process_turret_hits(
        turret_hits: Vec<(usize, usize, Vec3, Vec3)>,
        bases: &mut Vec<Base>,
        particles: &mut Vec<Particle>,
        score: &mut u32,
    ) {
        for (base_idx, turret_idx, bullet_pos, turret_pos) in turret_hits {
            if bases[base_idx].damage_ground_turret(turret_idx, 25.0) {
                // Turret destroyed
                for _ in 0..20 {
                    particles.push(Particle::new_with_type(turret_pos, ParticleType::Explosion));
                }
                *score += 100;
            } else {
                // Just damaged
                for _ in 0..3 {
                    particles.push(Particle::new_with_type(bullet_pos, ParticleType::Spark));
                }
                for _ in 0..2 {
                    particles.push(Particle::new_with_type(bullet_pos, ParticleType::Debris));
                }
            }
        }
    }
    
    /// Check bullet-enemy collisions
    pub fn check_bullet_enemy_collisions(
        bullets: &[Bullet],
        enemies: &[Enemy],
        particles: &mut Vec<Particle>,
        score: &mut u32,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut bullets_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();
        
        for (bi, bullet) in bullets.iter().enumerate() {
            if matches!(bullet.bullet_type, BulletType::Player) {
                for (ei, enemy) in enemies.iter().enumerate() {
                    let dist = (bullet.pos - enemy.pos).length();
                    if dist < 30.0 {
                        bullets_to_remove.push(bi);
                        enemies_to_remove.push(ei);
                        
                        // Create explosion particles
                        for _ in 0..8 {
                            particles.push(Particle::new_with_type(enemy.pos, ParticleType::Explosion));
                        }
                        for _ in 0..12 {
                            particles.push(Particle::new_with_type(enemy.pos, ParticleType::Spark));
                        }
                        for _ in 0..5 {
                            particles.push(Particle::new_with_type(enemy.pos, ParticleType::Debris));
                        }
                        
                        *score += Self::get_enemy_score(&enemy.enemy_type);
                        break;
                    }
                }
            }
        }
        
        (bullets_to_remove, enemies_to_remove)
    }
    
    /// Check enemy bullet collisions with player
    pub fn check_enemy_bullet_player_collisions(
        bullets: &[Bullet],
        player: &mut Player,
        enemies: &[Enemy],
        particles: &mut Vec<Particle>,
        player_invulnerable_timer: &mut f32,
    ) -> (Vec<usize>, Vec<(Enemy, Vec3)>) {
        let mut bullets_to_remove = Vec::new();
        let mut reflected_bullets = Vec::new();
        
        for (bi, bullet) in bullets.iter().enumerate() {
            match bullet.bullet_type {
                BulletType::Enemy | BulletType::HeavyTurret => {
                    // Check collision with player
                    let dist = (bullet.pos - player.pos).length();
                    if dist < 25.0 && *player_invulnerable_timer <= 0.0 {
                        bullets_to_remove.push(bi);
                        
                        let damage = match bullet.bullet_type {
                            BulletType::HeavyTurret => 0.5,
                            _ => 0.25,
                        };
                        
                        if player.take_damage(damage) {
                            *player_invulnerable_timer = 1.0;
                            
                            for _ in 0..10 {
                                particles.push(Particle::new_with_type(player.pos, ParticleType::Explosion));
                            }
                            for _ in 0..15 {
                                particles.push(Particle::new_with_type(player.pos, ParticleType::Spark));
                            }
                        } else {
                            for _ in 0..8 {
                                particles.push(Particle::new_with_type(player.pos, ParticleType::Shield));
                            }
                        }
                    }
                    
                    // Check reflection by reflector enemies
                    for enemy in enemies.iter() {
                        if enemy.is_reflecting() {
                            let dist = (bullet.pos - enemy.pos).length();
                            if dist < 60.0 {
                                bullets_to_remove.push(bi);
                                let reflect_dir = (player.pos - enemy.pos).normalize_or_zero();
                                if reflect_dir.length() > 0.0 {
                                    reflected_bullets.push((enemy.clone(), reflect_dir));
                                }
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        (bullets_to_remove, reflected_bullets)
    }
    
    /// Check player-enemy collisions
    pub fn check_player_enemy_collisions(
        player: &mut Player,
        enemies: &[Enemy],
        particles: &mut Vec<Particle>,
        player_invulnerable_timer: &mut f32,
    ) -> Vec<usize> {
        let mut enemies_to_remove = Vec::new();
        
        for (ei, enemy) in enemies.iter().enumerate() {
            let dist = (player.pos - enemy.pos).length();
            if dist < 40.0 && *player_invulnerable_timer <= 0.0 {
                enemies_to_remove.push(ei);
                
                if player.take_damage(0.5) {
                    *player_invulnerable_timer = 1.0;
                }
                
                for _ in 0..15 {
                    particles.push(Particle::new_with_type(enemy.pos, ParticleType::Explosion));
                }
                for _ in 0..10 {
                    particles.push(Particle::new_with_type(enemy.pos, ParticleType::Debris));
                }
            }
        }
        
        enemies_to_remove
    }
    
    /// Check player-mine collisions
    pub fn check_player_mine_collisions(
        player: &mut Player,
        mines: &[Mine],
        particles: &mut Vec<Particle>,
        player_invulnerable_timer: &mut f32,
    ) -> Vec<usize> {
        let mut mines_to_remove = Vec::new();
        
        for (mi, mine) in mines.iter().enumerate() {
            let dist = (player.pos - mine.pos).length();
            if dist < 30.0 && *player_invulnerable_timer <= 0.0 {
                mines_to_remove.push(mi);
                
                if player.take_damage(0.75) {
                    *player_invulnerable_timer = 1.5;
                }
                
                for _ in 0..20 {
                    particles.push(Particle::new_with_type(mine.pos, ParticleType::Explosion));
                }
                for _ in 0..25 {
                    particles.push(Particle::new_with_type(mine.pos, ParticleType::Spark));
                }
                for _ in 0..15 {
                    particles.push(Particle::new_with_type(mine.pos, ParticleType::Debris));
                }
            }
        }
        
        mines_to_remove
    }
    
    /// Check for special enemy abilities affecting the player
    pub fn check_enemy_abilities(
        player: &mut Player,
        enemies: &[Enemy],
        particles: &mut Vec<Particle>,
        player_invulnerable_timer: &mut f32,
        player_slowed: &mut bool,
        player_slow_timer: &mut f32,
    ) {
        for enemy in enemies.iter() {
            // Laser enemy damage
            if enemy.is_laser_active() && *player_invulnerable_timer <= 0.0 {
                if enemy.is_player_in_laser(player.pos) {
                    if player.take_damage(0.01) {
                        for _ in 0..3 {
                            particles.push(Particle::new_with_type(player.pos, ParticleType::Spark));
                        }
                    }
                }
            }
            
            // Disruptor slow effect
            if enemy.is_disrupting() && !*player_slowed {
                let dist = (player.pos - enemy.pos).length();
                if dist < 300.0 {
                    *player_slowed = true;
                    *player_slow_timer = 3.0;
                    
                    for _ in 0..10 {
                        particles.push(Particle::new_with_type(player.pos, ParticleType::Shield));
                    }
                }
            }
            
            // Vortex pull effect
            if enemy.is_vortex_active() {
                let to_enemy = enemy.pos - player.pos;
                let dist = to_enemy.length();
                if dist > 50.0 && dist < 500.0 {
                    let pull_strength = (1.0 - dist / 500.0) * 150.0;
                    let pull_dir = to_enemy.normalize();
                    player.pos += pull_dir * pull_strength * 0.016; // Assuming 60 FPS
                }
            }
        }
    }
    
    /// Get score value for enemy type
    fn get_enemy_score(enemy_type: &EnemyType) -> u32 {
        match enemy_type {
            EnemyType::Cube => 10,
            EnemyType::Pyramid => 20,
            EnemyType::Spinner => 30,
            EnemyType::Hunter => 40,
            EnemyType::Guardian => 50,
            EnemyType::Laser => 60,
            EnemyType::Swarm => 5,
            EnemyType::Phaser => 80,
            EnemyType::Shield => 100,
            EnemyType::Bomber => 70,
            EnemyType::Disruptor => 60,
            EnemyType::Carrier => 150,
            EnemyType::Reflector => 90,
            EnemyType::Vortex => 120,
        }
    }
}