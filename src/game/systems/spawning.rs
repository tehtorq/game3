use rand::prelude::*;
use glam::Vec3;
use crate::enemy::{Enemy, EnemyType};
use crate::base::{Base, BaseType};

pub struct SpawnEvent {
    pub enemy_type: EnemyType,
    pub position: Vec3,
}

pub struct SpawningSystem;

impl SpawningSystem {
    /// Get enemies to spawn from all active bases
    pub fn get_spawn_events(bases: &mut [Base], wave: u32) -> Vec<SpawnEvent> {
        let mut spawn_events = Vec::new();
        
        for base in bases.iter_mut() {
            if base.is_active && base.can_spawn() {
                let spawns = Self::get_base_spawns(base, wave);
                spawn_events.extend(spawns);
                // Reset the spawn timer after spawning
                base.reset_spawn_timer();
            }
        }
        
        spawn_events
    }
    
    /// Get spawn events for a single base
    pub fn get_base_spawns(base: &Base, wave: u32) -> Vec<SpawnEvent> {
        let mut rng = thread_rng();
        let mut events = Vec::new();
        
        let spawn_count = match base.base_type {
            BaseType::Basic => rng.gen_range(1..=2),
            BaseType::Heavy => rng.gen_range(2..=3),
            BaseType::Shielded => rng.gen_range(1..=3),
            BaseType::Fortress => rng.gen_range(3..=5),
            BaseType::Outpost => 1,
        };
        
        for _ in 0..spawn_count {
            let enemy_type = Self::select_enemy_type(base.base_type, wave, &mut rng);
            let spawn_offset = Vec3::new(
                rng.gen_range(-50.0..50.0),
                0.0,
                rng.gen_range(-50.0..50.0)
            );
            
            events.push(SpawnEvent {
                enemy_type,
                position: base.pos + spawn_offset,
            });
        }
        
        events
    }
    
    /// Select enemy type based on base type and wave
    fn select_enemy_type(base_type: BaseType, wave: u32, rng: &mut impl Rng) -> EnemyType {
        let roll = rng.gen_range(0..100);
        
        match base_type {
            BaseType::Basic => {
                // Basic bases spawn simple enemies
                if wave < 3 {
                    if roll < 70 { EnemyType::Cube } else { EnemyType::Pyramid }
                } else if wave < 6 {
                    if roll < 40 { EnemyType::Cube }
                    else if roll < 70 { EnemyType::Pyramid }
                    else { EnemyType::Spinner }
                } else {
                    if roll < 20 { EnemyType::Cube }
                    else if roll < 40 { EnemyType::Pyramid }
                    else if roll < 60 { EnemyType::Spinner }
                    else if roll < 80 { EnemyType::Hunter }
                    else { EnemyType::Swarm }
                }
            }
            BaseType::Heavy => {
                // Heavy bases spawn tougher enemies
                if wave < 5 {
                    if roll < 50 { EnemyType::Pyramid } else { EnemyType::Spinner }
                } else if wave < 10 {
                    if roll < 30 { EnemyType::Spinner }
                    else if roll < 60 { EnemyType::Hunter }
                    else if roll < 85 { EnemyType::Guardian }
                    else { EnemyType::Laser }
                } else {
                    if roll < 20 { EnemyType::Hunter }
                    else if roll < 40 { EnemyType::Guardian }
                    else if roll < 60 { EnemyType::Laser }
                    else if roll < 80 { EnemyType::Bomber }
                    else { EnemyType::Disruptor }
                }
            }
            BaseType::Shielded => {
                // Shielded bases spawn defensive/support enemies
                if wave < 7 {
                    if roll < 60 { EnemyType::Guardian } else { EnemyType::Shield }
                } else {
                    if roll < 30 { EnemyType::Guardian }
                    else if roll < 50 { EnemyType::Shield }
                    else if roll < 70 { EnemyType::Reflector }
                    else if roll < 90 { EnemyType::Disruptor }
                    else { EnemyType::Phaser }
                }
            }
            BaseType::Fortress => {
                // Fortresses spawn elite enemies
                if wave < 10 {
                    if roll < 40 { EnemyType::Hunter }
                    else if roll < 70 { EnemyType::Laser }
                    else { EnemyType::Bomber }
                } else {
                    if roll < 20 { EnemyType::Laser }
                    else if roll < 35 { EnemyType::Bomber }
                    else if roll < 50 { EnemyType::Phaser }
                    else if roll < 65 { EnemyType::Carrier }
                    else if roll < 80 { EnemyType::Reflector }
                    else if roll < 95 { EnemyType::Vortex }
                    else { EnemyType::Disruptor }
                }
            }
            BaseType::Outpost => {
                // Outposts spawn fast/swarming enemies
                if roll < 40 { EnemyType::Swarm }
                else if roll < 70 { EnemyType::Hunter }
                else if roll < 90 { EnemyType::Phaser }
                else { EnemyType::Spinner }
            }
        }
    }
    
    /// Handle carrier enemy spawns
    pub fn spawn_from_carrier(carrier_pos: Vec3) -> Vec<SpawnEvent> {
        let mut events = Vec::new();
        let mut rng = thread_rng();
        
        for i in 0..3 {
            let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
            let offset = Vec3::new(
                angle.cos() * 50.0,
                0.0,
                angle.sin() * 50.0
            );
            
            events.push(SpawnEvent {
                enemy_type: EnemyType::Swarm,
                position: carrier_pos + offset,
            });
        }
        
        events
    }
    
    /// Create enemies from spawn events
    pub fn create_enemies(events: Vec<SpawnEvent>) -> Vec<Enemy> {
        events.into_iter()
            .map(|event| {
                let mut enemy = Enemy::new(event.position.x, event.position.z, event.enemy_type);
                // Adjust spawn height to terrain
                let terrain_height = crate::terrain::height_at(event.position.x, event.position.z);
                enemy.pos.y = terrain_height + 50.0; // Spawn above terrain
                enemy
            })
            .collect()
    }
}