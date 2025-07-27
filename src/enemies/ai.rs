use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AlertState {
    Unaware,     // Normal patrol behavior
    Suspicious,  // Heard something, investigating
    Alert,       // Saw player, engaging
    Searching,   // Lost player, searching area
    Returning,   // Returning to patrol
}

pub struct AIController {
    pub alert_state: AlertState,
    pub last_known_player_pos: Vec3,
    pub alert_cooldown: f32,
    pub search_timer: f32,
    pub investigation_point: Vec3,
    pub communication_range: f32,
    pub view_cone_angle: f32,
    pub hearing_range: f32,
    pub detection_range: f32,
    
    // Combat decision making
    pub aggression: f32,
    pub preferred_combat_distance: f32,
    pub can_attack: bool,
    pub attack_cooldown: f32,
}

impl AIController {
    pub fn new(enemy_type: &super::EnemyType, spawn_pos: Vec3) -> Self {
        let mut rng = thread_rng();
        
        // Set AI parameters based on enemy type
        let (aggression, detection_range, view_angle, hearing_range, comm_range, can_attack, combat_distance) = 
            match enemy_type {
                super::EnemyType::Cube => (0.5, 1500.0, PI * 0.7, 800.0, 1200.0, false, 350.0),
                super::EnemyType::Pyramid => (0.8, 2500.0, PI * 0.6, 1200.0, 1800.0, true, 300.0),
                super::EnemyType::Spinner => (0.6, 2000.0, PI * 0.8, 1000.0, 1500.0, false, 350.0),
                super::EnemyType::Hunter => (0.95, 4000.0, PI * 0.5, 2000.0, 2500.0, true, 400.0),
                super::EnemyType::Guardian => (0.6, 1800.0, PI * 0.9, 1500.0, 2000.0, true, 500.0),
                super::EnemyType::Laser => (0.7, 3000.0, PI * 0.4, 1500.0, 2000.0, false, 600.0),
                super::EnemyType::Swarm => (0.9, 1200.0, PI, 800.0, 3000.0, false, 200.0),
                super::EnemyType::Phaser => (0.8, 5000.0, PI * 0.4, 1800.0, 1500.0, false, 800.0),
                super::EnemyType::Shield => (0.3, 2000.0, PI * 1.5, 1200.0, 2500.0, false, 0.0),
                super::EnemyType::Bomber => (0.5, 2200.0, PI * 0.7, 1000.0, 1500.0, false, 450.0),
                super::EnemyType::Disruptor => (0.6, 2500.0, PI * 0.8, 1400.0, 1800.0, false, 400.0),
                super::EnemyType::Carrier => (0.4, 3000.0, PI * 0.9, 1500.0, 3000.0, false, 600.0),
                super::EnemyType::Reflector => (0.7, 2000.0, PI * 0.6, 1200.0, 1500.0, false, 350.0),
                super::EnemyType::Vortex => (0.0, 1500.0, PI * 2.0, 2000.0, 2500.0, false, 0.0),
            };
        
        Self {
            alert_state: AlertState::Unaware,
            last_known_player_pos: spawn_pos,
            alert_cooldown: 0.0,
            search_timer: 0.0,
            investigation_point: spawn_pos,
            communication_range: comm_range,
            view_cone_angle: view_angle,
            hearing_range,
            detection_range,
            aggression: aggression + rng.gen_range(-0.1..0.1),
            preferred_combat_distance: combat_distance * rng.gen_range(0.8..1.2),
            can_attack,
            attack_cooldown: 0.0,
        }
    }
    
    pub fn update(&mut self, enemy_pos: Vec3, enemy_vel: Vec3, player_pos: Vec3, dt: f32) {
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();
        
        // Update alert cooldown
        if self.alert_cooldown > 0.0 {
            self.alert_cooldown -= dt;
        }
        
        // Update attack cooldown
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        
        match self.alert_state {
            AlertState::Unaware => {
                if self.can_see_player(enemy_pos, enemy_vel, player_pos, distance) {
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0;
                    // println!("Enemy spotted player!");
                } else if self.can_hear_player(distance) {
                    self.alert_state = AlertState::Suspicious;
                    self.investigation_point = player_pos;
                    self.alert_cooldown = 3.0;
                }
            }
            
            AlertState::Suspicious => {
                let dist_to_investigate = (self.investigation_point - enemy_pos).length();
                
                if self.can_see_player(enemy_pos, enemy_vel, player_pos, distance) {
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0;
                } else if dist_to_investigate < 50.0 || self.alert_cooldown <= 0.0 {
                    self.alert_state = AlertState::Returning;
                    self.alert_cooldown = 2.0;
                }
            }
            
            AlertState::Alert => {
                if self.can_see_player(enemy_pos, enemy_vel, player_pos, distance) {
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 5.0;
                } else if self.alert_cooldown <= 0.0 {
                    self.alert_state = AlertState::Searching;
                    self.search_timer = 0.0;
                }
            }
            
            AlertState::Searching => {
                self.search_timer += dt;
                
                if self.can_see_player(enemy_pos, enemy_vel, player_pos, distance) {
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0;
                    self.search_timer = 0.0;
                } else if self.search_timer > 30.0 {
                    self.alert_state = AlertState::Returning;
                    self.alert_cooldown = 2.0;
                }
            }
            
            AlertState::Returning => {
                if self.can_see_player(enemy_pos, enemy_vel, player_pos, distance) {
                    self.alert_state = AlertState::Alert;
                    self.last_known_player_pos = player_pos;
                    self.alert_cooldown = 10.0;
                }
            }
        }
    }
    
    pub fn should_attack(&mut self, enemy_type: &super::EnemyType) -> bool {
        if self.alert_state != AlertState::Alert || !self.can_attack || self.attack_cooldown > 0.0 {
            return false;
        }
        
        let attack_chance = match enemy_type {
            super::EnemyType::Pyramid => 0.04,
            super::EnemyType::Hunter => 0.06,
            super::EnemyType::Guardian => 0.03,
            _ => 0.0,
        };
        
        if rand::random::<f32>() < attack_chance * self.aggression {
            self.attack_cooldown = match enemy_type {
                super::EnemyType::Pyramid => 0.75,
                super::EnemyType::Hunter => 0.5,
                super::EnemyType::Guardian => 1.0,
                _ => 1.0,
            };
            true
        } else {
            false
        }
    }
    
    fn can_see_player(&self, enemy_pos: Vec3, enemy_vel: Vec3, player_pos: Vec3, distance: f32) -> bool {
        if distance > self.detection_range {
            return false;
        }
        
        let to_player = (player_pos - enemy_pos).normalize_or_zero();
        if to_player.length() == 0.0 {
            return false;
        }
        
        let forward = if enemy_vel.length() > 0.1 {
            enemy_vel.normalize()
        } else {
            Vec3::new(0.0, 0.0, 1.0) // Default forward
        };
        
        let dot = forward.dot(to_player);
        let angle = dot.acos();
        
        angle <= self.view_cone_angle / 2.0
    }
    
    fn can_hear_player(&self, distance: f32) -> bool {
        distance <= self.hearing_range
    }
}

// Function to alert nearby enemies (called from game.rs)
pub fn alert_nearby_enemies(alerting_enemy_pos: Vec3, alerting_enemy_comm_range: f32, 
                           last_known_pos: Vec3, all_enemies: &mut [impl AlertStateGetter]) {
    for other in all_enemies.iter_mut() {
        let distance = (other.get_position() - alerting_enemy_pos).length();
        if distance <= alerting_enemy_comm_range && other.get_alert_state() == AlertState::Unaware {
            other.set_suspicious(last_known_pos);
        }
    }
}

// Trait for enemies to implement alert state access
pub trait AlertStateGetter {
    fn get_position(&self) -> Vec3;
    fn get_alert_state(&self) -> AlertState;
    fn set_suspicious(&mut self, investigation_point: Vec3);
}