// Re-export all enemy-related types and traits
pub mod basic_enemies;
pub mod advanced_enemies;
pub mod special_enemies;
pub mod movement;
pub mod ai;
pub mod core;
pub mod factory;

use glam::Vec3;
use crate::renderer::Drawable;

// Re-export commonly used types
pub use self::ai::{AlertState, alert_nearby_enemies};
pub use self::movement::{MovementPattern, MovementController};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyType {
    // Basic enemies
    Cube,      // Basic enemy that follows terrain
    Pyramid,   // Fast enemy that dives and climbs
    Spinner,   // Orbiting enemy with complex patterns
    
    // Advanced enemies
    Hunter,    // Tracks player position
    Guardian,  // Patrols specific areas
    Laser,     // Enemy with sweeping laser attack
    
    // Special enemies
    Swarm,     // Small enemies that move in groups
    Phaser,    // Teleporting sniper
    Shield,    // Creates shields for other enemies
    Bomber,    // Drops explosive mines
    Disruptor, // Emits slowing waves
    Carrier,   // Spawns swarm enemies
    Reflector, // Reflects bullets back
    Vortex,    // Creates gravity well
}

// Common enemy trait that all enemy types must implement
pub trait Enemy: Drawable + Send + Sync {
    fn get_type(&self) -> EnemyType;
    fn get_position(&self) -> Vec3;
    fn set_position(&mut self, pos: Vec3);
    fn get_velocity(&self) -> Vec3;
    fn set_velocity(&mut self, vel: Vec3);
    
    // Update returns optional attack direction
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3>;
    
    // Common getters
    fn get_alert_state(&self) -> AlertState;
    fn get_health(&self) -> f32;
    fn take_damage(&mut self, damage: f32) -> bool; // Returns true if destroyed
    
    // Special ability checks
    fn is_laser_active(&self) -> bool { false }
    fn get_laser_info(&self) -> Option<(Vec3, Vec3)> { None }
    fn get_shield_active(&self) -> bool { false }
    fn get_shield_radius(&self) -> f32 { 0.0 }
    fn should_drop_mine(&self) -> bool { false }
    fn get_wave_active(&self) -> bool { false }
    fn get_wave_radius(&self) -> f32 { 0.0 }
    fn should_spawn_swarm(&self) -> bool { false }
    fn is_reflecting(&self) -> bool { false }
    fn get_vortex_strength(&self) -> f32 { 0.0 }
    
    // Patrol routes
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>);
}

// Base enemy data that all enemies share
#[derive(Clone)]
pub struct BaseEnemyData {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rotation: Vec3,
    pub rotation_speed: Vec3,
    pub spawn_point: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub phase: f32, // Animation phase
}

impl BaseEnemyData {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            vel: Vec3::ZERO,
            rotation: Vec3::ZERO,
            rotation_speed: Vec3::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            ),
            spawn_point: Vec3::new(x, y, z),
            health: 100.0,
            max_health: 100.0,
            phase: rand::random::<f32>() * std::f32::consts::PI * 2.0,
        }
    }
    
    pub fn update_rotation(&mut self, dt: f32) {
        use std::f32::consts::PI;
        
        self.rotation.x = (self.rotation.x + self.rotation_speed.x * dt) % (2.0 * PI);
        self.rotation.y = (self.rotation.y + self.rotation_speed.y * dt) % (2.0 * PI);
        self.rotation.z = (self.rotation.z + self.rotation_speed.z * dt) % (2.0 * PI);
        self.phase += dt;
        
        // Wrap phase to prevent overflow
        if self.phase > 1000.0 {
            self.phase = self.phase % (2.0 * PI);
        }
    }
}

// Helper function to create enemies by type
pub fn create_enemy(x: f32, z: f32, enemy_type: EnemyType) -> Box<dyn Enemy> {
    match enemy_type {
        EnemyType::Cube => Box::new(basic_enemies::CubeEnemy::new(x, z)),
        EnemyType::Pyramid => Box::new(basic_enemies::PyramidEnemy::new(x, z)),
        EnemyType::Spinner => Box::new(basic_enemies::SpinnerEnemy::new(x, z)),
        
        EnemyType::Hunter => Box::new(advanced_enemies::HunterEnemy::new(x, z)),
        EnemyType::Guardian => Box::new(advanced_enemies::GuardianEnemy::new(x, z)),
        EnemyType::Laser => Box::new(advanced_enemies::LaserEnemy::new(x, z)),
        
        EnemyType::Swarm => Box::new(special_enemies::SwarmEnemy::new(x, z)),
        EnemyType::Phaser => Box::new(special_enemies::PhaserEnemy::new(x, z)),
        EnemyType::Shield => Box::new(special_enemies::ShieldEnemy::new(x, z)),
        EnemyType::Bomber => Box::new(special_enemies::BomberEnemy::new(x, z)),
        EnemyType::Disruptor => Box::new(special_enemies::DisruptorEnemy::new(x, z)),
        EnemyType::Carrier => Box::new(special_enemies::CarrierEnemy::new(x, z)),
        EnemyType::Reflector => Box::new(special_enemies::ReflectorEnemy::new(x, z)),
        EnemyType::Vortex => Box::new(special_enemies::VortexEnemy::new(x, z)),
    }
}