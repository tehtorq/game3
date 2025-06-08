use rand::prelude::*;
use std::f32::consts::PI;

use crate::player::Player;
use crate::enemy::{Enemy, EnemyType};
use crate::bullet::{Bullet, BulletType};
use crate::particle::Particle;
use crate::renderer::Renderer;

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
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
            enemy_spawn_timer: 3.0,  // Start with 3 second delay before first wave
            shoot_cooldown: 0.0,
            wave: 1,
            score: 0,
        };
        
        game.spawn_wave();
        
        game
    }

    pub fn update(&mut self, left: bool, right: bool, up: bool, down: bool, shoot: bool, boost: bool, dt: f32) {
        // Update player
        self.player.update(left, right, up, down, boost, dt);
        
        // Handle shooting
        if shoot && self.shoot_cooldown <= 0.0 {
            self.bullets.push(Bullet::new(&self.player));
            self.shoot_cooldown = 0.15;
        }
        self.shoot_cooldown -= dt;
        
        // Update enemies with player awareness and handle attacks
        let mut enemy_bullets = Vec::new();
        for enemy in &mut self.enemies {
            if let Some(attack_dir) = enemy.update_with_player(self.player.pos, dt) {
                enemy_bullets.push(Bullet::new_enemy(enemy, attack_dir));
            }
        }
        self.bullets.extend(enemy_bullets);
        
        // Remove enemies that are too far away or behind
        let player_z = self.player.pos.z;
        self.enemies.retain(|e| {
            let distance = (e.pos - self.player.pos).length();
            // Keep enemies within 5000 units and don't remove them if they're still ahead
            distance < 5000.0 && (e.pos.z < player_z || e.pos.z > player_z - 2000.0)
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
        
        // Spawn new enemies with longer intervals
        self.enemy_spawn_timer -= dt;
        if self.enemy_spawn_timer <= 0.0 {
            self.spawn_wave();
            // Longer spawn intervals: 5-8 seconds based on wave
            self.enemy_spawn_timer = (8.0 - (self.wave as f32 * 0.3)).max(5.0);
        }
    }

    pub fn draw(&self, _renderer: &mut Renderer) {
        // This method can remain empty as we're handling drawing in main.rs now
    }


    fn spawn_wave(&mut self) {
        let mut rng = thread_rng();
        
        // Wave composition changes based on wave number
        let wave_templates = [
            // Wave 1-3: Basic enemies
            vec![(EnemyType::Cube, 5), (EnemyType::Pyramid, 2)],
            // Wave 4-6: Add spinners
            vec![(EnemyType::Cube, 4), (EnemyType::Pyramid, 3), (EnemyType::Spinner, 2)],
            // Wave 7-9: Add hunters
            vec![(EnemyType::Pyramid, 3), (EnemyType::Spinner, 3), (EnemyType::Hunter, 2)],
            // Wave 10+: Add guardians
            vec![(EnemyType::Spinner, 2), (EnemyType::Hunter, 3), (EnemyType::Guardian, 2), (EnemyType::Pyramid, 2)],
        ];
        
        let template_index = ((self.wave - 1) / 3).min(3) as usize;
        let template = &wave_templates[template_index];
        
        // Spawn different formations based on wave
        match self.wave % 4 {
            1 => self.spawn_arc_formation(template, &mut rng),
            2 => self.spawn_pincer_formation(template, &mut rng),
            3 => self.spawn_surround_formation(template, &mut rng),
            _ => self.spawn_ambush_formation(template, &mut rng),
        }
    }
    
    fn spawn_arc_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn in an arc in front of the player
        let mut spawn_count = 0;
        for &(enemy_type, count) in template {
            for i in 0..count {
                let total_enemies: usize = template.iter().map(|(_, c)| c).sum();
                let angle_offset = ((spawn_count + i) as f32 / total_enemies as f32 - 0.5) * PI * 0.8;
                // Spawn much farther ahead: 2000-4000 units
                let distance = rng.gen_range(2000.0..4000.0);
                let spawn_angle = self.player.rotation + angle_offset;
                
                let x = self.player.pos.x - spawn_angle.sin() * distance;
                let z = self.player.pos.z - spawn_angle.cos() * distance;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
            spawn_count += count;
        }
    }
    
    fn spawn_pincer_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn on both sides of the player
        for &(enemy_type, count) in template {
            for i in 0..count {
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                // Spawn farther ahead and to the sides
                let forward_offset = rng.gen_range(1500.0..2500.0);
                let side_offset = rng.gen_range(800.0..1500.0) * side;
                
                let spawn_angle = self.player.rotation;
                let x = self.player.pos.x - spawn_angle.sin() * forward_offset + spawn_angle.cos() * side_offset;
                let z = self.player.pos.z - spawn_angle.cos() * forward_offset - spawn_angle.sin() * side_offset;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
        }
    }
    
    fn spawn_surround_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn in a circle around the player
        let mut spawn_count = 0;
        let total_enemies: usize = template.iter().map(|(_, c)| c).sum();
        
        for &(enemy_type, count) in template {
            for i in 0..count {
                let angle = ((spawn_count + i) as f32 / total_enemies as f32) * PI * 2.0;
                // Larger surrounding circle
                let distance = rng.gen_range(1500.0..2500.0);
                
                let x = self.player.pos.x + angle.cos() * distance;
                let z = self.player.pos.z + angle.sin() * distance;
                
                self.enemies.push(Enemy::new(x, z, enemy_type));
            }
            spawn_count += count;
        }
    }
    
    fn spawn_ambush_formation(&mut self, template: &Vec<(EnemyType, usize)>, rng: &mut impl Rng) {
        // Enemies spawn from behind and above
        for &(enemy_type, count) in template {
            for i in 0..count {
                let angle_offset = rng.gen_range(-PI/4.0..PI/4.0);
                let spawn_angle = self.player.rotation + PI + angle_offset; // Behind player
                // Even ambushes spawn farther away
                let distance = rng.gen_range(1000.0..1800.0);
                
                let x = self.player.pos.x - spawn_angle.sin() * distance;
                let z = self.player.pos.z - spawn_angle.cos() * distance;
                
                // Force some enemies to spawn higher for vertical ambush
                let mut enemy = Enemy::new(x, z, enemy_type);
                if i % 2 == 0 {
                    enemy.pos.y += rng.gen_range(100.0..200.0);
                }
                
                self.enemies.push(enemy);
            }
        }
    }

    fn check_collisions(&mut self) {
        let mut bullets_to_remove = vec![];
        let mut enemies_to_remove = vec![];
        
        // Check bullet-enemy collisions (player bullets only)
        for (bi, bullet) in self.bullets.iter().enumerate() {
            match bullet.bullet_type {
                BulletType::Player => {
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
                                EnemyType::Hunter => 40,
                                EnemyType::Guardian => 50,
                            };
                        }
                    }
                }
                BulletType::Enemy => {
                    // Check collision with player
                    let dist = (bullet.pos - self.player.pos).length();
                    if dist < 25.0 {
                        bullets_to_remove.push(bi);
                        
                        // Create hit effect
                        for _ in 0..10 {
                            self.particles.push(Particle::new(self.player.pos));
                        }
                        
                        // TODO: Add player health/damage system
                    }
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