use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;
use crate::terrain;

#[derive(Clone, Copy, Debug)]
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

pub struct MovementController {
    pub movement_pattern: MovementPattern,
    pub patrol_waypoints: Vec<Vec3>,
    pub current_waypoint: usize,
    pub target_point: Vec3,
    pub speed: f32,
    
    // Stuck detection
    last_position: Vec3,
    stuck_timer: f32,
    stuck_threshold: f32,
    
    // Combat maneuvering
    combat_maneuver_timer: f32,
    combat_maneuver_type: i32,
    zigzag_phase: f32,
    zigzag_direction: f32,
    close_range_angle: f32,
}

impl MovementController {
    pub fn new(pattern: MovementPattern, speed: f32, spawn_pos: Vec3) -> Self {
        let mut rng = thread_rng();
        
        Self {
            movement_pattern: pattern,
            patrol_waypoints: Vec::new(),
            current_waypoint: 0,
            target_point: spawn_pos,
            speed,
            last_position: spawn_pos,
            stuck_timer: 0.0,
            stuck_threshold: 5.0,
            combat_maneuver_timer: 0.0,
            combat_maneuver_type: 0,
            zigzag_phase: rng.gen_range(0.0..PI * 2.0),
            zigzag_direction: if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
            close_range_angle: rng.gen_range(0.0..PI * 2.0),
        }
    }
    
    pub fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement_pattern = MovementPattern::Patrol;
        self.patrol_waypoints = waypoints;
        self.current_waypoint = 0;
    }
    
    pub fn update(&mut self, pos: &mut Vec3, vel: &mut Vec3, 
                  spawn_point: Vec3, phase: f32, player_pos: Vec3, 
                  alert_state: super::AlertState, last_known_player_pos: Vec3,
                  investigation_point: Vec3, aggression: f32,
                  preferred_combat_distance: f32, dt: f32) {
        
        // Override movement pattern based on alert state
        let effective_pattern = match alert_state {
            super::AlertState::Alert => MovementPattern::Tracking,
            super::AlertState::Searching => MovementPattern::Hover,
            super::AlertState::Suspicious => MovementPattern::Hover,
            _ => self.movement_pattern,
        };
        
        let distance_to_player = (player_pos - *pos).length();
        
        match effective_pattern {
            MovementPattern::Hover => {
                self.update_hover(pos, vel, phase, player_pos, alert_state, 
                                 last_known_player_pos, investigation_point, 
                                 distance_to_player, preferred_combat_distance, aggression, dt);
            }
            
            MovementPattern::Sinusoidal => {
                self.update_sinusoidal(vel, phase);
            }
            
            MovementPattern::Orbital => {
                self.update_orbital(pos, vel, spawn_point, phase);
            }
            
            MovementPattern::Diving => {
                self.update_diving(pos, vel, phase, player_pos, distance_to_player, 
                                  alert_state, preferred_combat_distance, aggression, dt);
            }
            
            MovementPattern::Tracking => {
                self.update_tracking(pos, vel, player_pos, last_known_player_pos, 
                                    alert_state, distance_to_player, 
                                    preferred_combat_distance, aggression, dt);
            }
            
            MovementPattern::Patrol => {
                self.update_patrol(pos, vel, distance_to_player, alert_state);
            }
            
            MovementPattern::Flocking => {
                self.update_flocking(pos, vel, player_pos, phase, aggression);
            }
            
            MovementPattern::Teleport => {
                *vel = Vec3::ZERO; // No regular movement
            }
            
            MovementPattern::Stationary => {
                self.update_stationary(pos, vel);
            }
            
            MovementPattern::Drifting => {
                self.update_drifting(vel, phase);
            }
        }
        
        // Update position
        *pos += *vel * dt;
        
        // Ensure we don't go below terrain
        let terrain_height = terrain::height_at(pos.x, pos.z);
        if pos.y < terrain_height + 10.0 {
            pos.y = terrain_height + 10.0;
            vel.y = vel.y.max(0.0);
        }
        
        // Stuck detection and recovery
        self.update_stuck_detection(pos, vel, dt);
    }
    
    fn update_hover(&mut self, pos: &Vec3, vel: &mut Vec3, phase: f32, 
                    player_pos: Vec3, alert_state: super::AlertState,
                    last_known_player_pos: Vec3, investigation_point: Vec3,
                    distance_to_player: f32, preferred_combat_distance: f32,
                    aggression: f32, dt: f32) {
        
        let terrain_height = terrain::height_at(pos.x, pos.z);
        let hover_height = terrain_height + 50.0 + (phase * 0.5).sin() * 15.0;
        
        let height_diff = hover_height - pos.y;
        vel.y = height_diff * 2.0;
        
        match alert_state {
            super::AlertState::Alert if distance_to_player < preferred_combat_distance * 1.5 => {
                let combat_vel = self.calculate_combat_movement(pos, player_pos, 
                                                               distance_to_player, 
                                                               preferred_combat_distance,
                                                               aggression, dt);
                vel.x = combat_vel.x;
                vel.z = combat_vel.z;
                vel.y = height_diff * 2.0 + combat_vel.y * 0.5;
            }
            super::AlertState::Searching => {
                let to_search = last_known_player_pos - *pos;
                let horizontal = Vec3::new(to_search.x, 0.0, to_search.z).normalize_or_zero();
                vel.x = horizontal.x * self.speed * 1.2;
                vel.z = horizontal.z * self.speed * 1.2;
            }
            super::AlertState::Suspicious => {
                let to_investigate = investigation_point - *pos;
                let horizontal = Vec3::new(to_investigate.x, 0.0, to_investigate.z).normalize_or_zero();
                vel.x = horizontal.x * self.speed;
                vel.z = horizontal.z * self.speed;
            }
            _ => {
                let wander_speed = self.speed * 0.8;
                vel.x = (phase * 0.3).sin() * wander_speed;
                vel.z = (phase * 0.25 + 1.0).cos() * wander_speed;
                
                let forward_angle = phase * 0.1;
                vel.x += forward_angle.cos() * self.speed * 0.3;
                vel.z += forward_angle.sin() * self.speed * 0.3;
            }
        }
    }
    
    fn update_sinusoidal(&self, vel: &mut Vec3, phase: f32) {
        let forward_speed = self.speed;
        let wave_amplitude = 30.0;
        let wave_frequency = 2.0;
        
        vel.x = 0.0; // Direction set by rotation
        vel.z = forward_speed;
        vel.y = (phase * wave_frequency).sin() * wave_amplitude;
    }
    
    fn update_orbital(&self, pos: &Vec3, vel: &mut Vec3, spawn_point: Vec3, phase: f32) {
        let orbit_radius = 150.0;
        let orbit_speed = self.speed / orbit_radius;
        
        let angle = phase * orbit_speed;
        let target_x = spawn_point.x + angle.cos() * orbit_radius;
        let target_z = spawn_point.z + angle.sin() * orbit_radius;
        let target_y = terrain::height_at(target_x, target_z) + 60.0 + (phase * 2.0).sin() * 20.0;
        
        let target = Vec3::new(target_x, target_y, target_z);
        let to_target = target - *pos;
        *vel = to_target.normalize_or_zero() * self.speed;
    }
    
    fn update_diving(&mut self, pos: &Vec3, vel: &mut Vec3, phase: f32, 
                     player_pos: Vec3, distance_to_player: f32,
                     alert_state: super::AlertState, preferred_combat_distance: f32,
                     aggression: f32, dt: f32) {
        
        let dive_cycle = phase % 4.0;
        let to_player = player_pos - *pos;
        let detection_range = 2500.0; // TODO: Get from AI controller
        
        if dive_cycle < 2.0 {
            // Climbing phase
            let terrain_height = terrain::height_at(pos.x, pos.z);
            let climb_target = terrain_height + 150.0;
            vel.y = (climb_target - pos.y).clamp(-self.speed, self.speed);
            
            if distance_to_player < detection_range {
                if alert_state == super::AlertState::Alert && distance_to_player < preferred_combat_distance {
                    let combat_vel = self.calculate_combat_movement(pos, player_pos, 
                                                                   distance_to_player,
                                                                   preferred_combat_distance,
                                                                   aggression, dt);
                    vel.x = combat_vel.x * 0.7;
                    vel.z = combat_vel.z * 0.7;
                } else {
                    let horizontal = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
                    vel.x = horizontal.x * self.speed * 0.5;
                    vel.z = horizontal.z * self.speed * 0.5;
                }
            }
        } else {
            // Diving phase
            if distance_to_player < detection_range {
                let spiral_angle = phase * 5.0;
                let spiral_offset = Vec3::new(
                    spiral_angle.cos() * 30.0,
                    0.0,
                    spiral_angle.sin() * 30.0
                );
                let dive_target = player_pos + Vec3::new(0.0, -20.0, 0.0) + spiral_offset;
                let to_target = dive_target - *pos;
                *vel = to_target.normalize_or_zero() * self.speed * 1.5;
            } else {
                vel.y = -self.speed;
            }
        }
    }
    
    fn update_tracking(&mut self, pos: &Vec3, vel: &mut Vec3, player_pos: Vec3,
                       last_known_player_pos: Vec3, alert_state: super::AlertState,
                       distance_to_player: f32, preferred_combat_distance: f32,
                       aggression: f32, dt: f32) {
        
        let target_pos = if alert_state == super::AlertState::Alert {
            player_pos
        } else {
            last_known_player_pos
        };
        
        let detection_range = 4000.0; // TODO: Get from AI controller
        
        if alert_state == super::AlertState::Alert && distance_to_player < detection_range {
            *vel = self.calculate_combat_movement(pos, player_pos, distance_to_player,
                                                 preferred_combat_distance, aggression, dt);
        } else {
            let to_target = target_pos - *pos;
            let speed_mult = match alert_state {
                super::AlertState::Alert => 1.2,
                super::AlertState::Searching => 0.9,
                _ => 0.7,
            };
            *vel = to_target.normalize_or_zero() * self.speed * aggression * speed_mult;
        }
        
        // Maintain altitude
        let terrain_height = terrain::height_at(pos.x, pos.z);
        if pos.y < terrain_height + 30.0 {
            vel.y = (vel.y + 50.0).max(0.0);
        }
    }
    
    fn update_patrol(&mut self, pos: &Vec3, vel: &mut Vec3, 
                     distance_to_player: f32, alert_state: super::AlertState) {
        
        if !self.patrol_waypoints.is_empty() {
            let target_waypoint = self.patrol_waypoints[self.current_waypoint];
            let to_target = target_waypoint - *pos;
            let distance_to_waypoint = to_target.length();
            
            if distance_to_waypoint < 50.0 {
                self.current_waypoint = self.current_waypoint + 1;
                
                if self.current_waypoint >= self.patrol_waypoints.len() {
                    self.movement_pattern = MovementPattern::Tracking;
                    *vel = Vec3::ZERO;
                    return;
                }
            } else {
                *vel = to_target.normalize_or_zero() * self.speed * 1.5;
            }
            
            let detection_range = 1800.0; // TODO: Get from AI controller
            if distance_to_player < detection_range && alert_state == super::AlertState::Alert {
                self.movement_pattern = MovementPattern::Tracking;
            }
        } else {
            self.movement_pattern = MovementPattern::Tracking;
        }
    }
    
    fn update_flocking(&self, pos: &Vec3, vel: &mut Vec3, player_pos: Vec3, 
                       phase: f32, aggression: f32) {
        let to_player = player_pos - *pos;
        let to_player_norm = to_player.normalize_or_zero();
        
        let swarm_offset = Vec3::new(
            (phase * 7.0).sin() * 50.0,
            (phase * 5.0).cos() * 30.0,
            (phase * 6.0).sin() * 50.0
        );
        
        *vel = (to_player_norm * self.speed + swarm_offset) * aggression;
        
        let terrain_height = terrain::height_at(pos.x, pos.z);
        let swarm_height = terrain_height + 30.0 + (phase * 3.0).sin() * 10.0;
        vel.y = (swarm_height - pos.y) * 3.0;
    }
    
    fn update_stationary(&self, pos: &mut Vec3, vel: &mut Vec3) {
        *vel = Vec3::ZERO;
        
        let terrain_height = terrain::height_at(pos.x, pos.z);
        let hover_height = terrain_height + 60.0;
        pos.y = pos.y * 0.9 + hover_height * 0.1;
    }
    
    fn update_drifting(&self, vel: &mut Vec3, phase: f32) {
        let drift_angle = phase * 0.1;
        vel.x = drift_angle.cos() * self.speed;
        vel.z = drift_angle.sin() * self.speed;
        
        // Altitude handled by main update
    }
    
    fn calculate_combat_movement(&mut self, pos: &Vec3, player_pos: Vec3, 
                                distance_to_player: f32, preferred_combat_distance: f32,
                                aggression: f32, dt: f32) -> Vec3 {
        let mut rng = thread_rng();
        
        let close_range_threshold = 150.0;
        
        self.combat_maneuver_timer -= dt;
        if self.combat_maneuver_timer <= 0.0 {
            self.combat_maneuver_timer = rng.gen_range(1.0..3.0);
            
            if distance_to_player < close_range_threshold {
                self.combat_maneuver_type = 5;
                self.close_range_angle = rng.gen_range(0.0..PI * 2.0);
            } else if distance_to_player < preferred_combat_distance * 0.7 {
                self.combat_maneuver_type = rng.gen_range(1..=4);
            } else if distance_to_player > preferred_combat_distance * 1.5 {
                self.combat_maneuver_type = 0;
            } else {
                self.combat_maneuver_type = rng.gen_range(0..=3);
            }
        }
        
        let to_player = player_pos - *pos;
        let to_player_normalized = to_player.normalize_or_zero();
        
        let height_diff = player_pos.y - pos.y;
        let vertical_tracking = height_diff.clamp(-self.speed * 0.5, self.speed * 0.5);
        
        let base_movement = match self.combat_maneuver_type {
            0 => {
                // Zigzag approach
                self.zigzag_phase += dt * 1.5;
                
                if self.zigzag_phase % (PI * 2.0) < 0.1 {
                    self.zigzag_direction *= -1.0;
                }
                
                let perpendicular = Vec3::new(-to_player_normalized.z, 0.0, to_player_normalized.x);
                let zigzag_amplitude = 150.0;
                let zigzag_offset = perpendicular * (self.zigzag_direction * zigzag_amplitude);
                
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
                // Circle
                let tangent = Vec3::new(-to_player.z, 0.0, to_player.x).normalize_or_zero();
                let radius_correction = if distance_to_player < preferred_combat_distance {
                    -to_player_normalized * 0.3
                } else {
                    to_player_normalized * 0.3
                };
                tangent + radius_correction
            },
            4 => {
                // Retreat
                let retreat = -to_player_normalized;
                let dodge = Vec3::new(
                    (self.zigzag_phase * 2.0).sin() * 0.5,
                    0.0,
                    (self.zigzag_phase * 2.0).cos() * 0.5
                );
                retreat + dodge
            },
            5 => {
                // Close range random
                self.close_range_angle += rng.gen_range(-1.0..1.0) * dt * 3.0;
                let random_dir = Vec3::new(
                    self.close_range_angle.cos(),
                    0.0,
                    self.close_range_angle.sin()
                );
                random_dir * 0.8 + to_player_normalized * 0.2
            },
            _ => to_player_normalized,
        };
        
        let mut movement = base_movement * self.speed * aggression;
        movement.y = vertical_tracking + (self.zigzag_phase * 0.8).sin() * 10.0;
        
        movement
    }
    
    fn update_stuck_detection(&mut self, pos: &Vec3, vel: &mut Vec3, dt: f32) {
        let movement_distance = (*pos - self.last_position).length();
        if movement_distance < self.stuck_threshold * dt && vel.length() > 10.0 {
            self.stuck_timer += dt;
            
            if self.stuck_timer > 2.0 {
                match self.movement_pattern {
                    MovementPattern::Patrol => {
                        self.current_waypoint = (self.current_waypoint + 1) % self.patrol_waypoints.len();
                        self.stuck_timer = 0.0;
                    },
                    _ => {
                        let mut rng = thread_rng();
                        vel.x += rng.gen_range(-50.0..50.0);
                        vel.z += rng.gen_range(-50.0..50.0);
                        vel.y += 20.0;
                        self.stuck_timer = 0.0;
                    }
                }
            }
        } else {
            self.stuck_timer = 0.0;
        }
        
        self.last_position = *pos;
    }
}