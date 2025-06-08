use glam::Vec3;
use rand::prelude::*;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub enum EnemyType {
    Cube,      // Basic enemy that follows terrain
    Pyramid,   // Fast enemy that dives and climbs
    Spinner,   // Orbiting enemy with complex patterns
    Hunter,    // Tracks player position
    Guardian,  // Patrols specific areas
}

#[derive(Clone, Copy)]
pub enum MovementPattern {
    Hover,      // Maintains altitude above terrain
    Sinusoidal, // Wave-like movement
    Orbital,    // Circles around a point
    Diving,     // Swoops down and up
    Tracking,   // Follows player
    Patrol,     // Moves between waypoints
}

#[derive(Clone)]
pub struct Enemy {
    pub pos: Vec3,
    pub vel: Vec3,
    pub enemy_type: EnemyType,
    rotation: Vec3,
    rotation_speed: Vec3,
    movement_pattern: MovementPattern,
    spawn_point: Vec3,      // Remember where we spawned
    target_point: Vec3,     // Where we're moving to
    phase: f32,             // Animation phase for patterns
    speed: f32,             // Base movement speed
    aggression: f32,        // How aggressive this enemy is (0-1)
    detection_range: f32,   // How far it can detect player
    attack_cooldown: f32,   // Time until next attack
    can_attack: bool,       // Whether this enemy type can attack
}

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

impl Enemy {
    pub fn new(x: f32, z: f32, enemy_type: EnemyType) -> Self {
        let mut rng = thread_rng();
        
        // Calculate spawn height based on terrain
        let terrain_height = terrain_height_at(x, z);
        let spawn_height = terrain_height + rng.gen_range(30.0..100.0);
        
        // Assign movement pattern based on enemy type
        let movement_pattern = match enemy_type {
            EnemyType::Cube => MovementPattern::Hover,
            EnemyType::Pyramid => MovementPattern::Diving,
            EnemyType::Spinner => MovementPattern::Orbital,
            EnemyType::Hunter => MovementPattern::Tracking,
            EnemyType::Guardian => MovementPattern::Patrol,
        };
        
        // Set speed based on enemy type
        let speed = match enemy_type {
            EnemyType::Cube => rng.gen_range(80.0..120.0),
            EnemyType::Pyramid => rng.gen_range(150.0..200.0),
            EnemyType::Spinner => rng.gen_range(100.0..140.0),
            EnemyType::Hunter => rng.gen_range(120.0..160.0),
            EnemyType::Guardian => rng.gen_range(60.0..90.0),
        };
        
        // Set aggression and detection range
        let (aggression, detection_range) = match enemy_type {
            EnemyType::Cube => (0.3, 500.0),
            EnemyType::Pyramid => (0.7, 800.0),
            EnemyType::Spinner => (0.5, 600.0),
            EnemyType::Hunter => (0.9, 1200.0),
            EnemyType::Guardian => (0.4, 400.0),
        };
        
        let spawn_point = Vec3::new(x, spawn_height, z);
        
        Self {
            pos: spawn_point,
            vel: Vec3::ZERO,
            enemy_type,
            rotation: Vec3::ZERO,
            rotation_speed: Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            ),
            movement_pattern,
            spawn_point,
            target_point: spawn_point,
            phase: rng.gen_range(0.0..PI * 2.0),
            speed,
            aggression: aggression + rng.gen_range(-0.1..0.1),
            detection_range,
            attack_cooldown: 0.0,
            can_attack: matches!(enemy_type, EnemyType::Pyramid | EnemyType::Hunter | EnemyType::Guardian),
        }
    }
    
    pub fn new_with_pattern(x: f32, z: f32, enemy_type: EnemyType, pattern: MovementPattern) -> Self {
        let mut enemy = Self::new(x, z, enemy_type);
        enemy.movement_pattern = pattern;
        enemy
    }

    pub fn update(&mut self, dt: f32) {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
    }
    
    pub fn update_with_player(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        // Update rotation
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Update attack cooldown
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        
        // Calculate distance to player
        let to_player = player_pos - self.pos;
        let distance_to_player = to_player.length();
        
        // Update based on movement pattern
        match self.movement_pattern {
            MovementPattern::Hover => {
                // Maintain altitude above terrain
                let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                let hover_height = terrain_height + 50.0 + (self.phase * 0.5).sin() * 15.0;
                
                // Smooth altitude adjustment
                let height_diff = hover_height - self.pos.y;
                self.vel.y = height_diff * 2.0;
                
                // Gentle wandering
                self.vel.x = (self.phase * 0.3).sin() * self.speed * 0.5;
                self.vel.z = (self.phase * 0.25 + 1.0).cos() * self.speed * 0.5;
            },
            
            MovementPattern::Sinusoidal => {
                // Wave-like movement pattern
                let forward_speed = self.speed;
                let wave_amplitude = 30.0;
                let wave_frequency = 2.0;
                
                self.vel.x = (self.rotation.y).sin() * forward_speed;
                self.vel.z = (self.rotation.y).cos() * forward_speed;
                self.vel.y = (self.phase * wave_frequency).sin() * wave_amplitude;
            },
            
            MovementPattern::Orbital => {
                // Circle around spawn point
                let orbit_radius = 150.0;
                let orbit_speed = self.speed / orbit_radius;
                
                let angle = self.phase * orbit_speed;
                let target_x = self.spawn_point.x + angle.cos() * orbit_radius;
                let target_z = self.spawn_point.z + angle.sin() * orbit_radius;
                let target_y = terrain_height_at(target_x, target_z) + 60.0 + (self.phase * 2.0).sin() * 20.0;
                
                let target = Vec3::new(target_x, target_y, target_z);
                let to_target = target - self.pos;
                self.vel = to_target.normalize_or_zero() * self.speed;
            },
            
            MovementPattern::Diving => {
                // Swooping attack pattern
                let dive_cycle = self.phase % 4.0;
                
                if dive_cycle < 2.0 {
                    // Climbing phase
                    let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                    let climb_target = terrain_height + 150.0;
                    self.vel.y = (climb_target - self.pos.y).clamp(-self.speed, self.speed);
                    
                    // Move towards player horizontally during climb
                    if distance_to_player < self.detection_range {
                        let horizontal = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
                        self.vel.x = horizontal.x * self.speed * 0.5;
                        self.vel.z = horizontal.z * self.speed * 0.5;
                    }
                } else {
                    // Diving phase
                    if distance_to_player < self.detection_range {
                        // Dive towards player
                        let dive_target = player_pos + Vec3::new(0.0, -20.0, 0.0);
                        let to_target = dive_target - self.pos;
                        self.vel = to_target.normalize_or_zero() * self.speed * 1.5;
                    } else {
                        // Random dive
                        self.vel.y = -self.speed;
                    }
                }
            },
            
            MovementPattern::Tracking => {
                // Hunt the player
                if distance_to_player < self.detection_range {
                    // Lead the target
                    let lead_time = distance_to_player / self.speed * 0.5;
                    let predicted_pos = player_pos; // Could add player velocity prediction here
                    
                    let to_target = predicted_pos - self.pos;
                    self.vel = to_target.normalize_or_zero() * self.speed * self.aggression;
                    
                    // Maintain some altitude
                    let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                    if self.pos.y < terrain_height + 30.0 {
                        self.vel.y = (self.vel.y + 50.0).max(0.0);
                    }
                } else {
                    // Search pattern when player not detected
                    let search_radius = 300.0;
                    let angle = self.phase * 0.3;
                    self.target_point = self.spawn_point + Vec3::new(
                        angle.cos() * search_radius,
                        0.0,
                        angle.sin() * search_radius
                    );
                    
                    let to_target = self.target_point - self.pos;
                    self.vel = to_target.normalize_or_zero() * self.speed * 0.5;
                }
            },
            
            MovementPattern::Patrol => {
                // Move between patrol points
                let patrol_radius = 400.0;
                let num_points = 4;
                let current_point = ((self.phase * 0.1) as i32 % num_points) as f32;
                let angle = current_point * (2.0 * PI / num_points as f32);
                
                self.target_point = self.spawn_point + Vec3::new(
                    angle.cos() * patrol_radius,
                    0.0,
                    angle.sin() * patrol_radius
                );
                
                let to_target = self.target_point - self.pos;
                if to_target.length() > 50.0 {
                    self.vel = to_target.normalize_or_zero() * self.speed;
                    
                    // Maintain altitude
                    let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                    let patrol_height = terrain_height + 60.0;
                    self.vel.y = (patrol_height - self.pos.y).clamp(-self.speed * 0.5, self.speed * 0.5);
                }
                
                // React to player if nearby
                if distance_to_player < self.detection_range * 0.5 {
                    let reaction = to_player.normalize_or_zero() * self.speed * self.aggression * 0.5;
                    self.vel += reaction;
                }
            },
        }
        
        // Update position
        self.pos += self.vel * dt;
        
        // Ensure enemies don't go below terrain
        let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
        if self.pos.y < terrain_height + 10.0 {
            self.pos.y = terrain_height + 10.0;
            self.vel.y = self.vel.y.max(0.0);
        }
        
        // Check if we should attack
        if self.can_attack && self.attack_cooldown <= 0.0 && distance_to_player < self.detection_range {
            let attack_chance = match self.enemy_type {
                EnemyType::Pyramid => 0.02,    // Fast shooters
                EnemyType::Hunter => 0.03,     // Aggressive shooters
                EnemyType::Guardian => 0.015,  // Defensive shooters
                _ => 0.0,
            };
            
            if rand::random::<f32>() < attack_chance * self.aggression {
                // Set cooldown based on enemy type
                self.attack_cooldown = match self.enemy_type {
                    EnemyType::Pyramid => 1.5,
                    EnemyType::Hunter => 1.0,
                    EnemyType::Guardian => 2.0,
                    _ => 2.0,
                };
                
                // Return bullet spawn direction towards player
                return Some(to_player.normalize_or_zero());
            }
        }
        
        None
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
            EnemyType::Hunter => {
                // Draw a menacing diamond shape
                renderer.draw_pyramid(self.pos + Vec3::new(0.0, 20.0, 0.0), 30.0, rotation);
                let rotation2 = rotation_matrix(self.rotation.y, self.rotation.x + PI, self.rotation.z);
                renderer.draw_pyramid(self.pos - Vec3::new(0.0, 20.0, 0.0), 30.0, rotation2);
            }
            EnemyType::Guardian => {
                // Draw a fortress-like shape
                renderer.draw_cube(self.pos, 50.0, rotation);
                let rotation2 = rotation_matrix(self.rotation.y + PI/4.0, 0.0, 0.0);
                renderer.draw_cube(self.pos, 35.0, rotation2);
            }
        }
    }
}