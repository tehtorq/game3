use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::terrain::terrain_height_fallback;
use crate::terrain_instanced::InstancedTerrain;
use rand::prelude::*;
use std::f32::consts::PI;

pub struct Base {
    pub pos: Vec3,
    pub health: f32,
    pub turrets: Vec<Turret>,
    pub ground_turrets: Vec<GroundTurret>,  // Heavy defensive turrets
    pub spawn_timer: f32,
    pub spawn_interval: f32,
    pub is_active: bool,
    pub base_type: BaseType,
    pub rotation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BaseType {
    Small,   // 2 turrets, spawns basic enemies
    Medium,  // 4 turrets, spawns intermediate enemies  
    Large,   // 6 turrets, spawns advanced enemies
    Fortress,// 8 turrets, spawns boss-tier enemies
}

pub struct Turret {
    pub offset: Vec3,      // Offset from base center
    pub rotation: f32,     // Current rotation
    pub pitch: f32,        // Vertical aim
    pub fire_timer: f32,   // Time until next shot
    pub tracking_speed: f32,
    pub range: f32,
}

pub struct GroundTurret {
    pub pos: Vec3,         // World position
    pub rotation: f32,     // Current rotation
    pub pitch: f32,        // Vertical aim
    pub fire_timer: f32,   // Time until next shot
    pub burst_count: i32,  // Shots remaining in burst
    pub is_active: bool,   // Can be destroyed independently
    pub health: f32,
}

impl Base {
    pub fn new(x: f32, z: f32, base_type: BaseType) -> Self {
        Self::new_with_terrain(x, z, base_type, None)
    }
    
    pub fn new_with_terrain(x: f32, z: f32, base_type: BaseType, terrain: Option<&InstancedTerrain>) -> Self {
        let y = if let Some(t) = terrain {
            t.get_height_at(x, z)
        } else {
            terrain_height_fallback(x, z)
        } + 20.0; // Place base slightly above terrain
        let pos = Vec3::new(x, y, z);
        
        // Create turrets based on base type
        let (turret_count, spawn_interval) = match base_type {
            BaseType::Small => (2, 10.0),
            BaseType::Medium => (4, 8.0),
            BaseType::Large => (6, 5.0),
            BaseType::Fortress => (8, 4.0),
        };
        
        let mut turrets = Vec::new();
        for i in 0..turret_count {
            let angle = i as f32 * PI * 2.0 / turret_count as f32;
            let radius = match base_type {
                BaseType::Small => 60.0,   // Spread base turrets out more too
                BaseType::Medium => 90.0,
                BaseType::Large => 120.0,
                BaseType::Fortress => 150.0,
            };
            
            turrets.push(Turret {
                offset: Vec3::new(angle.cos() * radius, 10.0, angle.sin() * radius),
                rotation: angle,
                pitch: 0.0,
                fire_timer: i as f32 * 0.5, // Stagger initial fire times
                tracking_speed: 4.0,
                range: 6000.0, // Double range - engage from very far away
            });
        }
        
        // Create ground turrets - heavy defensive emplacements
        let ground_turret_count = match base_type {
            BaseType::Small => 6,     // More turrets for chaos
            BaseType::Medium => 8,
            BaseType::Large => 12,
            BaseType::Fortress => 16,
        };
        
        let ground_turret_radius = match base_type {
            BaseType::Small => 200.0,   // Spread out more
            BaseType::Medium => 300.0,
            BaseType::Large => 400.0,
            BaseType::Fortress => 500.0,
        };
        
        let mut ground_turrets = Vec::new();
        for i in 0..ground_turret_count {
            let angle = i as f32 * PI * 2.0 / ground_turret_count as f32;
            let turret_x = x + angle.cos() * ground_turret_radius;
            let turret_z = z + angle.sin() * ground_turret_radius;
            let turret_y = if let Some(t) = terrain {
                t.get_height_at(turret_x, turret_z)
            } else {
                terrain_height_fallback(turret_x, turret_z)
            } + 5.0; // Lower to ground
            
            ground_turrets.push(GroundTurret {
                pos: Vec3::new(turret_x, turret_y, turret_z),
                rotation: angle + PI, // Face inward initially
                pitch: 0.0,
                fire_timer: i as f32 * 0.2, // Stagger fire times more closely
                burst_count: 0,
                is_active: true,
                health: 50.0,
            });
        }
        
        Self {
            pos,
            health: match base_type {
                BaseType::Small => 100.0,
                BaseType::Medium => 200.0,
                BaseType::Large => 300.0,
                BaseType::Fortress => 500.0,
            },
            turrets,
            ground_turrets,
            spawn_timer: 5.0, // Initial delay before spawning
            spawn_interval,
            is_active: true,
            base_type,
            rotation: 0.0,
        }
    }
    
    pub fn update(&mut self, player_pos: Vec3, dt: f32) -> Vec<(Vec3, Vec3, bool)> { // bool = is_heavy
        if !self.is_active {
            return Vec::new();
        }
        
        // Update spawn timer
        self.spawn_timer -= dt;
        
        // Rotate base slowly for visual effect
        self.rotation += dt * 0.1;
        
        // Update turrets and collect bullets to fire
        let mut bullets = Vec::new();
        
        // Update base turrets (lighter, faster tracking)
        for turret in &mut self.turrets {
            // Calculate turret world position
            let turret_pos = self.pos + turret.offset;
            let to_player = player_pos - turret_pos;
            let distance = to_player.length();
            
            // Only track and fire if player is in range
            if distance < turret.range && distance > 10.0 {
                // Calculate desired rotation to face player
                let horizontal_dir = Vec3::new(to_player.x, 0.0, to_player.z).normalize();
                let target_rotation = horizontal_dir.z.atan2(horizontal_dir.x);
                
                // Smoothly rotate turret towards player
                let rotation_diff = angle_difference(turret.rotation, target_rotation);
                turret.rotation += rotation_diff.clamp(-turret.tracking_speed * dt, turret.tracking_speed * dt);
                
                // Calculate pitch to aim at player
                let horizontal_dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
                let target_pitch = (to_player.y / horizontal_dist).atan();
                turret.pitch += (target_pitch - turret.pitch) * turret.tracking_speed * dt;
                
                // Fire if ready and roughly aimed
                turret.fire_timer -= dt;
                if turret.fire_timer <= 0.0 && rotation_diff.abs() < 0.3 {
                    // Calculate bullet direction
                    let dir = to_player.normalize();
                    bullets.push((turret_pos, dir, false)); // Regular turret
                    turret.fire_timer = 0.5; // Fire twice per second
                }
            }
        }
        
        // Update ground turrets (heavy, burst fire)
        for ground_turret in &mut self.ground_turrets {
            if !ground_turret.is_active {
                continue;
            }
            
            let to_player = player_pos - ground_turret.pos;
            let distance = to_player.length();
            
            // Ground turrets have extreme range for early engagement
            if distance < 10000.0 && distance > 20.0 { // Extreme range - start firing very early
                // Calculate desired rotation to face player
                let horizontal_dir = Vec3::new(to_player.x, 0.0, to_player.z).normalize();
                let target_rotation = horizontal_dir.z.atan2(horizontal_dir.x);
                
                // Slower tracking for heavier turrets
                let rotation_diff = angle_difference(ground_turret.rotation, target_rotation);
                ground_turret.rotation += rotation_diff.clamp(-1.5 * dt, 1.5 * dt);
                
                // Calculate pitch
                let horizontal_dist = (to_player.x * to_player.x + to_player.z * to_player.z).sqrt();
                let target_pitch = (to_player.y / horizontal_dist).atan();
                ground_turret.pitch += (target_pitch - ground_turret.pitch) * 1.5 * dt;
                
                // Fire burst if ready and aimed
                ground_turret.fire_timer -= dt;
                
                // Handle burst firing
                if ground_turret.burst_count > 0 && ground_turret.fire_timer <= 0.0 {
                    // Fire a shot in the burst
                    let spread = 0.08; // More spread for chaotic effect
                    let dir = to_player.normalize();
                    let spread_x = (random::<f32>() - 0.5) * spread;
                    let spread_y = (random::<f32>() - 0.5) * spread;
                    let spread_z = (random::<f32>() - 0.5) * spread;
                    let spread_dir = (dir + Vec3::new(spread_x, spread_y, spread_z)).normalize();
                    
                    bullets.push((ground_turret.pos + Vec3::new(0.0, 5.0, 0.0), spread_dir, true)); // Heavy turret
                    ground_turret.burst_count -= 1;
                    ground_turret.fire_timer = 0.05; // Very fast burst fire for chaos
                    
                } else if ground_turret.burst_count == 0 && ground_turret.fire_timer <= 0.0 && rotation_diff.abs() < 0.2 {
                    // Start new burst
                    ground_turret.burst_count = 8; // 8-round burst for more chaos
                    ground_turret.fire_timer = 0.0;
                }
            }
        }
        
        bullets
    }
    
    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            self.is_active = false;
        }
    }
    
    pub fn damage_ground_turret(&mut self, turret_index: usize, damage: f32) -> bool {
        if let Some(turret) = self.ground_turrets.get_mut(turret_index) {
            if turret.is_active {
                turret.health -= damage;
                if turret.health <= 0.0 {
                    turret.is_active = false;
                    return true; // Turret destroyed
                }
            }
        }
        false
    }
    
    pub fn get_ground_turret_positions(&self) -> Vec<(usize, Vec3, bool)> {
        self.ground_turrets.iter().enumerate()
            .map(|(i, t)| (i, t.pos, t.is_active))
            .collect()
    }
    
    pub fn should_spawn(&self) -> bool {
        self.is_active && self.spawn_timer <= 0.0
    }
    
    pub fn reset_spawn_timer(&mut self) {
        self.spawn_timer = self.spawn_interval;
    }
    
    pub fn get_spawn_types(&self) -> Vec<crate::enemy::EnemyType> {
        use crate::enemy::EnemyType;
        
        match self.base_type {
            BaseType::Small => vec![EnemyType::Cube, EnemyType::Pyramid],
            BaseType::Medium => vec![EnemyType::Spinner, EnemyType::Swarm, EnemyType::Shield],
            BaseType::Large => vec![EnemyType::Hunter, EnemyType::Laser, EnemyType::Bomber, EnemyType::Phaser],
            BaseType::Fortress => vec![EnemyType::Guardian, EnemyType::Vortex, EnemyType::Carrier, EnemyType::Reflector],
        }
    }
    
    pub fn get_patrol_routes(&self) -> Vec<Vec<Vec3>> {
        // Generate patrol routes radiating from base - MUCH larger for huge map
        let mut routes = Vec::new();
        let route_count = match self.base_type {
            BaseType::Small => 2,
            BaseType::Medium => 3,
            BaseType::Large => 4,
            BaseType::Fortress => 6,
        };
        
        let route_length = match self.base_type {
            BaseType::Small => 10000.0,
            BaseType::Medium => 15000.0,
            BaseType::Large => 20000.0,
            BaseType::Fortress => 25000.0,
        };
        
        for i in 0..route_count {
            let angle = i as f32 * PI * 2.0 / route_count as f32 + self.rotation;
            let mut waypoints = vec![self.pos + Vec3::new(0.0, 100.0, 0.0)]; // Start above base
            
            // Create waypoints extending outward
            for j in 1..8 {
                let distance = (j as f32 / 7.0) * route_length;
                let angle_variation = (j as f32 * 0.2).sin() * 0.3; // Curved routes
                let current_angle = angle + angle_variation;
                let x = self.pos.x + current_angle.cos() * distance;
                let z = self.pos.z + current_angle.sin() * distance;
                let y = terrain_height_fallback(x, z) + 100.0 + (j as f32 * 30.0);
                waypoints.push(Vec3::new(x, y, z));
            }
            
            routes.push(waypoints);
        }
        
        routes
    }
}

impl Drawable for Base {
    fn draw(&self, renderer: &mut Renderer) {
        if !self.is_active {
            return;
        }
        
        let size = match self.base_type {
            BaseType::Small => 50.0,   // Bigger bases to match spread turrets
            BaseType::Medium => 70.0,
            BaseType::Large => 90.0,
            BaseType::Fortress => 120.0,
        };
        
        // Draw main structure - octagonal base
        let segments = 8;
        let mut base_vertices = Vec::new();
        for i in 0..segments {
            let angle = i as f32 * PI * 2.0 / segments as f32 + self.rotation;
            base_vertices.push(self.pos + Vec3::new(angle.cos() * size, 0.0, angle.sin() * size));
        }
        
        // Draw base walls
        for i in 0..segments {
            let next = (i + 1) % segments;
            let top_a = base_vertices[i] + Vec3::new(0.0, 20.0, 0.0);
            let top_b = base_vertices[next] + Vec3::new(0.0, 20.0, 0.0);
            
            // Wall face
            renderer.draw_triangle(base_vertices[i], base_vertices[next], top_a);
            renderer.draw_triangle(base_vertices[next], top_b, top_a);
        }
        
        // Draw top
        let center_top = self.pos + Vec3::new(0.0, 20.0, 0.0);
        for i in 0..segments {
            let next = (i + 1) % segments;
            renderer.draw_triangle(
                center_top,
                base_vertices[i] + Vec3::new(0.0, 20.0, 0.0),
                base_vertices[next] + Vec3::new(0.0, 20.0, 0.0)
            );
        }
        
        // Draw central spire
        let spire_height = match self.base_type {
            BaseType::Small => 40.0,
            BaseType::Medium => 60.0,
            BaseType::Large => 80.0,
            BaseType::Fortress => 100.0,
        };
        let spire_top = self.pos + Vec3::new(0.0, spire_height, 0.0);
        let spire_size = size * 0.3;
        
        for i in 0..4 {
            let angle = i as f32 * PI * 0.5 + self.rotation * 2.0;
            let next_angle = (i + 1) as f32 * PI * 0.5 + self.rotation * 2.0;
            let base_a = center_top + Vec3::new(angle.cos() * spire_size, 0.0, angle.sin() * spire_size);
            let base_b = center_top + Vec3::new(next_angle.cos() * spire_size, 0.0, next_angle.sin() * spire_size);
            
            renderer.draw_triangle(spire_top, base_a, base_b);
        }
        
        // Draw turrets
        for turret in &self.turrets {
            let turret_pos = self.pos + turret.offset;
            
            // Turret base
            let turret_size = 8.0;
            for i in 0..6 {
                let angle1 = i as f32 * PI / 3.0;
                let angle2 = (i + 1) as f32 * PI / 3.0;
                
                let v1 = turret_pos + Vec3::new(angle1.cos() * turret_size, 0.0, angle1.sin() * turret_size);
                let v2 = turret_pos + Vec3::new(angle2.cos() * turret_size, 0.0, angle2.sin() * turret_size);
                let top = turret_pos + Vec3::new(0.0, 10.0, 0.0);
                
                renderer.draw_triangle(turret_pos, v1, v2);
                renderer.draw_triangle(v1, v2, top);
            }
            
            // Turret barrel
            let barrel_length = 15.0;
            let barrel_dir = Vec3::new(
                turret.rotation.cos() * turret.pitch.cos(),
                turret.pitch.sin(),
                turret.rotation.sin() * turret.pitch.cos()
            );
            let barrel_end = turret_pos + Vec3::new(0.0, 8.0, 0.0) + barrel_dir * barrel_length;
            
            // Draw barrel as pyramid
            let barrel_base = turret_pos + Vec3::new(0.0, 8.0, 0.0);
            let right = Vec3::new(-barrel_dir.z, 0.0, barrel_dir.x).normalize() * 3.0;
            let up = barrel_dir.cross(right).normalize() * 3.0;
            
            renderer.draw_triangle(barrel_end, barrel_base + right, barrel_base + up);
            renderer.draw_triangle(barrel_end, barrel_base + up, barrel_base - right);
            renderer.draw_triangle(barrel_end, barrel_base - right, barrel_base - up);
            renderer.draw_triangle(barrel_end, barrel_base - up, barrel_base + right);
        }
        
        // Draw ground turrets
        for ground_turret in &self.ground_turrets {
            if !ground_turret.is_active {
                continue;
            }
            
            // Heavy turret base - octagonal bunker
            let bunker_size = 15.0;
            let bunker_height = 8.0;
            let segments = 8;
            
            // Base vertices
            let mut base_verts = Vec::new();
            for i in 0..segments {
                let angle = i as f32 * PI * 2.0 / segments as f32;
                base_verts.push(ground_turret.pos + Vec3::new(
                    angle.cos() * bunker_size,
                    0.0,
                    angle.sin() * bunker_size
                ));
            }
            
            // Draw bunker walls
            for i in 0..segments {
                let next = (i + 1) % segments;
                let top_a = base_verts[i] + Vec3::new(0.0, bunker_height, 0.0);
                let top_b = base_verts[next] + Vec3::new(0.0, bunker_height, 0.0);
                
                renderer.draw_triangle(base_verts[i], base_verts[next], top_a);
                renderer.draw_triangle(base_verts[next], top_b, top_a);
            }
            
            // Draw bunker top
            let center_top = ground_turret.pos + Vec3::new(0.0, bunker_height, 0.0);
            for i in 0..segments {
                let next = (i + 1) % segments;
                renderer.draw_triangle(
                    center_top,
                    base_verts[i] + Vec3::new(0.0, bunker_height, 0.0),
                    base_verts[next] + Vec3::new(0.0, bunker_height, 0.0)
                );
            }
            
            // Heavy dual barrels
            let barrel_length = 25.0;
            let barrel_offset = 4.0;
            let barrel_base_height = bunker_height + 2.0;
            
            // Calculate barrel direction
            let barrel_dir = Vec3::new(
                ground_turret.rotation.cos() * ground_turret.pitch.cos(),
                ground_turret.pitch.sin(),
                ground_turret.rotation.sin() * ground_turret.pitch.cos()
            );
            
            // Right vector for barrel separation
            let right = Vec3::new(-barrel_dir.z, 0.0, barrel_dir.x).normalize() * barrel_offset;
            let up = barrel_dir.cross(right).normalize() * 2.0;
            
            // Draw two barrels
            for side in [-1.0, 1.0] {
                let barrel_base = ground_turret.pos + Vec3::new(0.0, barrel_base_height, 0.0) + right * side;
                let barrel_end = barrel_base + barrel_dir * barrel_length;
                
                // Hexagonal barrel
                for i in 0..6 {
                    let angle1 = i as f32 * PI / 3.0;
                    let angle2 = (i + 1) as f32 * PI / 3.0;
                    
                    let offset1 = right * angle1.cos() * 1.5 + up * angle1.sin() * 1.5;
                    let offset2 = right * angle2.cos() * 1.5 + up * angle2.sin() * 1.5;
                    
                    renderer.draw_triangle(barrel_end, barrel_base + offset1, barrel_base + offset2);
                }
            }
            
            // Muzzle flash effect when firing
            if ground_turret.burst_count > 0 {
                let flash_size = 8.0;
                let flash_pos = ground_turret.pos + Vec3::new(0.0, barrel_base_height, 0.0) + barrel_dir * (barrel_length + 5.0);
                
                // Draw flash as star shape
                for i in 0..4 {
                    let angle = i as f32 * PI / 2.0;
                    let v1 = flash_pos + right * angle.cos() * flash_size + up * angle.sin() * flash_size;
                    let v2 = flash_pos + right * (angle + PI/4.0).cos() * flash_size * 0.5 + up * (angle + PI/4.0).sin() * flash_size * 0.5;
                    renderer.draw_triangle(flash_pos, v1, v2);
                }
            }
        }
    }
}


// Helper function to calculate angle difference
fn angle_difference(a: f32, b: f32) -> f32 {
    let diff = b - a;
    if diff > PI {
        diff - 2.0 * PI
    } else if diff < -PI {
        diff + 2.0 * PI
    } else {
        diff
    }
}