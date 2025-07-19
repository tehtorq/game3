use glam::Vec3;
use crate::renderer::{Renderer, Drawable};

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

/// Base trait for all enemy types
pub trait EnemyBehavior: Drawable {
    fn update(&mut self, player_pos: Vec3, dt: f32) -> EnemyUpdateResult;
    fn get_position(&self) -> Vec3;
    fn get_enemy_type(&self) -> EnemyType;
    fn get_alert_state(&self) -> AlertState;
    fn take_damage(&mut self, damage: f32) -> bool; // Returns true if destroyed
    fn is_alive(&self) -> bool;
}

/// Result of enemy update containing any actions to take
pub struct EnemyUpdateResult {
    pub bullets_to_spawn: Vec<BulletSpawnRequest>,
    pub mines_to_spawn: Vec<Vec3>,
    pub enemies_to_spawn: Vec<EnemySpawnRequest>,
    pub particles_to_spawn: Vec<ParticleSpawnRequest>,
}

impl Default for EnemyUpdateResult {
    fn default() -> Self {
        Self {
            bullets_to_spawn: Vec::new(),
            mines_to_spawn: Vec::new(),
            enemies_to_spawn: Vec::new(),
            particles_to_spawn: Vec::new(),
        }
    }
}

pub struct BulletSpawnRequest {
    pub position: Vec3,
    pub direction: Vec3,
    pub bullet_type: crate::bullet::BulletType,
}

pub struct EnemySpawnRequest {
    pub position: Vec3,
    pub enemy_type: EnemyType,
}

pub struct ParticleSpawnRequest {
    pub position: Vec3,
    pub particle_type: crate::particle::ParticleType,
    pub count: u32,
}

/// Common enemy configuration
pub struct EnemyConfig {
    pub speed: f32,
    pub health: f32,
    pub aggression: f32,
    pub detection_range: f32,
    pub attack_cooldown: f32,
    pub can_attack: bool,
    pub view_cone_angle: f32,
    pub hearing_range: f32,
    pub communication_range: f32,
}

impl EnemyConfig {
    pub fn for_type(enemy_type: EnemyType) -> Self {
        use rand::prelude::*;
        let mut rng = thread_rng();
        
        match enemy_type {
            EnemyType::Cube => Self {
                speed: rng.gen_range(200.0..300.0),
                health: 1.0,
                aggression: 0.3,
                detection_range: 1500.0,
                attack_cooldown: 2.0,
                can_attack: true,
                view_cone_angle: 60.0_f32.to_radians(),
                hearing_range: 2000.0,
                communication_range: 1000.0,
            },
            EnemyType::Pyramid => Self {
                speed: rng.gen_range(350.0..450.0),
                health: 1.0,
                aggression: 0.5,
                detection_range: 2000.0,
                attack_cooldown: 1.5,
                can_attack: true,
                view_cone_angle: 90.0_f32.to_radians(),
                hearing_range: 2500.0,
                communication_range: 1500.0,
            },
            EnemyType::Hunter => Self {
                speed: rng.gen_range(300.0..400.0),
                health: 2.0,
                aggression: 0.8,
                detection_range: 3000.0,
                attack_cooldown: 1.0,
                can_attack: true,
                view_cone_angle: 120.0_f32.to_radians(),
                hearing_range: 4000.0,
                communication_range: 2000.0,
            },
            EnemyType::Guardian => Self {
                speed: rng.gen_range(150.0..250.0),
                health: 3.0,
                aggression: 0.4,
                detection_range: 2500.0,
                attack_cooldown: 2.5,
                can_attack: true,
                view_cone_angle: 180.0_f32.to_radians(),
                hearing_range: 3000.0,
                communication_range: 1500.0,
            },
            // Add other enemy types...
            _ => Self {
                speed: 200.0,
                health: 1.0,
                aggression: 0.5,
                detection_range: 1500.0,
                attack_cooldown: 2.0,
                can_attack: true,
                view_cone_angle: 90.0_f32.to_radians(),
                hearing_range: 2000.0,
                communication_range: 1000.0,
            }
        }
    }
}