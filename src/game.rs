use rand::prelude::*;
use std::f32::consts::PI;

use crate::player::Player;
use crate::enemy::{Enemy, EnemyType};
use crate::mine::Mine;

// Calculate terrain height at a given position (matches shader calculation)
fn terrain_height_at(x: f32, z: f32) -> f32 {
    let base_y = 20.0;
    
    // Large-scale terrain features
    let mut large_scale = (x * 0.0005).sin() * (z * 0.0007).sin() * 240.0;
    large_scale += (x * 0.0003 + 1.5).cos() * (z * 0.0004 - 0.8).sin() * 200.0;
    
    // Gentle slopes
    let mut gentle = (x * 0.0031).sin() * (z * 0.0027).cos() * 25.0;
    gentle += (x * 0.0047).sin() * (z * 0.0053).sin() * 20.0;
    
    // Roughness
    let mut roughness = (x * 0.0023 + 2.7).sin() * (z * 0.0019 - 1.3).cos();
    roughness += (x * 0.0041 - z * 0.0037).sin() * 0.5;
    roughness = (roughness + 1.5) / 3.0;
    roughness = if roughness < 0.6 { 0.0 } else if roughness > 0.8 { 1.0 } else { (roughness - 0.6) / 0.2 };
    
    // Bumpy details
    let mut bumps = 0.0;
    bumps += (x * 0.0173).sin() * (z * 0.0199).sin() * 20.0;
    bumps += (x * 0.0293 + 2.1).cos() * (z * 0.0311 - 1.7).sin() * 15.0;
    bumps += (x * 0.0519 + z * 0.0413).sin() * 8.0;
    bumps += (x * 0.0871 - z * 0.0926).sin() * 5.0;
    bumps += (x * 0.137).sin() * (z * 0.149).cos() * 3.0;
    
    base_y + large_scale + gentle + (bumps * roughness)
}
use crate::bullet::{Bullet, BulletType};
use crate::particle::Particle;
use crate::renderer::Renderer;
use glam::Vec3;

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub mines: Vec<Mine>,
    pub enemy_spawn_timer: f32,
    pub shoot_cooldown: f32,
    pub wave: u32,
    pub score: u32,
    pub player_invulnerable_timer: f32,  // Brief invulnerability after being hit
    pub player_slowed: bool,  // Disruptor effect
    pub player_slow_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            player: Player::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            mines: Vec::new(),
            enemy_spawn_timer: 3.0,  // Start with 3 second delay before first wave
            shoot_cooldown: 0.0,
            wave: 1,
            score: 0,
            player_invulnerable_timer: 0.0,
            player_slowed: false,
            player_slow_timer: 0.0,
        };
        
        game.spawn_wave();
        
        game
    }

    pub fn update(&mut self, left: bool, right: bool, up: bool, down: bool, shoot: bool, boost: bool, dt: f32) {
        // Update player (slow effect would need to be implemented in player.rs)
        self.player.update(left, right, up, down, boost, dt);
        
        // Update timers
        if self.player_invulnerable_timer > 0.0 {
            self.player_invulnerable_timer -= dt;
        }
        if self.player_slow_timer > 0.0 {
            self.player_slow_timer -= dt;
            if self.player_slow_timer <= 0.0 {
                self.player_slowed = false;
            }
        }
        
        // Handle shooting
        if shoot && self.shoot_cooldown <= 0.0 {
            self.bullets.push(Bullet::new(&self.player));
            self.shoot_cooldown = 0.15;
        }
        self.shoot_cooldown -= dt;
        
        // Update enemies with player awareness and handle attacks
        let mut enemy_bullets = Vec::new();
        let mut new_mines = Vec::new();
        let mut new_swarms = Vec::new();
        
        for enemy in &mut self.enemies {
            if let Some(attack_dir) = enemy.update_with_player(self.player.pos, dt) {
                enemy_bullets.push(Bullet::new_enemy(enemy, attack_dir));
            }
            
            // Handle special abilities
            if enemy.should_drop_mine() {
                new_mines.push(Mine::new(enemy.pos - Vec3::new(0.0, 20.0, 0.0)));
            }
            
            if enemy.should_spawn_swarm() {
                // Spawn 3 swarm enemies
                for i in 0..3 {
                    let angle = i as f32 * 2.0 * PI / 3.0;
                    let offset = Vec3::new(angle.cos() * 50.0, -20.0, angle.sin() * 50.0);
                    new_swarms.push(Enemy::new(
                        enemy.pos.x + offset.x,
                        enemy.pos.z + offset.z,
                        EnemyType::Swarm
                    ));
                }
            }
            
            // Apply vortex effect to player
            if enemy.get_vortex_strength() > 0.0 {
                let to_vortex = enemy.pos - self.player.pos;
                let distance = to_vortex.length();
                if distance < 500.0 && distance > 10.0 {  // Avoid too close to prevent normalize issues
                    let pull_strength = enemy.get_vortex_strength() * (1.0 - distance / 500.0) * dt;
                    let pull_dir = to_vortex.normalize_or_zero();  // Safe normalize
                    if pull_dir.length() > 0.0 {
                        self.player.pos += pull_dir * pull_strength;
                    }
                }
            }
            
            // Check disruptor wave
            if enemy.get_wave_active() {
                let wave_radius = enemy.get_wave_radius();
                let distance = (enemy.pos - self.player.pos).length();
                if distance < wave_radius && distance > wave_radius - 50.0 {
                    self.player_slowed = true;
                    self.player_slow_timer = 3.0;
                }
            }
        }
        
        self.bullets.extend(enemy_bullets);
        self.mines.extend(new_mines);
        self.enemies.extend(new_swarms);
        
        // Remove enemies that are too far away or behind
        let player_z = self.player.pos.z;
        self.enemies.retain(|e| {
            let distance = (e.pos - self.player.pos).length();
            // Keep enemies within 5000 units and don't remove them if they're still ahead
            distance < 5000.0 && (e.pos.z < player_z || e.pos.z > player_z - 2000.0)
        });
        
        // Update bullets and check terrain collisions
        let mut terrain_hits = Vec::new();
        for (i, bullet) in self.bullets.iter_mut().enumerate() {
            bullet.update(dt);
            
            // Check if bullet hit terrain
            let terrain_height = terrain_height_at(bullet.pos.x, bullet.pos.z);
            if bullet.pos.y < terrain_height {
                terrain_hits.push((i, bullet.pos));
            }
        }
        
        // Create particles for terrain hits
        for (_, hit_pos) in &terrain_hits {
            // Create dust/debris particles at impact point
            for _ in 0..10 {
                // Offset particle spawn slightly above terrain impact
                let particle_pos = *hit_pos + Vec3::new(0.0, 5.0, 0.0);
                self.particles.push(Particle::new(particle_pos));
            }
        }
        
        // Remove bullets that hit terrain
        let mut hits_to_remove: Vec<usize> = terrain_hits.iter().map(|(i, _)| *i).collect();
        hits_to_remove.sort_unstable();
        hits_to_remove.dedup();
        for &i in hits_to_remove.iter().rev() {
            self.bullets.remove(i);
        }
        
        // Remove bullets that are too far away
        self.bullets.retain(|b| {
            let dist = (b.pos - self.player.pos).length();
            dist < 2000.0
        });
        
        // Update particles
        self.particles.retain_mut(|p| p.update(dt));
        
        // Update mines
        self.mines.retain_mut(|m| m.update(dt));
        
        // Check collisions
        self.check_collisions();
        
        // Spawn new enemies with longer intervals
        self.enemy_spawn_timer -= dt;
        if self.enemy_spawn_timer <= 0.0 {
            self.wave += 1;  // Increment wave counter
            println!("Spawning wave {}", self.wave);
            self.spawn_wave();
            // Longer spawn intervals: 5-8 seconds based on wave
            self.enemy_spawn_timer = (8.0 - (self.wave as f32 * 0.3)).max(5.0);
        }
    }

    pub fn draw(&self, _renderer: &mut Renderer) {
        // This method can remain empty as we're handling drawing in main.rs now
    }


    fn spawn_wave(&mut self) {
        let mut rng = thread_rng();
        
        // Wave composition changes based on wave number
        let wave_templates = [
            // Wave 1-2: Basic enemies
            vec![(EnemyType::Cube, 5), (EnemyType::Pyramid, 2)],
            // Wave 3-4: Add swarms and spinners
            vec![(EnemyType::Cube, 3), (EnemyType::Pyramid, 2), (EnemyType::Spinner, 2), (EnemyType::Swarm, 8)],
            // Wave 5-6: Add support enemies
            vec![(EnemyType::Pyramid, 2), (EnemyType::Spinner, 2), (EnemyType::Shield, 1), (EnemyType::Disruptor, 1), (EnemyType::Swarm, 6)],
            // Wave 7-8: Add dangerous enemies
            vec![(EnemyType::Hunter, 2), (EnemyType::Laser, 1), (EnemyType::Bomber, 2), (EnemyType::Phaser, 1), (EnemyType::Shield, 1)],
            // Wave 9-10: Add advanced enemies
            vec![(EnemyType::Guardian, 2), (EnemyType::Reflector, 2), (EnemyType::Vortex, 1), (EnemyType::Bomber, 1), (EnemyType::Hunter, 1)],
            // Wave 11+: Ultimate challenge
            vec![(EnemyType::Carrier, 1), (EnemyType::Laser, 2), (EnemyType::Phaser, 2), (EnemyType::Vortex, 1), (EnemyType::Shield, 2), (EnemyType::Reflector, 1)],
        ];
        
        let template_index = ((self.wave - 1) / 2).min(5) as usize;
        let template = &wave_templates[template_index];
        
        println!("Wave {} using template index {} with enemies:", self.wave, template_index);
        for (enemy_type, count) in template {
            println!("  - {:?} x{}", enemy_type, count);
        }
        
        // Spawn different formations based on wave
        match self.wave % 4 {
            1 => self.spawn_arc_formation(template, &mut rng),
            2 => self.spawn_pincer_formation(template, &mut rng),
            3 => self.spawn_surround_formation(template, &mut rng),
            _ => self.spawn_ambush_formation(template, &mut rng),
        }
    }
    
    fn spawn_arc_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn in an arc in front of the player
        let mut spawn_count = 0;
        for &(enemy_type, count) in template {
            for i in 0..count {
                let total_enemies: usize = template.iter().map(|(_, c)| c).sum();
                let angle_offset = ((spawn_count + i) as f32 / total_enemies as f32 - 0.5) * PI * 0.8;
                // Spawn much farther ahead: 2000-4000 units
                let distance = rng.gen_range(2000.0..4000.0);
                let spawn_angle = self.player.rotation + angle_offset;
                
                let x = self.player.pos.x - spawn_angle.sin() * distance;
                let z = self.player.pos.z - spawn_angle.cos() * distance;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
            spawn_count += count;
        }
    }
    
    fn spawn_pincer_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn on both sides of the player
        for &(enemy_type, count) in template {
            for i in 0..count {
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                // Spawn farther ahead and to the sides
                let forward_offset = rng.gen_range(1500.0..2500.0);
                let side_offset = rng.gen_range(800.0..1500.0) * side;
                
                let spawn_angle = self.player.rotation;
                let x = self.player.pos.x - spawn_angle.sin() * forward_offset + spawn_angle.cos() * side_offset;
                let z = self.player.pos.z - spawn_angle.cos() * forward_offset - spawn_angle.sin() * side_offset;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
        }
    }
    
    fn spawn_surround_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn in a circle around the player
        let mut spawn_count = 0;
        let total_enemies: usize = template.iter().map(|(_, c)| c).sum();
        
        for &(enemy_type, count) in template {
            for i in 0..count {
                let angle = ((spawn_count + i) as f32 / total_enemies as f32) * PI * 2.0;
                // Larger surrounding circle
                let distance = rng.gen_range(1500.0..2500.0);
                
                let x = self.player.pos.x + angle.cos() * distance;
                let z = self.player.pos.z + angle.sin() * distance;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
            spawn_count += count;
        }
    }
    
    fn spawn_ambush_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn from behind and above
        for &(enemy_type, count) in template {
            for i in 0..count {
                let angle_offset = rng.gen_range(-PI/4.0..PI/4.0);
                let spawn_angle = self.player.rotation + PI + angle_offset; // Behind player
                // Even ambushes spawn farther away
                let distance = rng.gen_range(1000.0..1800.0);
                
                let x = self.player.pos.x - spawn_angle.sin() * distance;
                let z = self.player.pos.z - spawn_angle.cos() * distance;
                
                // Force some enemies to spawn higher for vertical ambush
                let mut enemy = Enemy::new(x, z, enemy_type);
                if i % 2 == 0 {
                    enemy.pos.y += rng.gen_range(100.0..200.0);
                }
                
                self.enemies.push(enemy);
            }
        }
    }

    fn check_collisions(&mut self) {
        let mut bullets_to_remove = vec![];
        let mut enemies_to_remove = vec![];
        let mut reflected_bullets = vec![];
        
        // Check bullet-enemy collisions (player bullets only)
        for (bi, bullet) in self.bullets.iter().enumerate() {
            match bullet.bullet_type {
                BulletType::Player => {
                    for (ei, enemy) in self.enemies.iter().enumerate() {
                        let dist = (bullet.pos - enemy.pos).length();
                        if dist < 30.0 {
                            bullets_to_remove.push(bi);
                            enemies_to_remove.push(ei);
                            
                            // Create explosion particles
                            for _ in 0..15 {
                                self.particles.push(Particle::new(enemy.pos));
                            }
                            
                            self.score += match enemy.enemy_type {
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
                            };
                        }
                    }
                }
                BulletType::Enemy => {
                    // Check collision with player
                    let dist = (bullet.pos - self.player.pos).length();
                    if dist < 25.0 && self.player_invulnerable_timer <= 0.0 {
                        bullets_to_remove.push(bi);
                        
                        // Create hit effect
                        for _ in 0..10 {
                            self.particles.push(Particle::new(self.player.pos));
                        }
                        
                        self.player_invulnerable_timer = 1.0;
                    }
                    
                    // Check reflection by reflector enemies
                    for (_ei, enemy) in self.enemies.iter().enumerate() {
                        if enemy.is_reflecting() {
                            let dist = (bullet.pos - enemy.pos).length();
                            if dist < 60.0 {
                                // Reflect bullet back towards player
                                bullets_to_remove.push(bi);
                                let reflect_dir = (self.player.pos - enemy.pos).normalize_or_zero();
                                if reflect_dir.length() > 0.0 {
                                    reflected_bullets.push((enemy.clone(), reflect_dir));
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Check player-enemy collisions
        for (ei, enemy) in self.enemies.iter().enumerate() {
            let dist = (self.player.pos - enemy.pos).length();
            if dist < 40.0 && self.player_invulnerable_timer <= 0.0 {
                enemies_to_remove.push(ei);
                for _ in 0..20 {
                    self.particles.push(Particle::new(enemy.pos));
                }
                self.player_invulnerable_timer = 1.0; // 1 second of invulnerability
            }
            
            // Check laser collision with player
            if enemy.is_laser_active() && self.player_invulnerable_timer <= 0.0 {
                let laser_end = enemy.get_laser_end_point();
                
                // Check if player intersects with laser line
                // Using point-to-line distance calculation
                let laser_vec = laser_end - enemy.pos;
                let to_player = self.player.pos - enemy.pos;
                
                // Project player position onto laser line
                let laser_length_sq = laser_vec.length_squared();
                if laser_length_sq > 0.0 {
                    let t = (to_player.dot(laser_vec) / laser_length_sq).clamp(0.0, 1.0);
                    let closest_point = enemy.pos + laser_vec * t;
                    let dist_to_laser = (self.player.pos - closest_point).length();
                    
                    // Laser has width of about 30 units
                    if dist_to_laser < 30.0 {
                        // Player hit by laser
                        for _ in 0..15 {
                            self.particles.push(Particle::new(self.player.pos));
                        }
                        self.player_invulnerable_timer = 2.0; // Longer invulnerability for laser hits
                    }
                }
            }
        }
        
        // Check mine collisions
        let mut mines_to_remove = vec![];
        for (mi, mine) in self.mines.iter().enumerate() {
            if mine.armed {
                let dist = (mine.pos - self.player.pos).length();
                if dist < mine.get_explosion_radius() {
                    mines_to_remove.push(mi);
                    
                    // Create explosion
                    for _ in 0..25 {
                        self.particles.push(Particle::new(mine.pos));
                    }
                    
                    if self.player_invulnerable_timer <= 0.0 {
                        self.player_invulnerable_timer = 1.5;
                    }
                }
            }
        }
        
        // Apply shield effects - make shielded enemies invulnerable
        let mut shielded_enemies = vec![false; self.enemies.len()];
        for (i, enemy) in self.enemies.iter().enumerate() {
            if enemy.get_shield_active() {
                let shield_radius = enemy.get_shield_radius();
                for (j, other) in self.enemies.iter().enumerate() {
                    if i != j {
                        let dist = (enemy.pos - other.pos).length();
                        if dist < shield_radius {
                            shielded_enemies[j] = true;
                        }
                    }
                }
            }
        }
        
        // Remove only non-shielded enemies
        enemies_to_remove.retain(|&ei| !shielded_enemies[ei]);
        
        // Remove collided entities
        bullets_to_remove.sort_unstable();
        bullets_to_remove.dedup();
        for &i in bullets_to_remove.iter().rev() {
            self.bullets.remove(i);
        }
        
        enemies_to_remove.sort_unstable();
        enemies_to_remove.dedup();
        for &i in enemies_to_remove.iter().rev() {
            self.enemies.remove(i);
        }
        
        mines_to_remove.sort_unstable();
        mines_to_remove.dedup();
        for &i in mines_to_remove.iter().rev() {
            self.mines.remove(i);
        }
        
        // Add reflected bullets
        for (enemy, dir) in reflected_bullets {
            self.bullets.push(Bullet::new_enemy(&enemy, dir));
        }
    }
}