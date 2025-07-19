use glam::Vec3;
use rand::prelude::*;
use std::f32::consts::PI;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use crate::terrain;
use super::{Enemy, EnemyType, BaseEnemyData, AlertState, MovementPattern};
use super::ai::AIController;
use super::movement::MovementController;

// Swarm Enemy - Small, fast, moves in groups
pub struct SwarmEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl SwarmEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + rand::random::<f32>() * 50.0 + 20.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Swarm, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Flocking, 450.0, base.spawn_point);
        
        Self { base, ai, movement }
    }
}

impl Enemy for SwarmEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Swarm }
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
        
        None // Swarm enemies don't shoot
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

impl Drawable for SwarmEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        let size = 15.0;
        renderer.draw_pyramid(self.base.pos, size, rotation);
        
        // Add small wings
        let wing_rotation = rotation_matrix(self.base.phase * 8.0, 0.0, 0.0);
        renderer.draw_pyramid(self.base.pos + Vec3::new(10.0, 0.0, 0.0), size * 0.5, wing_rotation);
        renderer.draw_pyramid(self.base.pos - Vec3::new(10.0, 0.0, 0.0), size * 0.5, wing_rotation);
    }
}

// Phaser Enemy - Teleporting sniper
pub struct PhaserEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    teleport_charge: f32,
    special_cooldown: f32,
}

impl PhaserEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 100.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Phaser, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Teleport, 0.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            teleport_charge: 0.0,
            special_cooldown: 0.0,
        }
    }
}

impl Enemy for PhaserEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Phaser }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        // Handle teleportation
        self.teleport_charge += dt;
        
        if self.teleport_charge > 2.0 {
            // Teleport to new position
            let angle = rand::random::<f32>() * PI * 2.0;
            let distance = 500.0 + rand::random::<f32>() * 500.0;
            
            self.base.pos.x = player_pos.x + angle.cos() * distance;
            self.base.pos.z = player_pos.z + angle.sin() * distance;
            
            let terrain_height = terrain::height_at(self.base.pos.x, self.base.pos.z);
            self.base.pos.y = terrain_height + 100.0;
            
            self.teleport_charge = 0.0;
            self.special_cooldown = 1.0;
        }
        
        self.base.vel = Vec3::ZERO; // No regular movement
        
        // Fire after teleport
        if self.special_cooldown > 0.0 {
            self.special_cooldown -= dt;
            if self.special_cooldown <= 0.0 && self.teleport_charge < 0.5 {
                let to_player = player_pos - self.base.pos;
                return Some(to_player.normalize_or_zero());
            }
        }
        
        None
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

impl Drawable for PhaserEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Diamond shape with energy rings
        let charge_scale = 1.0 + (self.teleport_charge * 0.5).min(1.0);
        renderer.draw_octahedron(self.base.pos, 30.0 * charge_scale, rotation);
        
        // Energy rings that expand when charging
        for i in 0..3 {
            let ring_scale = 1.0 + (self.teleport_charge + i as f32 * 0.3).sin() * 0.3;
            let phase_wrapped = self.base.phase % (2.0 * PI);
            let ring_rotation = rotation_matrix(
                phase_wrapped * (i + 1) as f32,
                phase_wrapped * 0.5,
                0.0
            );
            renderer.draw_cube(self.base.pos, 40.0 * ring_scale, ring_rotation);
        }
    }
}

// Shield Enemy - Creates protective shields
pub struct ShieldEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    shield_radius: f32,
}

impl ShieldEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 60.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Shield, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Stationary, 100.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            shield_radius: 200.0,
        }
    }
}

impl Enemy for ShieldEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Shield }
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
        
        None // Shield enemies don't attack
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn get_shield_active(&self) -> bool { true }
    fn get_shield_radius(&self) -> f32 { self.shield_radius }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for ShieldEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Central orb
        renderer.draw_octahedron(self.base.pos, 25.0, rotation);
        
        // Rotating shield panels
        for i in 0..6 {
            let phase_wrapped = self.base.phase % (2.0 * PI);
            let angle = i as f32 * PI / 3.0 + phase_wrapped;
            let panel_pos = self.base.pos + Vec3::new(
                angle.cos() * 40.0,
                (angle * 2.0).sin() * 10.0,
                angle.sin() * 40.0
            );
            let panel_rotation = rotation_matrix(angle % (2.0 * PI), phase_wrapped, 0.0);
            renderer.draw_cube(panel_pos, 15.0, panel_rotation);
        }
    }
}

// Bomber Enemy - Drops mines
pub struct BomberEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    mine_count: i32,
    special_cooldown: f32,
}

impl BomberEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 150.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Bomber, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Drifting, 150.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            mine_count: 5,
            special_cooldown: 0.0,
        }
    }
}

impl Enemy for BomberEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Bomber }
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
        
        if self.special_cooldown > 0.0 {
            self.special_cooldown -= dt;
        }
        
        let distance_to_player = (player_pos - self.base.pos).length();
        if self.mine_count > 0 && self.special_cooldown <= 0.0 && 
           distance_to_player < self.ai.detection_range {
            self.mine_count -= 1;
            self.special_cooldown = 1.5;
        }
        
        None
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn should_drop_mine(&self) -> bool {
        self.special_cooldown <= 0.01 && self.mine_count > 0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for BomberEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Large sphere with spikes
        renderer.draw_octahedron(self.base.pos, 40.0, rotation);
        
        // Spike indicators for remaining mines
        for i in 0..self.mine_count {
            let angle = i as f32 * PI * 2.0 / 5.0;
            let spike_offset = Vec3::new(
                angle.cos() * 30.0,
                0.0,
                angle.sin() * 30.0
            );
            renderer.draw_pyramid(self.base.pos + spike_offset, 15.0, rotation);
        }
    }
}

// Disruptor Enemy - Emits slowing waves
pub struct DisruptorEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    wave_charge: f32,
}

impl DisruptorEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 80.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Disruptor, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Hover, 175.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            wave_charge: 0.0,
        }
    }
}

impl Enemy for DisruptorEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Disruptor }
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
        
        let distance_to_player = (player_pos - self.base.pos).length();
        if distance_to_player < self.ai.detection_range {
            self.wave_charge += dt;
            
            if self.wave_charge > 2.0 {
                self.wave_charge = 0.0;
            }
        }
        
        None
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn get_wave_active(&self) -> bool { self.wave_charge > 2.0 }
    fn get_wave_radius(&self) -> f32 {
        if self.get_wave_active() {
            (self.wave_charge - 2.0) * 200.0
        } else {
            0.0
        }
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for DisruptorEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Twisted spiral shape
        let twist = self.base.phase * 2.0;
        for i in 0..5 {
            let height_offset = (i as f32 - 2.0) * 10.0;
            let twist_angle = twist + i as f32 * 0.5;
            let ring_pos = self.base.pos + Vec3::new(0.0, height_offset, 0.0);
            let ring_rotation = rotation_matrix(twist_angle, 0.0, 0.0);
            renderer.draw_cube(ring_pos, 30.0 - i as f32 * 3.0, ring_rotation);
        }
        
        // Charging effect
        if self.wave_charge > 0.0 {
            let charge_size = 50.0 * (self.wave_charge / 2.0).min(1.0);
            renderer.draw_octahedron(self.base.pos, charge_size, rotation);
        }
    }
}

// Carrier Enemy - Spawns swarm enemies
pub struct CarrierEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    spawn_timer: f32,
}

impl CarrierEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 150.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Carrier, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Drifting, 80.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            spawn_timer: 0.0,
        }
    }
}

impl Enemy for CarrierEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Carrier }
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
        
        self.spawn_timer += dt;
        if self.spawn_timer > 4.0 {
            self.spawn_timer = 0.0;
        }
        
        None
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn should_spawn_swarm(&self) -> bool { self.spawn_timer >= 4.0 }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for CarrierEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Large hexagonal platform
        renderer.draw_hexagon_prism(self.base.pos, 60.0, 20.0, rotation);
        
        // Hangar bays
        for i in 0..4 {
            let angle = i as f32 * PI / 2.0;
            let bay_offset = Vec3::new(
                angle.cos() * 45.0,
                -10.0,
                angle.sin() * 45.0
            );
            renderer.draw_cube(self.base.pos + bay_offset, 20.0, rotation);
        }
        
        // Spawn indicator
        if self.spawn_timer > 3.0 {
            let blink = ((self.spawn_timer - 3.0) * 10.0).sin();
            if blink > 0.0 {
                renderer.draw_octahedron(self.base.pos - Vec3::new(0.0, 20.0, 0.0), 15.0, rotation);
            }
        }
    }
}

// Reflector Enemy - Reflects bullets
pub struct ReflectorEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    reflection_active: bool,
}

impl ReflectorEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 60.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Reflector, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Orbital, 215.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            reflection_active: true,
        }
    }
}

impl Enemy for ReflectorEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Reflector }
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
        
        // Always face player for reflection
        let to_player = player_pos - self.base.pos;
        let angle_to_player = to_player.z.atan2(to_player.x);
        self.base.rotation.y = angle_to_player;
        
        None
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn is_reflecting(&self) -> bool { self.reflection_active }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for ReflectorEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Crystalline shape with mirror facets
        let facet_rotation = rotation_matrix(self.base.rotation.y, 0.0, 0.0);
        
        // Main crystal
        renderer.draw_octahedron(self.base.pos, 35.0, facet_rotation);
        
        // Mirror panels
        for i in 0..8 {
            let angle = i as f32 * PI / 4.0;
            let panel_offset = Vec3::new(
                angle.cos() * 25.0,
                0.0,
                angle.sin() * 25.0
            );
            let panel_rotation = rotation_matrix(self.base.rotation.y + angle, PI / 4.0, 0.0);
            renderer.draw_pyramid(self.base.pos + panel_offset, 15.0, panel_rotation);
        }
    }
}

// Vortex Enemy - Creates gravity well
pub struct VortexEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
    vortex_strength: f32,
}

impl VortexEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = terrain::height_at(x, z) + 60.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Vortex, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Stationary, 0.0, base.spawn_point);
        
        Self {
            base,
            ai,
            movement,
            vortex_strength: 300.0,
        }
    }
}

impl Enemy for VortexEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Vortex }
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
        
        // Pulsing vortex strength
        self.vortex_strength = 300.0 + (self.base.phase * 2.0).sin() * 100.0;
        
        None
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn get_vortex_strength(&self) -> f32 { self.vortex_strength }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for VortexEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Swirling energy spiral
        let vortex_speed = self.base.phase * 3.0;
        
        // Central core
        renderer.draw_octahedron(self.base.pos, 20.0, rotation);
        
        // Swirling rings
        for i in 0..8 {
            let height = (i as f32 - 4.0) * 15.0;
            let ring_angle = vortex_speed + i as f32 * 0.5;
            let ring_size = 30.0 + i as f32 * 5.0;
            
            // Draw partial ring to show swirl
            for j in 0..3 {
                let segment_angle = ring_angle + j as f32 * 2.0 * PI / 3.0;
                let segment_offset = Vec3::new(
                    segment_angle.cos() * ring_size,
                    height,
                    segment_angle.sin() * ring_size
                );
                let ring_rotation = rotation_matrix(ring_angle, 0.0, 0.0);
                renderer.draw_cube(self.base.pos + segment_offset, 10.0, ring_rotation);
            }
        }
    }
}