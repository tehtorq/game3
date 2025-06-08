use glam::Vec3;
use rand::prelude::*;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyType {
    Cube,      // Basic enemy that follows terrain
    Pyramid,   // Fast enemy that dives and climbs
    Spinner,   // Orbiting enemy with complex patterns
    Hunter,    // Tracks player position
    Guardian,  // Patrols specific areas
    Laser,     // Enemy with sweeping laser attack
    Swarm,     // Small enemies that move in groups
    Phaser,    // Teleporting sniper
    Shield,    // Creates shields for other enemies
    Bomber,    // Drops explosive mines
    Disruptor, // Emits slowing waves
    Carrier,   // Spawns swarm enemies
    Reflector, // Reflects bullets back
    Vortex,    // Creates gravity well
}

#[derive(Clone, Copy)]
pub enum MovementPattern {
    Hover,      // Maintains altitude above terrain
    Sinusoidal, // Wave-like movement
    Orbital,    // Circles around a point
    Diving,     // Swoops down and up
    Tracking,   // Follows player
    Patrol,     // Moves between waypoints
    Flocking,   // Swarm movement
    Teleport,   // Phaser teleportation
    Stationary, // Shield generator/Vortex
    Drifting,   // Slow carrier movement
}

#[derive(Clone)]
pub struct Enemy {
    pub pos: Vec3,
    pub vel: Vec3,
    pub enemy_type: EnemyType,
    pub rotation: Vec3,
    pub rotation_speed: Vec3,
    pub movement_pattern: MovementPattern,
    pub spawn_point: Vec3,      // Remember where we spawned
    pub target_point: Vec3,     // Where we're moving to
    pub phase: f32,             // Animation phase for patterns
    speed: f32,             // Base movement speed
    aggression: f32,        // How aggressive this enemy is (0-1)
    detection_range: f32,   // How far it can detect player
    attack_cooldown: f32,   // Time until next attack
    can_attack: bool,       // Whether this enemy type can attack
    laser_active: bool,     // Whether laser is currently firing
    laser_angle_h: f32,     // Current horizontal angle of laser sweep
    laser_angle_v: f32,     // Current vertical angle of laser sweep
    laser_target_angle_h: f32,// Target horizontal angle for laser sweep
    laser_target_angle_v: f32,// Target vertical angle for laser sweep
    laser_duration: f32,    // How long the laser has been active
    // Special ability fields
    special_cooldown: f32,  // Cooldown for special abilities
    teleport_charge: f32,   // Phaser teleport charge time
    shield_radius: f32,     // Shield generator radius
    mine_count: i32,        // Bomber's remaining mines
    wave_charge: f32,       // Disruptor wave charge
    spawn_timer: f32,       // Carrier spawn timer
    reflection_active: bool,// Reflector state
    vortex_strength: f32,   // Current vortex pull strength
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
            EnemyType::Laser => MovementPattern::Hover,
            EnemyType::Swarm => MovementPattern::Flocking,
            EnemyType::Phaser => MovementPattern::Teleport,
            EnemyType::Shield => MovementPattern::Stationary,
            EnemyType::Bomber => MovementPattern::Drifting,
            EnemyType::Disruptor => MovementPattern::Hover,
            EnemyType::Carrier => MovementPattern::Drifting,
            EnemyType::Reflector => MovementPattern::Orbital,
            EnemyType::Vortex => MovementPattern::Stationary,
        };
        
        // Set speed based on enemy type
        let speed = match enemy_type {
            EnemyType::Cube => rng.gen_range(80.0..120.0),
            EnemyType::Pyramid => rng.gen_range(150.0..200.0),
            EnemyType::Spinner => rng.gen_range(100.0..140.0),
            EnemyType::Hunter => rng.gen_range(120.0..160.0),
            EnemyType::Guardian => rng.gen_range(60.0..90.0),
            EnemyType::Laser => rng.gen_range(40.0..60.0),
            EnemyType::Swarm => rng.gen_range(200.0..250.0),
            EnemyType::Phaser => 0.0, // Teleports instead, no speed needed
            EnemyType::Shield => rng.gen_range(30.0..40.0),
            EnemyType::Bomber => rng.gen_range(40.0..60.0),
            EnemyType::Disruptor => rng.gen_range(50.0..70.0),
            EnemyType::Carrier => rng.gen_range(20.0..30.0),
            EnemyType::Reflector => rng.gen_range(70.0..90.0),
            EnemyType::Vortex => 0.0, // Stationary, no speed needed
        };
        
        // Set aggression and detection range
        let (aggression, detection_range) = match enemy_type {
            EnemyType::Cube => (0.3, 500.0),
            EnemyType::Pyramid => (0.7, 800.0),
            EnemyType::Spinner => (0.5, 600.0),
            EnemyType::Hunter => (0.9, 1200.0),
            EnemyType::Guardian => (0.4, 400.0),
            EnemyType::Laser => (0.6, 1000.0),
            EnemyType::Swarm => (0.8, 400.0),
            EnemyType::Phaser => (0.7, 1500.0),
            EnemyType::Shield => (0.2, 600.0),
            EnemyType::Bomber => (0.4, 700.0),
            EnemyType::Disruptor => (0.5, 800.0),
            EnemyType::Carrier => (0.3, 1000.0),
            EnemyType::Reflector => (0.6, 600.0),
            EnemyType::Vortex => (0.0, 500.0),
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
            laser_active: false,
            laser_angle_h: 0.0,
            laser_angle_v: 0.0,
            laser_target_angle_h: 0.0,
            laser_target_angle_v: 0.0,
            laser_duration: 0.0,
            special_cooldown: 0.0,
            teleport_charge: 0.0,
            shield_radius: match enemy_type {
                EnemyType::Shield => 200.0,
                _ => 0.0,
            },
            mine_count: match enemy_type {
                EnemyType::Bomber => 5,
                _ => 0,
            },
            wave_charge: 0.0,
            spawn_timer: 0.0,
            reflection_active: matches!(enemy_type, EnemyType::Reflector),
            vortex_strength: match enemy_type {
                EnemyType::Vortex => 300.0,
                _ => 0.0,
            },
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
        // Wrap phase to prevent overflow
        if self.phase > 1000.0 {
            self.phase = self.phase % (2.0 * PI);
        }
        
        // Special handling for enemies with unique behaviors
        match self.enemy_type {
            EnemyType::Laser => return self.update_laser_enemy(player_pos, dt),
            EnemyType::Phaser => return self.update_phaser_enemy(player_pos, dt),
            EnemyType::Shield => return self.update_shield_enemy(player_pos, dt),
            EnemyType::Bomber => return self.update_bomber_enemy(player_pos, dt),
            EnemyType::Disruptor => return self.update_disruptor_enemy(player_pos, dt),
            EnemyType::Carrier => return self.update_carrier_enemy(player_pos, dt),
            EnemyType::Reflector => return self.update_reflector_enemy(player_pos, dt),
            EnemyType::Vortex => return self.update_vortex_enemy(player_pos, dt),
            _ => {}
        }
        // Update rotation with wrapping
        self.rotation.x = (self.rotation.x + self.rotation_speed.x * dt) % (2.0 * PI);
        self.rotation.y = (self.rotation.y + self.rotation_speed.y * dt) % (2.0 * PI);
        self.rotation.z = (self.rotation.z + self.rotation_speed.z * dt) % (2.0 * PI);
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
                    let _lead_time = distance_to_player / self.speed * 0.5;
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
            
            MovementPattern::Flocking => {
                // Swarm movement - fast and erratic
                let to_player_norm = to_player.normalize_or_zero();
                
                // Add some randomness for swarming effect
                let swarm_offset = Vec3::new(
                    (self.phase * 7.0).sin() * 50.0,
                    (self.phase * 5.0).cos() * 30.0,
                    (self.phase * 6.0).sin() * 50.0
                );
                
                // Aggressive pursuit with swarm behavior
                self.vel = (to_player_norm * self.speed + swarm_offset) * self.aggression;
                
                // Maintain low altitude for swarming
                let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                let swarm_height = terrain_height + 30.0 + (self.phase * 3.0).sin() * 10.0;
                self.vel.y = (swarm_height - self.pos.y) * 3.0;
            },
            
            MovementPattern::Teleport => {
                // Phaser movement - charge then teleport
                self.teleport_charge += dt;
                
                if self.teleport_charge > 2.0 {
                    // Teleport to new position
                    let angle = rand::random::<f32>() * PI * 2.0;
                    let distance = 500.0 + rand::random::<f32>() * 500.0;
                    
                    self.pos.x = player_pos.x + angle.cos() * distance;
                    self.pos.z = player_pos.z + angle.sin() * distance;
                    
                    // Set altitude
                    let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                    self.pos.y = terrain_height + 100.0;
                    
                    self.teleport_charge = 0.0;
                    self.special_cooldown = 1.0; // Charge attack after teleport
                }
                
                // No regular movement
                self.vel = Vec3::ZERO;
            },
            
            MovementPattern::Stationary => {
                // Shield generator/Vortex - stays in place
                self.vel = Vec3::ZERO;
                
                // Maintain altitude
                let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                let hover_height = terrain_height + 60.0;
                self.pos.y = self.pos.y * 0.9 + hover_height * 0.1;
            },
            
            MovementPattern::Drifting => {
                // Bomber/Carrier - slow drift
                let drift_angle = self.phase * 0.1;
                self.vel.x = drift_angle.cos() * self.speed;
                self.vel.z = drift_angle.sin() * self.speed;
                
                // Maintain high altitude
                let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
                let cruise_height = terrain_height + 150.0;
                self.vel.y = (cruise_height - self.pos.y).clamp(-20.0, 20.0);
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
        
        // Check for NaN after position update
        if self.pos.is_nan() {
            println!("ERROR: Enemy position became NaN! Type: {:?}, Vel: {:?}", self.enemy_type, self.vel);
            self.pos = self.spawn_point; // Reset to spawn
            self.vel = Vec3::ZERO;
        }
        
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
    
    // Special enemy update methods
    fn update_phaser_enemy(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Handle teleportation (already in movement pattern)
        
        // Charge and fire after teleport
        if self.special_cooldown > 0.0 {
            self.special_cooldown -= dt;
            if self.special_cooldown <= 0.0 && self.teleport_charge < 0.5 {
                // Fire precise shot at player
                let to_player = player_pos - self.pos;
                return Some(to_player.normalize_or_zero());
            }
        }
        
        None
    }
    
    fn update_shield_enemy(&mut self, _player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Shield is always active, handled in game logic
        None
    }
    
    fn update_bomber_enemy(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        if self.special_cooldown > 0.0 {
            self.special_cooldown -= dt;
        }
        
        // Drop mines periodically
        let distance_to_player = (player_pos - self.pos).length();
        if self.mine_count > 0 && self.special_cooldown <= 0.0 && distance_to_player < self.detection_range {
            self.mine_count -= 1;
            self.special_cooldown = 1.5;
            // Mine dropping handled in game logic
        }
        
        None
    }
    
    fn update_disruptor_enemy(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Charge wave attack
        let distance_to_player = (player_pos - self.pos).length();
        if distance_to_player < self.detection_range {
            self.wave_charge += dt;
            
            if self.wave_charge > 2.0 {
                self.wave_charge = 0.0;
                // Wave emission handled in game logic
            }
        }
        
        None
    }
    
    fn update_carrier_enemy(&mut self, _player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Spawn timer
        self.spawn_timer += dt;
        if self.spawn_timer > 4.0 {
            self.spawn_timer = 0.0;
            // Spawning handled in game logic
        }
        
        None
    }
    
    fn update_reflector_enemy(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Always face player for reflection
        let to_player = player_pos - self.pos;
        let angle_to_player = to_player.z.atan2(to_player.x);
        self.rotation.y = angle_to_player;
        
        None
    }
    
    fn update_vortex_enemy(&mut self, _player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Pulsing vortex strength
        self.vortex_strength = 300.0 + (self.phase * 2.0).sin() * 100.0;
        
        None
    }
    
    fn update_laser_enemy(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        // Update rotation and phase
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
        
        // Update attack cooldown
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        
        // Calculate distance to player
        let to_player = player_pos - self.pos;
        let distance_to_player = to_player.length();
        
        // Hover movement pattern with slow approach
        let terrain_height = terrain_height_at(self.pos.x, self.pos.z);
        let hover_height = terrain_height + 80.0 + (self.phase * 0.3).sin() * 20.0;
        
        // Smooth altitude adjustment
        let height_diff = hover_height - self.pos.y;
        self.vel.y = height_diff * 2.0;
        
        // Slowly move towards player if far away
        if distance_to_player > 800.0 {
            let horizontal = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
            self.vel.x = horizontal.x * self.speed * 0.5;
            self.vel.z = horizontal.z * self.speed * 0.5;
        } else {
            // Gentle circling when close
            let orbit_angle = self.phase * 0.2;
            self.vel.x = orbit_angle.cos() * self.speed * 0.3;
            self.vel.z = orbit_angle.sin() * self.speed * 0.3;
        }
        
        // Update position
        self.pos += self.vel * dt;
        
        // Ensure enemy doesn't go below terrain
        if self.pos.y < terrain_height + 10.0 {
            self.pos.y = terrain_height + 10.0;
            self.vel.y = self.vel.y.max(0.0);
        }
        
        // Handle laser attack
        if self.laser_active {
            // Update laser sweep
            self.laser_duration += dt;
            
            // Sweep towards target angles in both horizontal and vertical
            let angle_diff_h = self.laser_target_angle_h - self.laser_angle_h;
            let angle_diff_v = self.laser_target_angle_v - self.laser_angle_v;
            let sweep_speed = 0.8; // Radians per second - slow enough to dodge
            
            // Smooth interpolation towards target
            self.laser_angle_h += angle_diff_h.signum() * sweep_speed * dt;
            self.laser_angle_v += angle_diff_v.signum() * sweep_speed * dt;
            
            // Check if laser sweep is complete
            if self.laser_duration > 3.0 || (angle_diff_h.abs() < 0.1 && angle_diff_v.abs() < 0.1) {
                self.laser_active = false;
                self.attack_cooldown = 4.0; // Long cooldown between laser attacks
                println!("Laser deactivated after {} seconds", self.laser_duration);
            }
        } else if distance_to_player < self.detection_range && self.attack_cooldown <= 0.0 {
            // Start a new laser attack with higher chance
            if rand::random::<f32>() < 0.05 * self.aggression { // Increased from 0.02
                self.laser_active = true;
                self.laser_duration = 0.0;
                
                // Calculate 3D angles to player
                let horizontal_dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
                let angle_to_player_h = to_player.z.atan2(to_player.x);
                let angle_to_player_v = to_player.y.atan2(horizontal_dist);
                
                // Start laser pointing away from player, will sweep towards them
                // Add some randomness to starting position
                let start_offset_h = if rand::random::<bool>() { -1.2 } else { 1.2 };
                let start_offset_v = rand::random::<f32>() - 0.5;  // Range -0.5 to 0.5
                
                self.laser_angle_h = angle_to_player_h + start_offset_h;
                self.laser_angle_v = angle_to_player_v + start_offset_v;
                
                // Target slightly past the player for a sweeping motion
                self.laser_target_angle_h = angle_to_player_h + start_offset_h * -0.3;
                self.laser_target_angle_v = angle_to_player_v - start_offset_v * 0.5;
                
                println!("Laser activated! Distance: {:.1}, H-Angle: {:.2}, V-Angle: {:.2}", 
                         distance_to_player, self.laser_angle_h, self.laser_angle_v);
            }
        }
        
        None // Laser enemies don't shoot bullets
    }
    
    pub fn is_laser_active(&self) -> bool {
        self.laser_active
    }
    
    pub fn get_laser_end_point(&self) -> Vec3 {
        // Calculate laser end point based on current horizontal and vertical angles
        let laser_length = 1500.0;
        
        // Convert spherical coordinates to Cartesian
        let horizontal_component = self.laser_angle_v.cos() * laser_length;
        let vertical_component = self.laser_angle_v.sin() * laser_length;
        
        let end_x = self.pos.x + self.laser_angle_h.cos() * horizontal_component;
        let end_z = self.pos.z + self.laser_angle_h.sin() * horizontal_component;
        let end_y = self.pos.y + vertical_component;
        
        Vec3::new(end_x, end_y, end_z)
    }
    
    pub fn get_laser_info(&self) -> Option<(Vec3, Vec3)> {
        if self.laser_active {
            let start = self.pos - Vec3::new(0.0, 20.0, 0.0);
            let end = self.get_laser_end_point();
            Some((start, end))
        } else {
            None
        }
    }
    
    // Getters for special abilities
    pub fn get_shield_active(&self) -> bool {
        self.enemy_type == EnemyType::Shield
    }
    
    pub fn get_shield_radius(&self) -> f32 {
        self.shield_radius
    }
    
    pub fn should_drop_mine(&self) -> bool {
        self.enemy_type == EnemyType::Bomber && self.special_cooldown <= 0.01 && self.mine_count > 0
    }
    
    pub fn get_wave_active(&self) -> bool {
        self.enemy_type == EnemyType::Disruptor && self.wave_charge > 2.0
    }
    
    pub fn get_wave_radius(&self) -> f32 {
        if self.get_wave_active() {
            (self.wave_charge - 2.0) * 200.0 // Expanding wave
        } else {
            0.0
        }
    }
    
    pub fn should_spawn_swarm(&self) -> bool {
        self.enemy_type == EnemyType::Carrier && self.spawn_timer >= 4.0
    }
    
    pub fn is_reflecting(&self) -> bool {
        self.enemy_type == EnemyType::Reflector && self.reflection_active
    }
    
    pub fn get_vortex_strength(&self) -> f32 {
        if self.enemy_type == EnemyType::Vortex {
            self.vortex_strength
        } else {
            0.0
        }
    }
}

impl Drawable for Enemy {
    fn draw(&self, renderer: &mut Renderer) {
        // Check for NaN values
        if self.pos.is_nan() || self.rotation.is_nan() {
            println!("WARNING: Enemy has NaN values! Type: {:?}, Pos: {:?}, Rotation: {:?}", 
                self.enemy_type, self.pos, self.rotation);
            return;
        }
        
        let rotation = rotation_matrix(self.rotation.y, self.rotation.x, self.rotation.z);
        
        match self.enemy_type {
            EnemyType::Cube => {
                // Basic enemy - simple cube with decorative elements
                renderer.draw_cube(self.pos, 40.0, rotation);
                // Add smaller rotating cube inside
                let inner_rotation = rotation_matrix(self.rotation.y * -2.0, self.rotation.x * -2.0, 0.0);
                renderer.draw_cube(self.pos, 20.0, inner_rotation);
            }
            EnemyType::Pyramid => {
                // Aggressive enemy - sharp double pyramid (octahedron)
                renderer.draw_octahedron(self.pos, 35.0, rotation);
                // Add spinning blades
                let blade_rotation = rotation_matrix(self.phase * 4.0, 0.0, 0.0);
                renderer.draw_pyramid(self.pos, 25.0, blade_rotation);
            }
            EnemyType::Spinner => {
                // Orbital enemy - complex spinning structure
                // Central octahedron
                renderer.draw_octahedron(self.pos, 25.0, rotation);
                // Three rotating rings
                // Removed debug logging
                let ring1 = rotation_matrix(self.rotation.y * 2.0, 0.0, 0.0);
                let ring2 = rotation_matrix(0.0, self.rotation.x * 2.0, 0.0);
                let ring3 = rotation_matrix(0.0, 0.0, self.rotation.z * 2.0);
                renderer.draw_cube(self.pos, 35.0, ring1);
                renderer.draw_cube(self.pos, 30.0, ring2);
                renderer.draw_cube(self.pos, 25.0, ring3);
            }
            EnemyType::Hunter => {
                // Tracking enemy - spike ball design
                renderer.draw_spike_ball(self.pos, 25.0, 20.0, rotation);
                // Add pulsing core
                let pulse = (self.phase * 3.0).sin() * 0.2 + 0.8;
                let core_rotation = rotation_matrix(self.rotation.y * -1.0, self.rotation.x * -1.0, 0.0);
                renderer.draw_octahedron(self.pos, 15.0 * pulse, core_rotation);
            }
            EnemyType::Guardian => {
                // Defensive enemy - hexagonal fortress
                renderer.draw_hexagon_prism(self.pos, 40.0, 30.0, rotation);
                // Add rotating shields
                let shield_rotation = rotation_matrix(self.phase * 0.5, 0.0, 0.0);
                renderer.draw_hexagon_prism(self.pos, 50.0, 15.0, shield_rotation);
                // Central core
                renderer.draw_octahedron(self.pos, 20.0, rotation);
            }
            EnemyType::Laser => {
                // Laser enemy - high-tech appearance
                renderer.draw_hexagon_prism(self.pos, 35.0, 40.0, rotation);
                // Add energy core that glows when laser is active
                let energy_scale = if self.laser_active { 1.5 } else { 1.0 };
                let core_rotation = rotation_matrix(self.phase * 5.0, self.phase * 3.0, 0.0);
                renderer.draw_octahedron(self.pos, 20.0 * energy_scale, core_rotation);
                // Rotating rings
                let ring_rotation = rotation_matrix(self.rotation.y * 3.0, self.rotation.x * 2.0, 0.0);
                renderer.draw_cube(self.pos + Vec3::new(0.0, 20.0, 0.0), 25.0, ring_rotation);
                renderer.draw_cube(self.pos - Vec3::new(0.0, 20.0, 0.0), 25.0, ring_rotation);
            }
            EnemyType::Swarm => {
                // Small triangular shape
                let size = 15.0;
                renderer.draw_pyramid(self.pos, size, rotation);
                // Add small wings
                let wing_rotation = rotation_matrix(self.phase * 8.0, 0.0, 0.0);
                renderer.draw_pyramid(self.pos + Vec3::new(10.0, 0.0, 0.0), size * 0.5, wing_rotation);
                renderer.draw_pyramid(self.pos - Vec3::new(10.0, 0.0, 0.0), size * 0.5, wing_rotation);
            }
            EnemyType::Phaser => {
                // Diamond shape with energy rings
                let charge_scale = 1.0 + (self.teleport_charge * 0.5).min(1.0);
                renderer.draw_octahedron(self.pos, 30.0 * charge_scale, rotation);
                
                // Energy rings that expand when charging
                for i in 0..3 {
                    let ring_scale = 1.0 + (self.teleport_charge + i as f32 * 0.3).sin() * 0.3;
                    // Wrap phase to prevent numerical issues
                    let phase_wrapped = self.phase % (2.0 * PI);
                    let ring_rotation = rotation_matrix(
                        phase_wrapped * (i + 1) as f32,
                        phase_wrapped * 0.5,
                        0.0
                    );
                    renderer.draw_cube(self.pos, 40.0 * ring_scale, ring_rotation);
                }
            }
            EnemyType::Shield => {
                // Central orb
                renderer.draw_octahedron(self.pos, 25.0, rotation);
                
                // Rotating shield panels
                for i in 0..6 {
                    let phase_wrapped = self.phase % (2.0 * PI);
                    let angle = i as f32 * PI / 3.0 + phase_wrapped;
                    let panel_pos = self.pos + Vec3::new(
                        angle.cos() * 40.0,
                        (angle * 2.0).sin() * 10.0,
                        angle.sin() * 40.0
                    );
                    let panel_rotation = rotation_matrix(angle % (2.0 * PI), phase_wrapped, 0.0);
                    renderer.draw_cube(panel_pos, 15.0, panel_rotation);
                }
            }
            EnemyType::Bomber => {
                // Large sphere with spikes
                renderer.draw_octahedron(self.pos, 40.0, rotation);
                
                // Spike indicators
                for i in 0..self.mine_count {
                    let angle = i as f32 * PI * 2.0 / 5.0;
                    let spike_offset = Vec3::new(
                        angle.cos() * 30.0,
                        0.0,
                        angle.sin() * 30.0
                    );
                    renderer.draw_pyramid(self.pos + spike_offset, 15.0, rotation);
                }
            }
            EnemyType::Disruptor => {
                // Twisted spiral shape
                let twist = self.phase * 2.0;
                for i in 0..5 {
                    let height_offset = (i as f32 - 2.0) * 10.0;
                    let twist_angle = twist + i as f32 * 0.5;
                    let ring_pos = self.pos + Vec3::new(0.0, height_offset, 0.0);
                    let ring_rotation = rotation_matrix(twist_angle, 0.0, 0.0);
                    renderer.draw_cube(ring_pos, 30.0 - i as f32 * 3.0, ring_rotation);
                }
                
                // Charging effect
                if self.wave_charge > 0.0 {
                    let charge_size = 50.0 * (self.wave_charge / 2.0).min(1.0);
                    renderer.draw_octahedron(self.pos, charge_size, rotation);
                }
            }
            EnemyType::Carrier => {
                // Large hexagonal platform
                renderer.draw_hexagon_prism(self.pos, 60.0, 20.0, rotation);
                
                // Hangar bays
                for i in 0..4 {
                    let angle = i as f32 * PI / 2.0;
                    let bay_offset = Vec3::new(
                        angle.cos() * 45.0,
                        -10.0,
                        angle.sin() * 45.0
                    );
                    renderer.draw_cube(self.pos + bay_offset, 20.0, rotation);
                }
                
                // Spawn indicator
                if self.spawn_timer > 3.0 {
                    let blink = ((self.spawn_timer - 3.0) * 10.0).sin();
                    if blink > 0.0 {
                        renderer.draw_octahedron(self.pos - Vec3::new(0.0, 20.0, 0.0), 15.0, rotation);
                    }
                }
            }
            EnemyType::Reflector => {
                // Crystalline shape with mirror facets
                let facet_rotation = rotation_matrix(self.rotation.y, 0.0, 0.0);
                
                // Main crystal
                renderer.draw_octahedron(self.pos, 35.0, facet_rotation);
                
                // Mirror panels
                for i in 0..8 {
                    let angle = i as f32 * PI / 4.0;
                    let panel_offset = Vec3::new(
                        angle.cos() * 25.0,
                        0.0,
                        angle.sin() * 25.0
                    );
                    let panel_rotation = rotation_matrix(self.rotation.y + angle, PI / 4.0, 0.0);
                    renderer.draw_pyramid(self.pos + panel_offset, 15.0, panel_rotation);
                }
            }
            EnemyType::Vortex => {
                // Swirling energy spiral
                let vortex_speed = self.phase * 3.0;
                
                // Central core
                renderer.draw_octahedron(self.pos, 20.0, rotation);
                
                // Swirling rings
                for i in 0..8 {
                    let height = (i as f32 - 4.0) * 15.0;
                    let ring_angle = vortex_speed + i as f32 * 0.5;
                    let ring_size = 30.0 + i as f32 * 5.0;
                    let _ring_pos = self.pos + Vec3::new(0.0, height, 0.0);
                    let ring_rotation = rotation_matrix(ring_angle, 0.0, 0.0);
                    
                    // Draw partial ring to show swirl
                    for j in 0..3 {
                        let segment_angle = ring_angle + j as f32 * 2.0 * PI / 3.0;
                        let segment_offset = Vec3::new(
                            segment_angle.cos() * ring_size,
                            height,
                            segment_angle.sin() * ring_size
                        );
                        renderer.draw_cube(segment_offset, 10.0, ring_rotation);
                    }
                }
            }
        }
    }
}