use rand::prelude::*;
use std::f32::consts::PI;
use glam::Vec3;

use crate::player::Player;
use crate::enemy::{Enemy, EnemyType};
use crate::bullet::Bullet;
use crate::particle::Particle;
use crate::terrain::TerrainChunk;
use crate::renderer::{Renderer, Drawable};

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub terrain_chunks: Vec<TerrainChunk>,
    pub enemy_spawn_timer: f32,
    pub shoot_cooldown: f32,
    pub wave: u32,
    pub score: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            player: Player::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            terrain_chunks: Vec::new(),
            enemy_spawn_timer: 0.0,
            shoot_cooldown: 0.0,
            wave: 1,
            score: 0,
        };
        
        // Create initial terrain chunks
        game.update_terrain_chunks();
        game.spawn_wave();
        
        game
    }

    pub fn update(&mut self, left: bool, right: bool, up: bool, down: bool, shoot: bool, dt: f32) {
        // Update player
        self.player.update(left, right, up, down, dt);
        
        // Update terrain chunks
        self.update_terrain_chunks();
        
        // Handle shooting
        if shoot && self.shoot_cooldown <= 0.0 {
            self.bullets.push(Bullet::new(&self.player));
            self.shoot_cooldown = 0.15;
        }
        self.shoot_cooldown -= dt;
        
        // Update enemies
        for enemy in &mut self.enemies {
            enemy.update(dt);
        }
        
        // Remove enemies that are too far away
        let player_z = self.player.pos.z;
        self.enemies.retain(|e| {
            let distance = (e.pos - self.player.pos).length();
            distance < 2000.0 && e.pos.z > player_z - 1000.0
        });
        
        // Update bullets
        for bullet in &mut self.bullets {
            bullet.update(dt);
        }
        
        // Remove bullets that are too far away
        self.bullets.retain(|b| {
            let dist = (b.pos - self.player.pos).length();
            dist < 2000.0
        });
        
        // Update particles
        self.particles.retain_mut(|p| p.update(dt));
        
        // Check collisions
        self.check_collisions();
        
        // Spawn new enemies
        self.enemy_spawn_timer -= dt;
        if self.enemy_spawn_timer <= 0.0 {
            self.spawn_wave();
            self.enemy_spawn_timer = 2.0;
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        // DEBUG: Draw a cross at world origin
        renderer.draw_line(Vec3::new(-100.0, 0.0, 0.0), Vec3::new(100.0, 0.0, 0.0));
        renderer.draw_line(Vec3::new(0.0, -100.0, 0.0), Vec3::new(0.0, 100.0, 0.0));
        renderer.draw_line(Vec3::new(0.0, 0.0, -100.0), Vec3::new(0.0, 0.0, 100.0));
        
        // Draw terrain
        for chunk in &self.terrain_chunks {
            chunk.draw(renderer);
        }
        
        // Draw player
        self.player.draw(renderer);
        
        // Draw enemies
        for enemy in &self.enemies {
            enemy.draw(renderer);
        }
        
        // Draw bullets
        for bullet in &self.bullets {
            bullet.draw(renderer);
        }
        
        // Draw particles
        for particle in &self.particles {
            particle.draw(renderer);
        }
    }

    fn update_terrain_chunks(&mut self) {
        let chunk_size = 80.0;
        let view_distance = 24; // Number of chunks in each direction
        
        // Calculate which chunk the player is in
        let player_chunk_x = (self.player.pos.x / chunk_size).floor() as i32;
        let player_chunk_z = (self.player.pos.z / chunk_size).floor() as i32;
        
        // Create a set of chunks that should exist
        let mut needed_chunks = std::collections::HashSet::new();
        for dx in -view_distance..=view_distance {
            for dz in -view_distance..=view_distance {
                needed_chunks.insert((player_chunk_x + dx, player_chunk_z + dz));
            }
        }
        
        // Remove chunks that are too far away
        self.terrain_chunks.retain(|chunk| {
            let chunk_x = (chunk.x_offset / chunk_size).round() as i32;
            let chunk_z = (chunk.z_offset / chunk_size).round() as i32;
            needed_chunks.contains(&(chunk_x, chunk_z))
        });
        
        // Add missing chunks
        for &(chunk_x, chunk_z) in &needed_chunks {
            let x_offset = chunk_x as f32 * chunk_size;
            let z_offset = chunk_z as f32 * chunk_size;
            
            // Check if this chunk already exists
            let exists = self.terrain_chunks.iter().any(|chunk| {
                let existing_x = (chunk.x_offset / chunk_size).round() as i32;
                let existing_z = (chunk.z_offset / chunk_size).round() as i32;
                existing_x == chunk_x && existing_z == chunk_z
            });
            
            if !exists {
                self.terrain_chunks.push(TerrainChunk::new(x_offset, z_offset));
            }
        }
    }

    fn spawn_wave(&mut self) {
        let mut rng = thread_rng();
        let enemy_count = 5 + self.wave as usize;
        
        for i in 0..enemy_count {
            // Spawn enemies in front of the player in a spread pattern
            let angle_offset = (i as f32 / enemy_count as f32 - 0.5) * PI * 0.8;
            let distance = rng.gen_range(300.0..800.0);
            let spawn_angle = self.player.rotation + angle_offset;
            
            let x = self.player.pos.x - spawn_angle.sin() * distance;
            let z = self.player.pos.z - spawn_angle.cos() * distance;
            
            let enemy_type = match rng.gen_range(0..3) {
                0 => EnemyType::Cube,
                1 => EnemyType::Pyramid,
                _ => EnemyType::Spinner,
            };
            
            self.enemies.push(Enemy::new(x, z, enemy_type));
        }
    }

    fn check_collisions(&mut self) {
        let mut bullets_to_remove = vec![];
        let mut enemies_to_remove = vec![];
        
        // Check bullet-enemy collisions
        for (bi, bullet) in self.bullets.iter().enumerate() {
            for (ei, enemy) in self.enemies.iter().enumerate() {
                let dist = (bullet.pos - enemy.pos).length();
                if dist < 30.0 {
                    bullets_to_remove.push(bi);
                    enemies_to_remove.push(ei);
                    
                    // Create explosion particles
                    for _ in 0..15 {
                        self.particles.push(Particle::new(enemy.pos));
                    }
                    
                    self.score += match enemy.enemy_type {
                        EnemyType::Cube => 10,
                        EnemyType::Pyramid => 20,
                        EnemyType::Spinner => 30,
                    };
                }
            }
        }
        
        // Check player-enemy collisions
        for (ei, enemy) in self.enemies.iter().enumerate() {
            let dist = (self.player.pos - enemy.pos).length();
            if dist < 40.0 {
                enemies_to_remove.push(ei);
                for _ in 0..20 {
                    self.particles.push(Particle::new(enemy.pos));
                }
            }
        }
        
        // Remove collided entities
        bullets_to_remove.sort_unstable();
        bullets_to_remove.dedup();
        for &i in bullets_to_remove.iter().rev() {
            self.bullets.remove(i);
        }
        
        enemies_to_remove.sort_unstable();
        enemies_to_remove.dedup();
        for &i in enemies_to_remove.iter().rev() {
            self.enemies.remove(i);
        }
    }
}