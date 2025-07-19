use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use crate::terrain;
use super::{Enemy, EnemyType, BaseEnemyData, AlertState, MovementPattern};
use super::ai::AIController;
use super::movement::MovementController;

// Hunter Enemy - Aggressive tracking enemy
pub struct HunterEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl HunterEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Hunter, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Tracking, 350.0, base.spawn_point);
        
        Self { base, ai, movement }
    }
}

impl Enemy for HunterEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Hunter }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        self.movement.update(
            &mut self.base.pos,
            &mut self.base.vel,
            self.base.spawn_point,
            self.base.phase,
            player_pos,
            self.ai.alert_state,
            self.ai.last_known_player_pos,
            self.ai.investigation_point,
            self.ai.aggression,
            self.ai.preferred_combat_distance,
            dt
        );
        
        if self.ai.should_attack(&EnemyType::Hunter) {
            let to_player = player_pos - self.base.pos;
            Some(to_player.normalize_or_zero())
        } else {
            None
        }
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for HunterEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Spike ball design
        renderer.draw_spike_ball(self.base.pos, 25.0, 20.0, rotation);
        
        // Add pulsing core
        let pulse = (self.base.phase * 3.0).sin() * 0.2 + 0.8;
        let core_rotation = rotation_matrix(
            self.base.rotation.y * -1.0,
            self.base.rotation.x * -1.0,
            0.0
        );
        renderer.draw_octahedron(self.base.pos, 15.0 * pulse, core_rotation);
    }
}

// Guardian Enemy - Defensive patrol enemy
pub struct GuardianEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl GuardianEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Guardian, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Patrol, 200.0, base.spawn_point);
        
        // Generate default patrol waypoints
        let mut waypoints = Vec::new();
        let patrol_radius = 2000.0;
        for i in 0..6 {
            let angle = i as f32 * PI * 2.0 / 6.0 + rand::random::<f32>() * 0.5 - 0.25;
            let radius = patrol_radius * rand::random::<f32>() * 0.3 + 0.7;
            let wx = x + angle.cos() * radius;
            let wz = z + angle.sin() * radius;
            let wy = terrain::height_at(wx, wz) + rand::random::<f32>() * 150.0 + 50.0;
            waypoints.push(Vec3::new(wx, wy, wz));
        }
        
        let mut enemy = Self { base, ai, movement };
        enemy.movement.set_patrol_route(waypoints);
        enemy
    }
}

impl Enemy for GuardianEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Guardian }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        self.movement.update(
            &mut self.base.pos,
            &mut self.base.vel,
            self.base.spawn_point,
            self.base.phase,
            player_pos,
            self.ai.alert_state,
            self.ai.last_known_player_pos,
            self.ai.investigation_point,
            self.ai.aggression,
            self.ai.preferred_combat_distance,
            dt
        );
        
        if self.ai.should_attack(&EnemyType::Guardian) {
            let to_player = player_pos - self.base.pos;
            Some(to_player.normalize_or_zero())
        } else {
            None
        }
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for GuardianEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Hexagonal fortress
        renderer.draw_hexagon_prism(self.base.pos, 40.0, 30.0, rotation);
        
        // Add rotating shields
        let shield_rotation = rotation_matrix(self.base.phase * 0.5, 0.0, 0.0);
        renderer.draw_hexagon_prism(self.base.pos, 50.0, 15.0, shield_rotation);
        
        // Central core
        renderer.draw_octahedron(self.base.pos, 20.0, rotation);
    }
}

// Laser Enemy - Long-range laser attack
pub struct LaserEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    laser_active: bool,
    laser_angle_h: f32,
    laser_angle_v: f32,
    laser_target_angle_h: f32,
    laser_target_angle_v: f32,
    laser_duration: f32,
}

impl LaserEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Laser, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Hover, 125.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            laser_active: false,
            laser_angle_h: 0.0,
            laser_angle_v: 0.0,
            laser_target_angle_h: 0.0,
            laser_target_angle_v: 0.0,
            laser_duration: 0.0,
        }
    }
}

impl Enemy for LaserEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Laser }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        // Special laser movement - slow approach
        let terrain_height = terrain::height_at(self.base.pos.x, self.base.pos.z);
        let hover_height = terrain_height + 80.0 + (self.base.phase * 0.3).sin() * 20.0;
        
        let height_diff = hover_height - self.base.pos.y;
        self.base.vel.y = height_diff * 2.0;
        
        let to_player = player_pos - self.base.pos;
        let distance_to_player = to_player.length();
        
        if distance_to_player > 800.0 {
            let horizontal = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
            self.base.vel.x = horizontal.x * self.movement.speed * 0.5;
            self.base.vel.z = horizontal.z * self.movement.speed * 0.5;
        } else {
            let orbit_angle = self.base.phase * 0.2;
            self.base.vel.x = orbit_angle.cos() * self.movement.speed * 0.3;
            self.base.vel.z = orbit_angle.sin() * self.movement.speed * 0.3;
        }
        
        self.base.pos += self.base.vel * dt;
        
        // Ensure enemy doesn't go below terrain
        if self.base.pos.y < terrain_height + 10.0 {
            self.base.pos.y = terrain_height + 10.0;
            self.base.vel.y = self.base.vel.y.max(0.0);
        }
        
        // Handle laser attack
        if self.laser_active {
            self.laser_duration += dt;
            
            let angle_diff_h = self.laser_target_angle_h - self.laser_angle_h;
            let angle_diff_v = self.laser_target_angle_v - self.laser_angle_v;
            let sweep_speed = 0.8;
            
            self.laser_angle_h += angle_diff_h.signum() * sweep_speed * dt;
            self.laser_angle_v += angle_diff_v.signum() * sweep_speed * dt;
            
            if self.laser_duration > 3.0 || (angle_diff_h.abs() < 0.1 && angle_diff_v.abs() < 0.1) {
                self.laser_active = false;
                self.ai.attack_cooldown = 4.0;
            }
        } else if distance_to_player < self.ai.detection_range && self.ai.attack_cooldown <= 0.0 {
            if rand::random::<f32>() < 0.1 * self.ai.aggression {
                self.laser_active = true;
                self.laser_duration = 0.0;
                
                let horizontal_dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
                let angle_to_player_h = to_player.z.atan2(to_player.x);
                let angle_to_player_v = to_player.y.atan2(horizontal_dist);
                
                let start_offset_h = if rand::random::<bool>() { -1.2 } else { 1.2 };
                let start_offset_v = rand::random::<f32>() - 0.5;
                
                self.laser_angle_h = angle_to_player_h + start_offset_h;
                self.laser_angle_v = angle_to_player_v + start_offset_v;
                
                self.laser_target_angle_h = angle_to_player_h + start_offset_h * -0.3;
                self.laser_target_angle_v = angle_to_player_v - start_offset_v * 0.5;
            }
        }
        
        None // Laser enemies don't shoot bullets
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn is_laser_active(&self) -> bool { self.laser_active }
    
    fn get_laser_info(&self) -> Option<(Vec3, Vec3)> {
        if self.laser_active {
            let start = self.base.pos - Vec3::new(0.0, 20.0, 0.0);
            let laser_length = 1500.0;
            
            let horizontal_component = self.laser_angle_v.cos() * laser_length;
            let vertical_component = self.laser_angle_v.sin() * laser_length;
            
            let end_x = self.base.pos.x + self.laser_angle_h.cos() * horizontal_component;
            let end_z = self.base.pos.z + self.laser_angle_h.sin() * horizontal_component;
            let end_y = self.base.pos.y + vertical_component;
            
            let end = Vec3::new(end_x, end_y, end_z);
            Some((start, end))
        } else {
            None
        }
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for LaserEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // High-tech appearance
        renderer.draw_hexagon_prism(self.base.pos, 35.0, 40.0, rotation);
        
        // Energy core that glows when laser is active
        let energy_scale = if self.laser_active { 1.5 } else { 1.0 };
        let core_rotation = rotation_matrix(self.base.phase * 5.0, self.base.phase * 3.0, 0.0);
        renderer.draw_octahedron(self.base.pos, 20.0 * energy_scale, core_rotation);
        
        // Rotating rings
        let ring_rotation = rotation_matrix(
            self.base.rotation.y * 3.0,
            self.base.rotation.x * 2.0,
            0.0
        );
        renderer.draw_cube(self.base.pos + Vec3::new(0.0, 20.0, 0.0), 25.0, ring_rotation);
        renderer.draw_cube(self.base.pos - Vec3::new(0.0, 20.0, 0.0), 25.0, ring_rotation);
    }
}