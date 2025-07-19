use glam::Vec3;
use crate::enemy::{Enemy, EnemyType};
use crate::terrain;

pub struct EnemyFactory;

impl EnemyFactory {
    /// Create a new enemy at the specified position
    pub fn create_enemy(x: f32, z: f32, enemy_type: EnemyType) -> Enemy {
        let mut enemy = Enemy::new(x, z, enemy_type);
        
        // Adjust spawn height based on terrain
        let terrain_height = terrain::height_at(x, z);
        enemy.pos.y = terrain_height + Self::get_spawn_height_offset(enemy_type);
        
        enemy
    }
    
    /// Create multiple enemies of the same type
    pub fn create_enemy_group(
        center: Vec3, 
        enemy_type: EnemyType, 
        count: usize, 
        spread: f32
    ) -> Vec<Enemy> {
        use rand::prelude::*;
        let mut rng = thread_rng();
        let mut enemies = Vec::with_capacity(count);
        
        for _ in 0..count {
            let offset_x = rng.gen_range(-spread..spread);
            let offset_z = rng.gen_range(-spread..spread);
            
            let enemy = Self::create_enemy(
                center.x + offset_x, 
                center.z + offset_z, 
                enemy_type
            );
            enemies.push(enemy);
        }
        
        enemies
    }
    
    /// Create a formation of enemies
    pub fn create_formation(
        center: Vec3,
        enemy_types: Vec<EnemyType>,
        formation_type: FormationType
    ) -> Vec<Enemy> {
        match formation_type {
            FormationType::Line(spacing) => {
                Self::create_line_formation(center, enemy_types, spacing)
            }
            FormationType::Circle(radius) => {
                Self::create_circle_formation(center, enemy_types, radius)
            }
            FormationType::V(angle, spacing) => {
                Self::create_v_formation(center, enemy_types, angle, spacing)
            }
            FormationType::Diamond(size) => {
                Self::create_diamond_formation(center, enemy_types, size)
            }
        }
    }
    
    /// Get the height offset for spawning different enemy types
    fn get_spawn_height_offset(enemy_type: EnemyType) -> f32 {
        match enemy_type {
            EnemyType::Cube | EnemyType::Guardian => 50.0,
            EnemyType::Pyramid | EnemyType::Hunter => 100.0,
            EnemyType::Spinner | EnemyType::Reflector => 150.0,
            EnemyType::Swarm => 30.0,
            EnemyType::Phaser => 200.0,
            EnemyType::Shield | EnemyType::Vortex => 20.0,
            EnemyType::Bomber | EnemyType::Carrier => 80.0,
            EnemyType::Laser | EnemyType::Disruptor => 60.0,
        }
    }
    
    fn create_line_formation(center: Vec3, enemy_types: Vec<EnemyType>, spacing: f32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let half_width = (enemy_types.len() as f32 - 1.0) * spacing / 2.0;
        
        for (i, enemy_type) in enemy_types.into_iter().enumerate() {
            let x_offset = i as f32 * spacing - half_width;
            let enemy = Self::create_enemy(
                center.x + x_offset,
                center.z,
                enemy_type
            );
            enemies.push(enemy);
        }
        
        enemies
    }
    
    fn create_circle_formation(center: Vec3, enemy_types: Vec<EnemyType>, radius: f32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let angle_step = std::f32::consts::TAU / enemy_types.len() as f32;
        
        for (i, enemy_type) in enemy_types.into_iter().enumerate() {
            let angle = i as f32 * angle_step;
            let x = center.x + angle.cos() * radius;
            let z = center.z + angle.sin() * radius;
            
            let enemy = Self::create_enemy(x, z, enemy_type);
            enemies.push(enemy);
        }
        
        enemies
    }
    
    fn create_v_formation(center: Vec3, enemy_types: Vec<EnemyType>, angle: f32, spacing: f32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let half_angle = angle / 2.0;
        
        // Leader at the front
        if let Some(leader_type) = enemy_types.first() {
            enemies.push(Self::create_enemy(center.x, center.z, *leader_type));
        }
        
        // Wings
        let wing_size = (enemy_types.len() - 1) / 2;
        for i in 1..=wing_size {
            let distance = i as f32 * spacing;
            
            // Left wing
            if let Some(&enemy_type) = enemy_types.get(i * 2 - 1) {
                let x = center.x - distance * half_angle.sin();
                let z = center.z - distance * half_angle.cos();
                enemies.push(Self::create_enemy(x, z, enemy_type));
            }
            
            // Right wing
            if let Some(&enemy_type) = enemy_types.get(i * 2) {
                let x = center.x + distance * half_angle.sin();
                let z = center.z - distance * half_angle.cos();
                enemies.push(Self::create_enemy(x, z, enemy_type));
            }
        }
        
        enemies
    }
    
    fn create_diamond_formation(center: Vec3, enemy_types: Vec<EnemyType>, size: f32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        
        // Diamond shape - place enemies at key points
        let positions = [
            Vec3::new(0.0, 0.0, size),      // Front
            Vec3::new(-size, 0.0, 0.0),     // Left
            Vec3::new(size, 0.0, 0.0),      // Right
            Vec3::new(0.0, 0.0, -size),     // Back
        ];
        
        for (i, enemy_type) in enemy_types.into_iter().take(4).enumerate() {
            if let Some(offset) = positions.get(i) {
                let pos = center + *offset;
                enemies.push(Self::create_enemy(pos.x, pos.z, enemy_type));
            }
        }
        
        enemies
    }
}

pub enum FormationType {
    Line(f32),           // spacing between enemies
    Circle(f32),         // radius
    V(f32, f32),        // angle, spacing
    Diamond(f32),        // size
}