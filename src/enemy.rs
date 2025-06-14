use glam::Vec3;
use rand::prelude::*;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use crate::terrain_generation;
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

#[derive(Clone, Copy, PartialEq)]
pub enum AlertState {
    Unaware,     // Normal patrol behavior
    Suspicious,  // Heard something, investigating
    Alert,       // Saw player, engaging
    Searching,   // Lost player, searching area
    Returning,   // Returning to patrol
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
    pub patrol_waypoints: Vec<Vec3>, // Waypoints for patrol pattern
    pub current_waypoint: usize,     // Current waypoint index
    speed: f32,             // Base movement speed
    aggression: f32,        // How aggressive this enemy is (0-1)
    detection_range: f32,   // How far it can detect player
    attack_cooldown: f32,   // Time until next attack
    can_attack: bool,       // Whether this enemy type can attack
    // Alert system fields
    pub alert_state: AlertState,     // Current alert level
    pub last_known_player_pos: Vec3, // Where we last saw the player
    pub alert_cooldown: f32,         // Time until alert level decreases
    pub search_timer: f32,           // How long we've been searching
    pub investigation_point: Vec3,   // Where to investigate suspicious activity
    pub communication_range: f32,    // How far this enemy can alert others
    pub view_cone_angle: f32,        // Field of view in radians
    pub hearing_range: f32,          // How far this enemy can hear
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
    // Stuck detection fields
    last_position: Vec3,    // Position from last frame
    stuck_timer: f32,       // How long we've been stuck
    stuck_threshold: f32,   // Movement threshold to consider stuck
    // Combat maneuvering fields
    combat_maneuver_timer: f32,  // Timer for changing maneuvers
    combat_maneuver_type: i32,   // Current maneuver type (0=approach, 1=strafe_left, 2=strafe_right, 3=circle, 4=retreat)
    preferred_combat_distance: f32, // Ideal distance to maintain from player
    zigzag_phase: f32,           // Phase for zigzag movement
    zigzag_direction: f32,       // Current zigzag direction (-1 or 1)
    close_range_angle: f32,      // Random movement angle when very close
}


impl Enemy {
    pub fn new(x: f32, z: f32, enemy_type: EnemyType) -> Self {
        let mut rng = thread_rng();
        
        // Calculate spawn height based on terrain
        let terrain_height = terrain_generation::height_at(x, z);
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
        
        // Set speed based on enemy type - MUCH faster for huge map
        let speed = match enemy_type {
            EnemyType::Cube => rng.gen_range(200.0..300.0),
            EnemyType::Pyramid => rng.gen_range(350.0..450.0),
            EnemyType::Spinner => rng.gen_range(250.0..350.0),
            EnemyType::Hunter => rng.gen_range(300.0..400.0),
            EnemyType::Guardian => rng.gen_range(150.0..250.0),
            EnemyType::Laser => rng.gen_range(100.0..150.0),
            EnemyType::Swarm => rng.gen_range(400.0..500.0),
            EnemyType::Phaser => 0.0, // Teleports instead, no speed needed
            EnemyType::Shield => rng.gen_range(80.0..120.0),
            EnemyType::Bomber => rng.gen_range(120.0..180.0),
            EnemyType::Disruptor => rng.gen_range(150.0..200.0),
            EnemyType::Carrier => rng.gen_range(60.0..100.0),
            EnemyType::Reflector => rng.gen_range(180.0..250.0),
            EnemyType::Vortex => 0.0, // Stationary, no speed needed
        };
        
        // Set aggression, detection range, and alert parameters - MUCH more aggressive for huge map
        let (aggression, detection_range, view_angle, hearing_range, comm_range) = match enemy_type {
            EnemyType::Cube => (0.5, 1500.0, PI * 0.7, 800.0, 1200.0),
            EnemyType::Pyramid => (0.8, 2500.0, PI * 0.6, 1200.0, 1800.0),
            EnemyType::Spinner => (0.6, 2000.0, PI * 0.8, 1000.0, 1500.0),
            EnemyType::Hunter => (0.95, 4000.0, PI * 0.5, 2000.0, 2500.0), // Hunters are terrifying
            EnemyType::Guardian => (0.6, 1800.0, PI * 0.9, 1500.0, 2000.0),
            EnemyType::Laser => (0.7, 3000.0, PI * 0.4, 1500.0, 2000.0),
            EnemyType::Swarm => (0.9, 1200.0, PI, 800.0, 3000.0), // Swarms communicate very well
            EnemyType::Phaser => (0.8, 5000.0, PI * 0.4, 1800.0, 1500.0), // Long range snipers
            EnemyType::Shield => (0.3, 2000.0, PI * 1.5, 1200.0, 2500.0), // Wide vision
            EnemyType::Bomber => (0.5, 2200.0, PI * 0.7, 1000.0, 1500.0),
            EnemyType::Disruptor => (0.6, 2500.0, PI * 0.8, 1400.0, 1800.0),
            EnemyType::Carrier => (0.4, 3000.0, PI * 0.9, 1500.0, 3000.0),
            EnemyType::Reflector => (0.7, 2000.0, PI * 0.6, 1200.0, 1500.0),
            EnemyType::Vortex => (0.0, 1500.0, PI * 2.0, 2000.0, 2500.0), // 360 degree awareness
        };
        
        let spawn_point = Vec3::new(x, spawn_height, z);
        
        // Generate patrol waypoints - all enemies patrol now, not just Guardians
        let patrol_waypoints = {
            let mut waypoints = Vec::new();
            let patrol_radius = match enemy_type {
                EnemyType::Guardian => 2000.0,
                EnemyType::Hunter => 3000.0,
                EnemyType::Swarm => 1500.0,
                EnemyType::Phaser => 4000.0, // Snipers patrol large areas
                _ => 1000.0,
            };
            
            let waypoint_count = match enemy_type {
                EnemyType::Guardian => 6,
                EnemyType::Hunter => 8,
                _ => 4,
            };
            
            for i in 0..waypoint_count {
                let angle = i as f32 * PI * 2.0 / waypoint_count as f32 + rng.gen_range(-0.3..0.3);
                let radius_variation = patrol_radius * rng.gen_range(0.7..1.3);
                let wx = x + angle.cos() * radius_variation;
                let wz = z + angle.sin() * radius_variation;
                let wy = terrain_generation::height_at(wx, wz) + rng.gen_range(50.0..200.0);
                waypoints.push(Vec3::new(wx, wy, wz));
            }
            waypoints
        };
        
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
            patrol_waypoints,
            current_waypoint: 0,
            speed,
            aggression: aggression + rng.gen_range(-0.1..0.1),
            detection_range,
            attack_cooldown: 0.0,
            can_attack: matches!(enemy_type, EnemyType::Pyramid | EnemyType::Hunter | EnemyType::Guardian),
            // Initialize alert system
            alert_state: AlertState::Unaware,
            last_known_player_pos: spawn_point,
            alert_cooldown: 0.0,
            search_timer: 0.0,
            investigation_point: spawn_point,
            communication_range: comm_range,
            view_cone_angle: view_angle,
            hearing_range,
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
            // Initialize stuck detection
            last_position: Vec3::new(x, spawn_height, z),
            stuck_timer: 0.0,
            stuck_threshold: 5.0, // Consider stuck if moved less than 5 units
            // Initialize combat maneuvering
            combat_maneuver_timer: 0.0,
            combat_maneuver_type: 0,
            preferred_combat_distance: {
                let base_distance = match enemy_type {
                    EnemyType::Pyramid => 300.0,     // Fast attackers stay closer
                    EnemyType::Hunter => 400.0,      // Mid-range
                    EnemyType::Guardian => 500.0,    // Defensive, longer range
                    EnemyType::Laser => 600.0,       // Long range
                    EnemyType::Phaser => 800.0,      // Sniper range
                    _ => 350.0,                     // Default
                };
                // Add individual variation of ±20%
                base_distance * rng.gen_range(0.8..1.2)
            },
            zigzag_phase: rng.gen_range(0.0..PI * 2.0),
            zigzag_direction: if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
            close_range_angle: rng.gen_range(0.0..PI * 2.0),
        }
    }
    
    pub fn new_with_pattern(x: f32, z: f32, enemy_type: EnemyType, pattern: MovementPattern) -> Self {
        let mut enemy = Self::new(x, z, enemy_type);
        enemy.movement_pattern = pattern;
        enemy
    }
    
    pub fn new_with_patrol(x: f32, z: f32, enemy_type: EnemyType, waypoints: Vec<Vec3>) -> Self {
        let mut enemy = Self::new(x, z, enemy_type);
        enemy.movement_pattern = MovementPattern::Patrol;
        enemy.patrol_waypoints = waypoints;
        enemy.current_waypoint = 0;
        enemy
    }

    pub fn update(&mut self, dt: f32) {
        self.rotation += self.rotation_speed * dt;
        self.phase += dt;
    }
    
    fn calculate_combat_movement(&mut self, player_pos: Vec3, distance_to_player: f32, dt: f32) -> Vec3 {
        let mut rng = thread_rng();
        
        // Define close range threshold
        let close_range_threshold = 150.0;
        
        // Update maneuver timer
        self.combat_maneuver_timer -= dt;
        if self.combat_maneuver_timer <= 0.0 {
            // Change maneuver
            self.combat_maneuver_timer = rng.gen_range(1.0..3.0);
            
            if distance_to_player < close_range_threshold {
                // Very close - random evasive movements
                self.combat_maneuver_type = 5; // New close-range mode
                self.close_range_angle = rng.gen_range(0.0..PI * 2.0);
            } else if distance_to_player < self.preferred_combat_distance * 0.7 {
                // Too close, prefer evasive maneuvers
                self.combat_maneuver_type = rng.gen_range(1..=4);
            } else if distance_to_player > self.preferred_combat_distance * 1.5 {
                // Too far, approach
                self.combat_maneuver_type = 0;
            } else {
                // Good distance, mix it up
                self.combat_maneuver_type = rng.gen_range(0..=3);
            }
        }
        
        let to_player = player_pos - self.pos;
        let to_player_normalized = to_player.normalize_or_zero();
        
        // Calculate height difference for vertical tracking
        let height_diff = player_pos.y - self.pos.y;
        let vertical_tracking = height_diff.clamp(-self.speed * 0.5, self.speed * 0.5);
        
        // Calculate base movement based on maneuver type
        let base_movement = match self.combat_maneuver_type {
            0 => {
                // Longer zigzag approach
                self.zigzag_phase += dt * 1.5; // Slower frequency for longer zigzags
                
                // Change direction less frequently for longer runs
                if self.zigzag_phase % (PI * 2.0) < 0.1 {
                    self.zigzag_direction *= -1.0;
                }
                
                let perpendicular = Vec3::new(-to_player_normalized.z, 0.0, to_player_normalized.x);
                let zigzag_amplitude = 150.0; // Larger amplitude
                let zigzag_offset = perpendicular * (self.zigzag_direction * zigzag_amplitude);
                
                // Blend approach with zigzag
                to_player_normalized * 0.7 + zigzag_offset.normalize_or_zero() * 0.5
            },
            1 => {
                // Strafe left
                let left = Vec3::new(-to_player_normalized.z, 0.0, to_player_normalized.x);
                left + to_player_normalized * 0.2
            },
            2 => {
                // Strafe right
                let right = Vec3::new(to_player_normalized.z, 0.0, -to_player_normalized.x);
                right + to_player_normalized * 0.2
            },
            3 => {
                // Circle around player
                let tangent = Vec3::new(-to_player.z, 0.0, to_player.x).normalize_or_zero();
                let radius_correction = if distance_to_player < self.preferred_combat_distance {
                    -to_player_normalized * 0.3
                } else {
                    to_player_normalized * 0.3
                };
                tangent + radius_correction
            },
            4 => {
                // Tactical retreat with evasion
                let retreat = -to_player_normalized;
                let dodge = Vec3::new(
                    (self.phase * 2.0).sin() * 0.5,
                    0.0,
                    (self.phase * 2.0).cos() * 0.5
                );
                retreat + dodge
            },
            5 => {
                // Close range random movement
                self.close_range_angle += rng.gen_range(-1.0..1.0) * dt * 3.0;
                let random_dir = Vec3::new(
                    self.close_range_angle.cos(),
                    0.0,
                    self.close_range_angle.sin()
                );
                // Mix random movement with slight player tracking
                random_dir * 0.8 + to_player_normalized * 0.2
            },
            _ => to_player_normalized,
        };
        
        // Combine horizontal movement with vertical tracking
        let mut movement = base_movement * self.speed * self.aggression;
        movement.y = vertical_tracking + (self.phase * 0.8).sin() * 10.0; // Track height + small variation
        
        movement
    }
    
    pub fn update_with_player(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        // Wrap phase to prevent overflow
        if self.phase > 1000.0 {
            self.phase = self.phase % (2.0 * PI);
        }
        
        // Update alert system first
        self.update_alert_state(player_pos, dt);
        
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
        
        // Override movement pattern based on alert state
        let effective_pattern = match self.alert_state {
            AlertState::Alert => MovementPattern::Tracking,
            AlertState::Searching => MovementPattern::Hover, // Will move to search point
            AlertState::Suspicious => MovementPattern::Hover, // Will investigate
            _ => self.movement_pattern,
        };
        
        // Update based on movement pattern
        match effective_pattern {
            MovementPattern::Hover => {
                // Maintain altitude above terrain
                let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
                let hover_height = terrain_height + 50.0 + (self.phase * 0.5).sin() * 15.0;
                
                // Smooth altitude adjustment
                let height_diff = hover_height - self.pos.y;
                self.vel.y = height_diff * 2.0;
                
                // Movement based on alert state
                match self.alert_state {
                    AlertState::Alert if distance_to_player < self.preferred_combat_distance * 1.5 => {
                        // Use combat maneuvering when engaged
                        let combat_vel = self.calculate_combat_movement(player_pos, distance_to_player, dt);
                        self.vel.x = combat_vel.x;
                        self.vel.z = combat_vel.z;
                        self.vel.y = height_diff * 2.0 + combat_vel.y * 0.5; // Blend altitude maintenance
                    }
                    AlertState::Searching => {
                        // Move towards last known player position
                        let to_search = self.last_known_player_pos - self.pos;
                        let horizontal = Vec3::new(to_search.x, 0.0, to_search.z).normalize_or_zero();
                        self.vel.x = horizontal.x * self.speed * 1.2; // Search faster
                        self.vel.z = horizontal.z * self.speed * 1.2;
                    }
                    AlertState::Suspicious => {
                        // Move towards investigation point
                        let to_investigate = self.investigation_point - self.pos;
                        let horizontal = Vec3::new(to_investigate.x, 0.0, to_investigate.z).normalize_or_zero();
                        self.vel.x = horizontal.x * self.speed; // Investigate at full speed
                        self.vel.z = horizontal.z * self.speed;
                    }
                    _ => {
                        // Active patrolling even when unaware
                        let wander_speed = self.speed * 0.8; // Much faster wandering
                        self.vel.x = (self.phase * 0.3).sin() * wander_speed;
                        self.vel.z = (self.phase * 0.25 + 1.0).cos() * wander_speed;
                        
                        // Add some forward momentum
                        let forward_angle = self.phase * 0.1;
                        self.vel.x += forward_angle.cos() * self.speed * 0.3;
                        self.vel.z += forward_angle.sin() * self.speed * 0.3;
                    }
                }
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
                let target_y = terrain_generation::height_at(target_x, target_z) + 60.0 + (self.phase * 2.0).sin() * 20.0;
                
                let target = Vec3::new(target_x, target_y, target_z);
                let to_target = target - self.pos;
                self.vel = to_target.normalize_or_zero() * self.speed;
            },
            
            MovementPattern::Diving => {
                // Swooping attack pattern
                let dive_cycle = self.phase % 4.0;
                
                if dive_cycle < 2.0 {
                    // Climbing phase
                    let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
                    let climb_target = terrain_height + 150.0;
                    self.vel.y = (climb_target - self.pos.y).clamp(-self.speed, self.speed);
                    
                    // Move towards player horizontally during climb with evasion
                    if distance_to_player < self.detection_range {
                        if self.alert_state == AlertState::Alert && distance_to_player < self.preferred_combat_distance {
                            // Use combat maneuvering during climb
                            let combat_vel = self.calculate_combat_movement(player_pos, distance_to_player, dt);
                            self.vel.x = combat_vel.x * 0.7; // Slower horizontal during climb
                            self.vel.z = combat_vel.z * 0.7;
                        } else {
                            let horizontal = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
                            self.vel.x = horizontal.x * self.speed * 0.5;
                            self.vel.z = horizontal.z * self.speed * 0.5;
                        }
                    }
                } else {
                    // Diving phase with spiral
                    if distance_to_player < self.detection_range {
                        // Spiral dive towards player
                        let spiral_angle = self.phase * 5.0;
                        let spiral_offset = Vec3::new(
                            spiral_angle.cos() * 30.0,
                            0.0,
                            spiral_angle.sin() * 30.0
                        );
                        let dive_target = player_pos + Vec3::new(0.0, -20.0, 0.0) + spiral_offset;
                        let to_target = dive_target - self.pos;
                        self.vel = to_target.normalize_or_zero() * self.speed * 1.5;
                    } else {
                        // Random dive
                        self.vel.y = -self.speed;
                    }
                }
            },
            
            MovementPattern::Tracking => {
                // Use last known position if in alert state
                let target_pos = if self.alert_state == AlertState::Alert {
                    player_pos
                } else {
                    self.last_known_player_pos
                };
                
                let distance_to_target = (target_pos - self.pos).length();
                
                // Use combat maneuvering when engaged
                if self.alert_state == AlertState::Alert && distance_to_player < self.detection_range {
                    self.vel = self.calculate_combat_movement(player_pos, distance_to_player, dt);
                } else {
                    // Normal approach when not in combat
                    let to_target = target_pos - self.pos;
                    let speed_mult = match self.alert_state {
                        AlertState::Alert => 1.2,
                        AlertState::Searching => 0.9,
                        _ => 0.7,
                    };
                    self.vel = to_target.normalize_or_zero() * self.speed * self.aggression * speed_mult;
                }
                
                // Maintain some altitude
                let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
                if self.pos.y < terrain_height + 30.0 {
                    self.vel.y = (self.vel.y + 50.0).max(0.0);
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
                let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
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
                    let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
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
                let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
                let hover_height = terrain_height + 60.0;
                self.pos.y = self.pos.y * 0.9 + hover_height * 0.1;
            },
            
            MovementPattern::Drifting => {
                // Bomber/Carrier - slow drift
                let drift_angle = self.phase * 0.1;
                self.vel.x = drift_angle.cos() * self.speed;
                self.vel.z = drift_angle.sin() * self.speed;
                
                // Maintain high altitude
                let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
                let cruise_height = terrain_height + 150.0;
                self.vel.y = (cruise_height - self.pos.y).clamp(-20.0, 20.0);
            },
            
            MovementPattern::Patrol => {
                // Use waypoint system for patrol
                if !self.patrol_waypoints.is_empty() {
                    // Get current target waypoint
                    let target_waypoint = self.patrol_waypoints[self.current_waypoint];
                    let to_target = target_waypoint - self.pos;
                    let distance_to_waypoint = to_target.length();
                    
                    // Check if we've reached the waypoint
                    if distance_to_waypoint < 50.0 {
                        // Move to next waypoint
                        self.current_waypoint = self.current_waypoint + 1;
                        
                        // If we've reached the end of the patrol route, convert to tracking behavior
                        if self.current_waypoint >= self.patrol_waypoints.len() {
                            self.movement_pattern = MovementPattern::Tracking;
                            self.vel = Vec3::ZERO;
                        }
                    } else {
                        // Move towards waypoint only if we're far enough away
                        self.vel = to_target.normalize_or_zero() * self.speed * 1.5; // Faster patrol
                    }
                    
                    // React to player if detected - switch to tracking immediately
                    if distance_to_player < self.detection_range && self.alert_state == AlertState::Alert {
                        self.movement_pattern = MovementPattern::Tracking;
                    }
                } else {
                    // No waypoints - switch to tracking
                    self.movement_pattern = MovementPattern::Tracking;
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
        let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
        if self.pos.y < terrain_height + 10.0 {
            self.pos.y = terrain_height + 10.0;
            self.vel.y = self.vel.y.max(0.0);
        }
        
        // Stuck detection and recovery
        let movement_distance = (self.pos - self.last_position).length();
        if movement_distance < self.stuck_threshold * dt && self.vel.length() > 10.0 {
            // We're trying to move but not making progress
            self.stuck_timer += dt;
            
            if self.stuck_timer > 2.0 {
                // We've been stuck for 2 seconds, try to recover
                match self.movement_pattern {
                    MovementPattern::Patrol => {
                        // Skip to next waypoint
                        self.current_waypoint = (self.current_waypoint + 1) % self.patrol_waypoints.len();
                        self.stuck_timer = 0.0;
                    },
                    _ => {
                        // For other patterns, add random impulse
                        let mut rng = thread_rng();
                        self.vel.x += rng.gen_range(-50.0..50.0);
                        self.vel.z += rng.gen_range(-50.0..50.0);
                        self.vel.y += 20.0; // Move up a bit
                        self.stuck_timer = 0.0;
                    }
                }
            }
        } else {
            // We're moving fine, reset stuck timer
            self.stuck_timer = 0.0;
        }
        
        // Update last position for next frame
        self.last_position = self.pos;
        
        // Check if we should attack - only when alert
        if self.alert_state == AlertState::Alert && self.can_attack && self.attack_cooldown <= 0.0 {
            let attack_chance = match self.enemy_type {
                EnemyType::Pyramid => 0.04,    // Fast shooters - doubled
                EnemyType::Hunter => 0.06,     // Aggressive shooters - doubled
                EnemyType::Guardian => 0.03,   // Defensive shooters - doubled
                _ => 0.0,
            };
            
            if rand::random::<f32>() < attack_chance * self.aggression {
                // Set cooldown based on enemy type - much faster fire rates
                self.attack_cooldown = match self.enemy_type {
                    EnemyType::Pyramid => 0.75,
                    EnemyType::Hunter => 0.5,
                    EnemyType::Guardian => 1.0,
                    _ => 1.0,
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
        let terrain_height = terrain_generation::height_at(self.pos.x, self.pos.z);
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
            // Start a new laser attack with much higher chance
            if rand::random::<f32>() < 0.1 * self.aggression { // Doubled from 0.05
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
    
    pub fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement_pattern = MovementPattern::Patrol;
        self.patrol_waypoints = waypoints;
        self.current_waypoint = 0;
    }
    
    fn update_alert_state(&mut self, player_pos: Vec3, dt: f32) {
        let to_player = player_pos - self.pos;
        let distance = to_player.length();
        
        // Update alert cooldown
        if self.alert_cooldown > 0.0 {
            self.alert_cooldown -= dt;
        }
        
        match self.alert_state {
            AlertState::Unaware => {
                // Check if we can see the player
                if self.can_see_player(player_pos, distance) {
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0; // Stay alert much longer
                    println!("Enemy {:?} spotted player!", self.enemy_type);
                } else if self.can_hear_player(distance) {
                    // Heard something suspicious
                    self.alert_state = AlertState::Suspicious;
                    self.investigation_point = player_pos;
                    self.alert_cooldown = 3.0;
                }
            }
            
            AlertState::Suspicious => {
                // Investigating suspicious noise
                let dist_to_investigate = (self.investigation_point - self.pos).length();
                
                if self.can_see_player(player_pos, distance) {
                    // Found the player!
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0; // Stay alert much longer
                } else if dist_to_investigate < 50.0 || self.alert_cooldown <= 0.0 {
                    // Reached investigation point or gave up
                    self.alert_state = AlertState::Returning;
                    self.alert_cooldown = 2.0;
                }
            }
            
            AlertState::Alert => {
                // Actively engaging player
                if self.can_see_player(player_pos, distance) {
                    // Update last known position
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 5.0;
                } else if self.alert_cooldown <= 0.0 {
                    // Lost sight of player
                    self.alert_state = AlertState::Searching;
                    self.search_timer = 0.0;
                }
            }
            
            AlertState::Searching => {
                // Looking for lost player
                self.search_timer += dt;
                
                if self.can_see_player(player_pos, distance) {
                    // Found player again!
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0; // Stay alert much longer
                    self.search_timer = 0.0;
                } else if self.search_timer > 30.0 { // Search for much longer
                    // Give up searching
                    self.alert_state = AlertState::Returning;
                    self.alert_cooldown = 2.0;
                }
            }
            
            AlertState::Returning => {
                // Going back to patrol
                let dist_to_spawn = (self.spawn_point - self.pos).length();
                
                if self.can_see_player(player_pos, distance) {
                    // Spotted player again!
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0; // Stay alert much longer
                } else if dist_to_spawn < 100.0 || self.alert_cooldown <= 0.0 {
                    // Returned to patrol
                    self.alert_state = AlertState::Unaware;
                }
            }
        }
    }
    
    fn can_see_player(&self, player_pos: Vec3, distance: f32) -> bool {
        // Check distance first
        if distance > self.detection_range {
            return false;
        }
        
        // Check if player is within view cone
        let to_player = (player_pos - self.pos).normalize_or_zero();
        if to_player.length() == 0.0 {
            return false;
        }
        
        // Get enemy's forward direction based on velocity or rotation
        let forward = if self.vel.length() > 0.1 {
            self.vel.normalize()
        } else {
            // Use rotation for stationary enemies
            Vec3::new(-self.rotation.y.sin(), 0.0, -self.rotation.y.cos())
        };
        
        // Calculate angle between forward direction and player direction
        let dot = forward.dot(to_player);
        let angle = dot.acos();
        
        // Check if within view cone
        angle <= self.view_cone_angle / 2.0
    }
    
    fn can_hear_player(&self, distance: f32) -> bool {
        // Simple hearing check - could be enhanced with player speed/actions
        distance <= self.hearing_range
    }
    
    pub fn alert_nearby_enemies(&self, all_enemies: &mut [Enemy]) {
        // Called when this enemy spots the player
        if self.alert_state != AlertState::Alert {
            return;
        }
        
        for other in all_enemies.iter_mut() {
            // Don't alert ourselves
            if std::ptr::eq(self, other) {
                continue;
            }
            
            let distance = (other.pos - self.pos).length();
            if distance <= self.communication_range && other.alert_state == AlertState::Unaware {
                // Alert the other enemy
                other.alert_state = AlertState::Suspicious;
                other.investigation_point = self.last_known_player_pos;
                other.alert_cooldown = 3.0;
            }
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