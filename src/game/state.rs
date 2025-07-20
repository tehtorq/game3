use crate::player::Player;
use crate::enemy::Enemy;
use crate::bullet::Bullet;
use crate::particle::Particle;
use crate::mine::Mine;
use crate::base::Base;
use crate::tree::Tree;
use crate::tree_procedural::ProceduralTreeSystem;
use crate::sounds::SoundSystem;

use super::systems::{CollisionSystem, SpawningSystem, WaveManager, CombatSystem, BaseManager};

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub particles: Vec<Particle>,
    pub mines: Vec<Mine>,
    pub bases: Vec<Base>,
    pub trees: Vec<Tree>, // This will be removed once we fully switch to procedural
    tree_system: ProceduralTreeSystem,
    tree_update_timer: f32,  // Timer for updating trees once per second
    pub shoot_cooldown: f32,
    pub score: u32,
    pub player_invulnerable_timer: f32,
    pub player_slowed: bool,
    pub player_slow_timer: f32,
    pub position_log_timer: f32,
    wave_manager: WaveManager,
    enemy_stats_timer: f32,
    frame_count: u32,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            player: Player::new(),
            enemies: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            mines: Vec::new(),
            bases: Vec::new(),
            trees: Vec::new(),
            tree_system: ProceduralTreeSystem::new(),
            tree_update_timer: 0.0,  // Initialize tree update timer
            shoot_cooldown: 0.0,
            score: 0,
            player_invulnerable_timer: 0.0,
            player_slowed: false,
            player_slow_timer: 0.0,
            position_log_timer: 0.0,
            wave_manager: WaveManager::new(),
            enemy_stats_timer: 0.0,
            frame_count: 0,
        };
        
        // Generate initial bases
        game.bases = BaseManager::generate_initial_bases();
        
        game
    }

    pub fn update(&mut self, left: bool, right: bool, forward: bool, backward: bool, shoot: bool, boost: bool, up: bool, down: bool, dt: f32, sound_system: &mut SoundSystem) {
        self.frame_count += 1;
        
        // Update player
        self.player.update(left, right, forward, backward, boost, up, down, dt);
        
        // Update timers
        self.update_timers(dt);
        
        // Handle shooting
        if shoot && self.shoot_cooldown <= 0.0 {
            self.bullets.push(CombatSystem::create_player_bullet(&self.player));
            self.shoot_cooldown = 0.15;
            CombatSystem::alert_enemies_to_sound(&mut self.enemies, self.player.pos, 1500.0);
        }
        self.shoot_cooldown -= dt;
        
        // Update bases based on distance
        const BASE_ACTIVE_RANGE: f32 = 17500.0;  // Range for base activity (5x increase)
        
        let mut turret_bullets = Vec::new();
        let mut spawn_events = Vec::new();
        
        for base in self.bases.iter_mut() {
            if base.is_active {
                let distance_to_player = (base.pos - self.player.pos).length();
                
                if distance_to_player < BASE_ACTIVE_RANGE {
                    // Full update for nearby bases
                    let shots = base.update(self.player.pos, dt);
                    for (start_pos, direction, is_heavy) in shots {
                        let bullet_type = if is_heavy { 
                            crate::bullet::BulletType::HeavyTurret 
                        } else { 
                            crate::bullet::BulletType::Enemy 
                        };
                        turret_bullets.push(crate::bullet::Bullet::new_at_position(start_pos, direction, bullet_type));
                    }
                    
                    // Update spawn timer
                    base.update_spawn_timer(dt);
                    
                    // Check spawning
                    if base.can_spawn() {
                        let base_spawns = SpawningSystem::get_base_spawns(base, self.wave_manager.wave);
                        spawn_events.extend(base_spawns);
                        base.reset_spawn_timer();
                    }
                }
                // Bases beyond active range don't update at all
            }
        }
        
        self.bullets.extend(turret_bullets);
        let new_enemies = SpawningSystem::create_enemies(spawn_events);
        self.enemies.extend(new_enemies);
        
        // Update enemies
        self.update_enemies(dt);
        
        // Handle collisions
        self.handle_all_collisions();
        
        // Update bullets
        self.bullets.retain_mut(|bullet| {
            bullet.update(dt);
            bullet.lifetime > 0.0
        });
        
        // Update particles
        self.particles.retain_mut(|particle| {
            particle.update(dt);
            particle.lifetime > 0.0
        });
        
        // Update mines
        self.mines.retain_mut(|mine| {
            mine.update(dt);
            mine.lifetime > 0.0
        });
        
        // Handle wave progression
        if self.wave_manager.update(dt) {
            let new_bases = self.wave_manager.spawn_wave_bases(self.player.pos, &self.bases);
            self.bases.extend(new_bases);
        }
        
        // Update sound system (it doesn't need enemies/particles anymore)
        sound_system.update(dt);
    }
    
    fn update_timers(&mut self, dt: f32) {
        if self.player_invulnerable_timer > 0.0 {
            self.player_invulnerable_timer -= dt;
        }
        if self.player_slow_timer > 0.0 {
            self.player_slow_timer -= dt;
            if self.player_slow_timer <= 0.0 {
                self.player_slowed = false;
            }
        }
        
        // Log player position every second
        self.position_log_timer += dt;
        if self.position_log_timer >= 1.0 {
            println!("Player position: ({:.1}, {:.1}, {:.1})", 
                self.player.pos.x, self.player.pos.y, self.player.pos.z);
            self.position_log_timer = 0.0;
        }
        
        // Log enemy statistics every second
        self.enemy_stats_timer += dt;
        if self.enemy_stats_timer >= 1.0 {
            self.print_enemy_stats();
            self.enemy_stats_timer = 0.0;
        }
    }
    
    fn update_enemies(&mut self, dt: f32) {
        // Update enemies and collect their actions
        let mut enemy_bullets = Vec::new();
        let mut mines_to_add = Vec::new();
        let mut carrier_spawns = Vec::new();
        
        // Only fully update enemies within active range
        const ACTIVE_RANGE: f32 = 15000.0;  // Full AI updates (5x increase)
        const SIMPLE_RANGE: f32 = 25000.0;  // Simple updates only (5x increase)
        
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            let distance_to_player = (enemy.pos - self.player.pos).length();
            
            if distance_to_player < ACTIVE_RANGE {
                // Full update for nearby enemies
                enemy.update_with_player(self.player.pos, dt);
                
                // Collect enemy bullets
                if let Some(bullet) = enemy.try_shoot() {
                    enemy_bullets.push(bullet);
                }
                
                // Handle bomber mine drops
                if enemy.should_drop_mine() {
                    mines_to_add.push(Mine::new(enemy.pos));
                }
                
                // Handle carrier spawns
                if enemy.should_spawn_swarm() {
                    carrier_spawns.push(i);
                }
            } else if distance_to_player < SIMPLE_RANGE {
                // Simple position update only - no AI
                enemy.update(dt);
            }
            // Enemies beyond SIMPLE_RANGE are completely frozen
        }
        
        // Add collected items
        self.bullets.extend(enemy_bullets);
        self.mines.extend(mines_to_add);
        
        // Process carrier spawns
        for carrier_idx in carrier_spawns {
            if let Some(carrier) = self.enemies.get(carrier_idx) {
                let spawn_events = SpawningSystem::spawn_from_carrier(carrier.pos);
                let new_enemies = SpawningSystem::create_enemies(spawn_events);
                self.enemies.extend(new_enemies);
            }
        }
    }
    
    fn handle_all_collisions(&mut self) {
        // Bullet-base collisions
        let (bullets_to_remove, turret_hits) = CollisionSystem::check_bullet_base_collisions(
            &self.bullets, &mut self.bases, &mut self.particles, &mut self.score
        );
        
        // Process turret hits
        CollisionSystem::process_turret_hits(
            turret_hits, &mut self.bases, &mut self.particles, &mut self.score
        );
        
        // Remove hit bullets
        for &idx in bullets_to_remove.iter().rev() {
            self.bullets.remove(idx);
        }
        
        // Bullet-enemy collisions
        let (bullets_to_remove, enemies_to_remove) = CollisionSystem::check_bullet_enemy_collisions(
            &self.bullets, &self.enemies, &mut self.particles, &mut self.score
        );
        
        // Remove in reverse order
        for &idx in bullets_to_remove.iter().rev() {
            if idx < self.bullets.len() {
                self.bullets.remove(idx);
            }
        }
        for &idx in enemies_to_remove.iter().rev() {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
            }
        }
        
        // Enemy bullet-player collisions
        let (bullets_to_remove, reflected_bullets) = CollisionSystem::check_enemy_bullet_player_collisions(
            &self.bullets, &mut self.player, &self.enemies, &mut self.particles, &mut self.player_invulnerable_timer
        );
        
        // Remove hit bullets
        for &idx in bullets_to_remove.iter().rev() {
            if idx < self.bullets.len() {
                self.bullets.remove(idx);
            }
        }
        
        // Add reflected bullets
        for (enemy, direction) in reflected_bullets {
            self.bullets.push(CombatSystem::create_reflected_bullet(enemy.pos, direction));
        }
        
        // Player-enemy collisions
        let enemies_to_remove = CollisionSystem::check_player_enemy_collisions(
            &mut self.player, &self.enemies, &mut self.particles, &mut self.player_invulnerable_timer
        );
        
        for &idx in enemies_to_remove.iter().rev() {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
            }
        }
        
        // Player-mine collisions
        let mines_to_remove = CollisionSystem::check_player_mine_collisions(
            &mut self.player, &self.mines, &mut self.particles, &mut self.player_invulnerable_timer
        );
        
        for &idx in mines_to_remove.iter().rev() {
            if idx < self.mines.len() {
                self.mines.remove(idx);
            }
        }
        
        // Check enemy abilities
        CollisionSystem::check_enemy_abilities(
            &mut self.player, &self.enemies, &mut self.particles, 
            &mut self.player_invulnerable_timer, &mut self.player_slowed, &mut self.player_slow_timer
        );
    }
    
    pub fn get_wave(&self) -> u32 {
        self.wave_manager.wave
    }
    
    fn print_enemy_stats(&self) {
        use crate::enemy::{EnemyType, AlertState};
        use std::collections::HashMap;
        
        let total_enemies = self.enemies.len();
        
        // Count by type
        let mut type_counts = HashMap::new();
        let mut alert_counts = HashMap::new();
        let mut type_alert_breakdown = HashMap::new();
        
        for enemy in &self.enemies {
            // Count by type
            *type_counts.entry(enemy.enemy_type).or_insert(0) += 1;
            
            // Count by alert state
            *alert_counts.entry(enemy.alert_state).or_insert(0) += 1;
            
            // Count by type and alert state
            let key = (enemy.enemy_type, enemy.alert_state);
            *type_alert_breakdown.entry(key).or_insert(0) += 1;
        }
        
        // Count active vs inactive enemies
        const ACTIVE_RANGE: f32 = 15000.0;  // 5x increase
        const SIMPLE_RANGE: f32 = 25000.0;  // 5x increase
        let mut active_enemies = 0;
        let mut simple_enemies = 0;
        let mut frozen_enemies = 0;
        
        for enemy in &self.enemies {
            let distance = (enemy.pos - self.player.pos).length();
            if distance < ACTIVE_RANGE {
                active_enemies += 1;
            } else if distance < SIMPLE_RANGE {
                simple_enemies += 1;
            } else {
                frozen_enemies += 1;
            }
        }
        
        // Count active bases
        const BASE_ACTIVE_RANGE: f32 = 17500.0;  // 5x increase
        let active_bases = self.bases.iter()
            .filter(|b| b.is_active && (b.pos - self.player.pos).length() < BASE_ACTIVE_RANGE)
            .count();
        let total_bases = self.bases.iter().filter(|b| b.is_active).count();
        
        println!("\n========== Enemy Statistics ==========");
        println!("Total Enemies: {} (Active: {}, Simple: {}, Frozen: {})", 
                 total_enemies, active_enemies, simple_enemies, frozen_enemies);
        println!("Active Bases: {} / {} total", active_bases, total_bases);
        
        // Print by type
        println!("\nBy Type:");
        let mut type_vec: Vec<_> = type_counts.iter().collect();
        type_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        for (enemy_type, count) in type_vec {
            println!("  {:?}: {}", enemy_type, count);
        }
        
        // Print by alert state
        println!("\nBy Alert State:");
        for alert_state in &[AlertState::Unaware, AlertState::Suspicious, AlertState::Alert, AlertState::Searching, AlertState::Returning] {
            if let Some(count) = alert_counts.get(alert_state) {
                println!("  {:?}: {}", alert_state, count);
            }
        }
        
        // Print detailed breakdown for alerted enemies
        println!("\nAlerted Enemy Breakdown:");
        for enemy_type in &[EnemyType::Cube, EnemyType::Pyramid, EnemyType::Spinner, EnemyType::Hunter, 
                           EnemyType::Guardian, EnemyType::Laser, EnemyType::Swarm] {
            let alert_count = type_alert_breakdown.get(&(*enemy_type, AlertState::Alert)).unwrap_or(&0);
            let suspicious_count = type_alert_breakdown.get(&(*enemy_type, AlertState::Suspicious)).unwrap_or(&0);
            if *alert_count > 0 || *suspicious_count > 0 {
                println!("  {:?}: {} alert, {} suspicious", enemy_type, alert_count, suspicious_count);
            }
        }
        
        println!("====================================\n");
    }
    
    
    pub fn update_trees(&mut self, dt: f32) {
        // Only update trees once per second to improve performance
        self.tree_update_timer -= dt;
        
        if self.tree_update_timer <= 0.0 {
            // Reset timer
            self.tree_update_timer = 1.0; // Update every second
            
            // Use procedural generation - trees are generated on demand
            let start_time = std::time::Instant::now();
            self.trees = self.tree_system.generate_visible_trees(self.player.pos);
            let elapsed = start_time.elapsed();
            
            // Log performance
            println!("Trees: {} generated in {:?}", self.trees.len(), elapsed);
        }
    }
    
    pub fn get_visible_trees(&self) -> &[Tree] {
        &self.trees
    }
}