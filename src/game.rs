use rand::prelude::*;
use std::f32::consts::PI;

use crate::player::Player;
use crate::enemy::{Enemy, EnemyType};
use crate::mine::Mine;

// Use the same terrain height calculation as the Terrain module for consistency
fn terrain_height_at(x: f32, z: f32) -> f32 {
    crate::terrain::Terrain::height_at(x, z)
}
use crate::bullet::{Bullet, BulletType};
use crate::particle::Particle;
use crate::renderer::Renderer;
use crate::base::{Base, BaseType};
use glam::Vec3;

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub mines: Vec<Mine>,
    pub bases: Vec<Base>,
    pub enemy_spawn_timer: f32,
    pub shoot_cooldown: f32,
    pub wave: u32,
    pub score: u32,
    pub player_invulnerable_timer: f32,  // Brief invulnerability after being hit
    pub player_slowed: bool,  // Disruptor effect
    pub player_slow_timer: f32,
    pub position_log_timer: f32,  // Timer for logging player position
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            player: Player::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            mines: Vec::new(),
            bases: Vec::new(),
            enemy_spawn_timer: 3.0,  // Start with 3 second delay before first wave
            shoot_cooldown: 0.0,
            wave: 1,
            score: 0,
            player_invulnerable_timer: 0.0,
            player_slowed: false,
            player_slow_timer: 0.0,
            position_log_timer: 0.0,
        };
        
        // Generate initial bases around the map
        game.generate_bases();
        
        game
    }

    pub fn update(&mut self, left: bool, right: bool, forward: bool, backward: bool, shoot: bool, boost: bool, up: bool, down: bool, dt: f32, sound_system: &mut crate::sounds::SoundSystem) {
        // Update player (slow effect would need to be implemented in player.rs)
        self.player.update(left, right, forward, backward, boost, up, down, dt);
        
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
        
        // Log player position every second
        self.position_log_timer += dt;
        if self.position_log_timer >= 1.0 {
            println!("Player position: ({:.1}, {:.1}, {:.1})", 
                self.player.pos.x, self.player.pos.y, self.player.pos.z);
            self.position_log_timer = 0.0;
        }
        
        // Handle shooting
        if shoot && self.shoot_cooldown <= 0.0 {
            self.bullets.push(Bullet::new(&self.player));
            self.shoot_cooldown = 0.15;
            
            // Alert nearby enemies to gunfire
            self.alert_enemies_to_sound(self.player.pos, 1500.0); // Gunshots are VERY loud on huge map
        }
        self.shoot_cooldown -= dt;
        
        // Update bases first
        let mut base_bullets = Vec::new();
        let mut bases_to_spawn = Vec::new();
        
        for (i, base) in self.bases.iter_mut().enumerate() {
            let turret_shots = base.update(self.player.pos, dt);
            base_bullets.extend(turret_shots);
            
            // Check if base should spawn enemies
            if base.should_spawn() {
                bases_to_spawn.push(i);
                base.reset_spawn_timer();
            }
        }
        
        // Spawn enemies from bases that are ready
        for base_index in bases_to_spawn {
            let base_pos = self.bases[base_index].pos;
            let enemy_types = self.bases[base_index].get_spawn_types();
            let routes = self.bases[base_index].get_patrol_routes();
            
            self.spawn_enemies_from_base_data(base_pos, enemy_types, routes);
        }
        
        // Create bullets from base turrets
        for (pos, dir, is_heavy) in base_bullets {
            let bullet_type = if is_heavy { BulletType::HeavyTurret } else { BulletType::Enemy };
            let mut bullet = Bullet::new_at_position(pos, dir, bullet_type);
            bullet.vel = dir * if is_heavy { 450.0 } else { 350.0 }; // Slightly slower for visibility
            self.bullets.push(bullet);
            // Play turret sound (deeper for heavy turrets) with distance
            let distance = (pos - self.player.pos).length();
            sound_system.play_enemy_laser(if is_heavy { "Guardian" } else { "Vortex" }, Some(distance));
        }
        
        // Update enemies with player awareness and handle attacks
        let mut enemy_bullets = Vec::new();
        let mut new_mines = Vec::new();
        let mut new_swarms = Vec::new();
        let mut alerted_enemies = Vec::new();
        
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            if let Some(attack_dir) = enemy.update_with_player(self.player.pos, dt) {
                enemy_bullets.push(Bullet::new_enemy(enemy, attack_dir));
                // Play enemy laser sound with position
                let distance = (enemy.pos - self.player.pos).length();
                sound_system.play_enemy_laser(&format!("{:?}", enemy.enemy_type), Some(distance));
            }
            
            // Track which enemies just became alerted
            if enemy.alert_state == crate::enemy::AlertState::Alert && enemy.alert_cooldown >= 4.9 {
                alerted_enemies.push(i);
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
        
        // Alert propagation - enemies that spotted the player alert nearby allies
        for alert_idx in alerted_enemies {
            // Get a raw pointer to the enemies slice to avoid borrowing issues
            let enemies_ptr = self.enemies.as_mut_ptr();
            let enemies_len = self.enemies.len();
            
            unsafe {
                let alerting_enemy = &(*enemies_ptr.add(alert_idx));
                let enemies_slice = std::slice::from_raw_parts_mut(enemies_ptr, enemies_len);
                alerting_enemy.alert_nearby_enemies(enemies_slice);
            }
        }
        
        // Remove enemies that are too far away or behind
        let player_z = self.player.pos.z;
        self.enemies.retain(|e| {
            let distance = (e.pos - self.player.pos).length();
            // Keep enemies within 10000 units to allow longer pursuits
            distance < 10000.0 && (e.pos.z < player_z || e.pos.z > player_z - 4000.0)
        });
        
        // Update bullets and check terrain collisions
        let mut terrain_hits = Vec::new();
        for (i, bullet) in self.bullets.iter_mut().enumerate() {
            bullet.update(dt);
            
            // Check if bullet hit terrain
            let terrain_height = terrain_height_at(bullet.pos.x, bullet.pos.z);
            if bullet.pos.y < terrain_height {
                terrain_hits.push((i, bullet.pos));
                // Debug log when a player bullet hits terrain
                if matches!(bullet.bullet_type, BulletType::Player) {
                    println!("Player bullet hit terrain at ({:.1}, {:.1}, {:.1}), terrain height: {:.1}", 
                        bullet.pos.x, bullet.pos.y, bullet.pos.z, terrain_height);
                }
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
        
        // Wave progression - add more bases as waves increase
        self.enemy_spawn_timer -= dt;
        if self.enemy_spawn_timer <= 0.0 {
            self.wave += 1;  // Increment wave counter
            println!("Wave {} - Adding new bases", self.wave);
            self.add_wave_bases();
            // Spawn intervals: 30-60 seconds between new bases
            self.enemy_spawn_timer = (40.0 - (self.wave as f32 * 2.0)).max(20.0); // Much faster base spawning
        }
    }

    pub fn draw(&self, _renderer: &mut Renderer) {
        // This method can remain empty as we're handling drawing in main.rs now
    }


    // Legacy spawn_wave function - no longer used, enemies spawn from bases now
    /*
    fn spawn_wave(&mut self) {
        // This function is deprecated - enemies now spawn from bases
    }
    
    fn spawn_arc_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn in an arc in front of the player
        let mut spawn_count = 0;
        for &(enemy_type, count) in template {
            for i in 0..count {
                let total_enemies: usize = template.iter().map(|(_, c)| c).sum();
                let angle_offset = ((spawn_count + i) as f32 / total_enemies as f32 - 0.5) * PI * 0.8;
                // Spawn at reasonable distance: 300-600 units
                let distance = rng.gen_range(300.0..600.0);
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
                // Spawn at reasonable distances
                let forward_offset = rng.gen_range(200.0..400.0);
                let side_offset = rng.gen_range(150.0..300.0) * side;
                
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
                // Surrounding circle at reasonable distance
                let distance = rng.gen_range(400.0..600.0);
                
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
                // Ambushes spawn behind at reasonable distance
                let distance = rng.gen_range(200.0..400.0);
                
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
    */

    fn generate_bases(&mut self) {
        let mut rng = thread_rng();
        
        // Map is approximately 51,360 x 51,360 units (terrain scale from view_distance=160)
        // Let's spread bases across the entire map
        const MAP_SIZE: f32 = 50000.0; // Slightly less than full size to avoid edges
        
        // Create a grid of bases across the map
        let base_configs = [
            // Central fortress
            (BaseType::Fortress, Vec3::new(0.0, 0.0, 0.0)),
            
            // Four large bases at cardinal directions
            (BaseType::Large, Vec3::new(MAP_SIZE * 0.7, 0.0, 0.0)),
            (BaseType::Large, Vec3::new(-MAP_SIZE * 0.7, 0.0, 0.0)),
            (BaseType::Large, Vec3::new(0.0, 0.0, MAP_SIZE * 0.7)),
            (BaseType::Large, Vec3::new(0.0, 0.0, -MAP_SIZE * 0.7)),
            
            // Medium bases at diagonals
            (BaseType::Medium, Vec3::new(MAP_SIZE * 0.5, 0.0, MAP_SIZE * 0.5)),
            (BaseType::Medium, Vec3::new(-MAP_SIZE * 0.5, 0.0, MAP_SIZE * 0.5)),
            (BaseType::Medium, Vec3::new(MAP_SIZE * 0.5, 0.0, -MAP_SIZE * 0.5)),
            (BaseType::Medium, Vec3::new(-MAP_SIZE * 0.5, 0.0, -MAP_SIZE * 0.5)),
            
            // Small bases scattered around
            (BaseType::Small, Vec3::new(MAP_SIZE * 0.3, 0.0, MAP_SIZE * 0.2)),
            (BaseType::Small, Vec3::new(-MAP_SIZE * 0.3, 0.0, MAP_SIZE * 0.2)),
            (BaseType::Small, Vec3::new(MAP_SIZE * 0.2, 0.0, -MAP_SIZE * 0.3)),
            (BaseType::Small, Vec3::new(-MAP_SIZE * 0.2, 0.0, -MAP_SIZE * 0.3)),
            (BaseType::Small, Vec3::new(MAP_SIZE * 0.4, 0.0, -MAP_SIZE * 0.2)),
            (BaseType::Small, Vec3::new(-MAP_SIZE * 0.4, 0.0, -MAP_SIZE * 0.1)),
            (BaseType::Small, Vec3::new(MAP_SIZE * 0.1, 0.0, MAP_SIZE * 0.4)),
            (BaseType::Small, Vec3::new(-MAP_SIZE * 0.1, 0.0, MAP_SIZE * 0.4)),
        ];
        
        for (base_type, offset) in &base_configs {
            // Add some randomness to positions (but not too much for strategic placement)
            let variation = match base_type {
                BaseType::Fortress => 0.0, // Fortress always at center
                BaseType::Large => 2000.0,
                BaseType::Medium => 3000.0,
                BaseType::Small => 4000.0,
            };
            
            let x = offset.x + if variation > 0.0 { rng.gen_range(-variation..variation) } else { 0.0 };
            let z = offset.z + if variation > 0.0 { rng.gen_range(-variation..variation) } else { 0.0 };
            
            // Ensure bases aren't too close to map edges
            let x = x.clamp(-MAP_SIZE * 0.9, MAP_SIZE * 0.9);
            let z = z.clamp(-MAP_SIZE * 0.9, MAP_SIZE * 0.9);
            
            self.bases.push(Base::new(x, z, *base_type));
        }
        
        println!("Generated {} initial bases across {}x{} unit map", self.bases.len(), MAP_SIZE * 2.0, MAP_SIZE * 2.0);
    }
    
    fn add_wave_bases(&mut self) {
        let mut rng = thread_rng();
        
        const MAP_SIZE: f32 = 50000.0;
        
        // Add bases based on wave number
        let new_base_count = ((self.wave - 1) / 3 + 1).min(3) as usize;
        
        for _ in 0..new_base_count {
            // Generate bases at random locations across the map
            // Higher waves spawn bases further from center
            let min_distance = 5000.0 + (self.wave as f32 * 2000.0);
            let max_distance = min_distance + 15000.0;
            
            // Ensure we don't exceed map bounds and always have a valid range
            let clamped_min = min_distance.min(MAP_SIZE * 0.7);
            let clamped_max = max_distance.min(MAP_SIZE * 0.8);
            
            // If min exceeds max, just use a fixed distance near the max
            let distance = if clamped_min >= clamped_max {
                clamped_max * 0.9
            } else {
                rng.gen_range(clamped_min..clamped_max)
            };
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let x = angle.cos() * distance;
            let z = angle.sin() * distance;
            
            let base_type = match self.wave {
                1..=3 => BaseType::Small,
                4..=6 => BaseType::Medium,
                7..=9 => BaseType::Large,
                _ => if rng.gen_bool(0.3) { BaseType::Fortress } else { BaseType::Large },
            };
            
            self.bases.push(Base::new(x, z, base_type));
            println!("Added new {} base at ({:.0}, {:.0})", 
                match base_type {
                    BaseType::Small => "Small",
                    BaseType::Medium => "Medium", 
                    BaseType::Large => "Large",
                    BaseType::Fortress => "Fortress",
                }, x, z);
        }
    }
    
    fn spawn_enemies_from_base_data(&mut self, base_pos: Vec3, enemy_types: Vec<EnemyType>, routes: Vec<Vec<Vec3>>) {
        let mut rng = thread_rng();
        
        // Spawn fewer enemies per cycle to reduce clustering
        let spawn_count = rng.gen_range(2..=4);
        
        for i in 0..spawn_count {
            if let Some(enemy_type) = enemy_types.choose(&mut rng) {
                // Spawn enemies with better spacing to prevent clustering
                // Use a larger base radius and vary it per enemy
                let base_radius = 200.0 + (i as f32 * 50.0);
                let angle = (i as f32 / spawn_count as f32) * std::f32::consts::PI * 2.0 + rng.gen_range(-0.2..0.2);
                
                // Vary the height more to prevent vertical clustering
                let height_offset = 50.0 + rng.gen_range(0.0..50.0);
                let spawn_offset = Vec3::new(
                    angle.cos() * base_radius, 
                    height_offset, 
                    angle.sin() * base_radius
                );
                let spawn_pos = base_pos + spawn_offset;
                
                let mut enemy = Enemy::new(spawn_pos.x, spawn_pos.z, *enemy_type);
                enemy.pos.y = spawn_pos.y;
                
                // Assign a patrol route
                if let Some(route) = routes.get(i % routes.len()) {
                    enemy.set_patrol_route(route.clone());
                }
                
                self.enemies.push(enemy);
            }
        }
    }
    
    fn check_collisions(&mut self) {
        let mut bullets_to_remove = vec![];
        let mut enemies_to_remove = vec![];
        let mut reflected_bullets = vec![];
        
        // Check bullet-base and ground turret collisions (player bullets only)
        let mut turret_hits = Vec::new(); // (base_idx, turret_idx, bullet_pos)
        
        for (bi, bullet) in self.bullets.iter().enumerate() {
            if matches!(bullet.bullet_type, BulletType::Player) {
                let mut hit_something = false;
                
                for (_base_idx, base) in self.bases.iter_mut().enumerate() {
                    if base.is_active {
                        // Check main base collision
                        let dist = (bullet.pos - base.pos).length();
                        if dist < 50.0 { // Base hit radius
                            bullets_to_remove.push(bi);
                            base.take_damage(25.0);
                            hit_something = true;
                            
                            // Create impact particles
                            for _ in 0..10 {
                                self.particles.push(Particle::new(bullet.pos));
                            }
                            
                            if !base.is_active {
                                // Base destroyed - big explosion
                                for _ in 0..50 {
                                    self.particles.push(Particle::new(base.pos));
                                }
                                self.score += 500; // Bonus for destroying base
                            }
                            break;
                        }
                    }
                    
                    if hit_something {
                        break;
                    }
                }
                
                // Check ground turret collisions separately to avoid double borrow
                if !hit_something {
                    for (base_idx, base) in self.bases.iter().enumerate() {
                        if base.is_active {
                            let turret_positions = base.get_ground_turret_positions();
                            for (turret_idx, turret_pos, is_active) in turret_positions {
                                if is_active {
                                    let dist = (bullet.pos - turret_pos).length();
                                    if dist < 20.0 { // Turret hit radius
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
        
        // Process turret hits
        for (base_idx, turret_idx, bullet_pos, turret_pos) in turret_hits {
            if self.bases[base_idx].damage_ground_turret(turret_idx, 25.0) {
                // Turret destroyed
                for _ in 0..20 {
                    self.particles.push(Particle::new(turret_pos));
                }
                self.score += 100; // Points for destroying turret
            } else {
                // Just damaged
                for _ in 0..5 {
                    self.particles.push(Particle::new(bullet_pos));
                }
            }
        }
        
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
                BulletType::Enemy | BulletType::HeavyTurret => {
                    // Check collision with player
                    let dist = (bullet.pos - self.player.pos).length();
                    if dist < 25.0 && self.player_invulnerable_timer <= 0.0 {
                        bullets_to_remove.push(bi);
                        
                        // Player takes damage - heavy turrets do more damage
                        let damage = match bullet.bullet_type {
                            BulletType::HeavyTurret => 0.5, // Double damage
                            _ => 0.25,
                        };
                        
                        if self.player.take_damage(damage) {
                            // Shield didn't absorb it - player is hit
                            self.player_invulnerable_timer = 1.0;
                            
                            // Create hit effect
                            for _ in 0..15 {
                                self.particles.push(Particle::new(self.player.pos));
                            }
                        } else {
                            // Shield absorbed the hit
                            for _ in 0..5 {
                                self.particles.push(Particle::new(self.player.pos));
                            }
                        }
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
                
                // Collision damage
                if self.player.take_damage(0.5) {
                    self.player_invulnerable_timer = 1.0;
                }
                
                for _ in 0..20 {
                    self.particles.push(Particle::new(enemy.pos));
                }
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
    
    fn alert_enemies_to_sound(&mut self, sound_pos: Vec3, sound_radius: f32) {
        for enemy in &mut self.enemies {
            let distance = (enemy.pos - sound_pos).length();
            
            // Check if enemy can hear the sound
            if distance <= sound_radius && distance <= enemy.hearing_range {
                match enemy.alert_state {
                    crate::enemy::AlertState::Unaware => {
                        // Become suspicious and investigate
                        enemy.alert_state = crate::enemy::AlertState::Suspicious;
                        enemy.investigation_point = sound_pos;
                        enemy.alert_cooldown = 3.0;
                    }
                    crate::enemy::AlertState::Suspicious => {
                        // Already suspicious - update investigation point
                        enemy.investigation_point = sound_pos;
                        enemy.alert_cooldown = 3.0;
                    }
                    _ => {} // Already alert or searching
                }
            }
        }
    }
}