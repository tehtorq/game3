use rand::prelude::*;
use crate::base::{Base, BaseType};
use glam::Vec3;

pub struct WaveManager {
    pub wave: u32,
    pub spawn_timer: f32,
}

impl WaveManager {
    pub fn new() -> Self {
        Self {
            wave: 1,
            spawn_timer: 3.0, // Start with 3 second delay
        }
    }
    
    pub fn update(&mut self, dt: f32) -> bool {
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.wave += 1;
            println!("Wave {} - Adding new bases", self.wave);
            // Spawn intervals: 30-60 seconds between new bases
            self.spawn_timer = (40.0 - (self.wave as f32 * 2.0)).max(20.0);
            true // Signal to spawn new wave
        } else {
            false
        }
    }
    
    /// Add bases for a new wave
    pub fn spawn_wave_bases(&self, player_pos: Vec3, existing_bases: &[Base]) -> Vec<Base> {
        let mut rng = thread_rng();
        let mut new_bases = Vec::new();
        
        // Different base patterns based on wave
        match self.wave % 5 {
            1 => {
                // Single forward base
                new_bases.extend(self.spawn_forward_bases(&mut rng, player_pos, 1, existing_bases));
            }
            2 => {
                // Flanking bases
                new_bases.extend(self.spawn_flanking_bases(&mut rng, player_pos, existing_bases));
            }
            3 => {
                // Defensive line
                new_bases.extend(self.spawn_defensive_line(&mut rng, player_pos, existing_bases));
            }
            4 => {
                // Encirclement attempt
                new_bases.extend(self.spawn_encirclement(&mut rng, player_pos, existing_bases));
            }
            0 => {
                // Boss wave - fortress
                new_bases.extend(self.spawn_fortress(&mut rng, player_pos, existing_bases));
            }
            _ => {}
        }
        
        new_bases
    }
    
    /// Spawn bases in front of the player
    fn spawn_forward_bases(&self, rng: &mut impl Rng, player_pos: Vec3, count: usize, existing_bases: &[Base]) -> Vec<Base> {
        let mut bases = Vec::new();
        
        for i in 0..count {
            let angle_offset = (i as f32 - (count as f32 - 1.0) / 2.0) * 0.3;
            let distance = rng.gen_range(2000.0..3000.0);
            let angle = rng.gen_range(-1.0..1.0) + angle_offset;
            
            let x = player_pos.x + angle.sin() * distance;
            let z = player_pos.z + angle.cos() * distance;
            
            if self.is_valid_base_position(Vec3::new(x, 0.0, z), existing_bases, &bases) {
                let base_type = self.select_base_type(rng);
                bases.push(Base::new(x, z, base_type));
            }
        }
        
        bases
    }
    
    /// Spawn bases on the flanks
    fn spawn_flanking_bases(&self, rng: &mut impl Rng, player_pos: Vec3, existing_bases: &[Base]) -> Vec<Base> {
        let mut bases = Vec::new();
        
        for side in [-1.0, 1.0] {
            let distance = rng.gen_range(1500.0..2500.0);
            let forward_offset = rng.gen_range(500.0..1500.0);
            
            let x = player_pos.x + side * distance;
            let z = player_pos.z + forward_offset;
            
            if self.is_valid_base_position(Vec3::new(x, 0.0, z), existing_bases, &bases) {
                let base_type = self.select_base_type(rng);
                bases.push(Base::new(x, z, base_type));
            }
        }
        
        bases
    }
    
    /// Spawn a defensive line of bases
    fn spawn_defensive_line(&self, rng: &mut impl Rng, player_pos: Vec3, existing_bases: &[Base]) -> Vec<Base> {
        let mut bases = Vec::new();
        let base_count = 3 + (self.wave / 5) as usize;
        let spacing = 400.0;
        let distance = rng.gen_range(2000.0..3000.0);
        
        for i in 0..base_count {
            let offset = (i as f32 - (base_count as f32 - 1.0) / 2.0) * spacing;
            let x = player_pos.x + offset;
            let z = player_pos.z + distance;
            
            if self.is_valid_base_position(Vec3::new(x, 0.0, z), existing_bases, &bases) {
                let base_type = if i == base_count / 2 {
                    BaseType::Fortress // Center is always fortress
                } else {
                    self.select_base_type(rng)
                };
                bases.push(Base::new(x, z, base_type));
            }
        }
        
        bases
    }
    
    /// Spawn bases in an encirclement pattern
    fn spawn_encirclement(&self, rng: &mut impl Rng, player_pos: Vec3, existing_bases: &[Base]) -> Vec<Base> {
        let mut bases = Vec::new();
        let base_count = 4 + (self.wave / 10) as usize;
        let radius = rng.gen_range(2500.0..3500.0);
        
        for i in 0..base_count {
            let angle = (i as f32 / base_count as f32) * std::f32::consts::TAU;
            let x = player_pos.x + angle.cos() * radius;
            let z = player_pos.z + angle.sin() * radius;
            
            if self.is_valid_base_position(Vec3::new(x, 0.0, z), existing_bases, &bases) {
                let base_type = self.select_base_type(rng);
                bases.push(Base::new(x, z, base_type));
            }
        }
        
        bases
    }
    
    /// Spawn a fortress formation
    fn spawn_fortress(&self, rng: &mut impl Rng, player_pos: Vec3, existing_bases: &[Base]) -> Vec<Base> {
        let mut bases = Vec::new();
        let distance = rng.gen_range(2500.0..3500.0);
        
        // Central fortress
        let x = player_pos.x;
        let z = player_pos.z + distance;
        
        if self.is_valid_base_position(Vec3::new(x, 0.0, z), existing_bases, &bases) {
            bases.push(Base::new(x, z, BaseType::Fortress));
            
            // Supporting bases around the fortress
            for angle in [0.0f32, 90.0, 180.0, 270.0] {
                let rad = angle.to_radians();
                let support_x = x + rad.cos() * 300.0;
                let support_z = z + rad.sin() * 300.0;
                
                if self.is_valid_base_position(Vec3::new(support_x, 0.0, support_z), existing_bases, &bases) {
                    bases.push(Base::new(support_x, support_z, BaseType::Outpost));
                }
            }
        }
        
        bases
    }
    
    /// Select base type based on wave
    fn select_base_type(&self, rng: &mut impl Rng) -> BaseType {
        let roll = rng.gen_range(0..100);
        
        match self.wave {
            1..=3 => BaseType::Basic,
            4..=6 => {
                if roll < 70 {
                    BaseType::Basic
                } else {
                    BaseType::Heavy
                }
            }
            7..=10 => {
                if roll < 40 {
                    BaseType::Basic
                } else if roll < 80 {
                    BaseType::Heavy
                } else {
                    BaseType::Shielded
                }
            }
            11..=15 => {
                if roll < 20 {
                    BaseType::Basic
                } else if roll < 50 {
                    BaseType::Heavy
                } else if roll < 80 {
                    BaseType::Shielded
                } else {
                    BaseType::Fortress
                }
            }
            _ => {
                if roll < 10 {
                    BaseType::Basic
                } else if roll < 30 {
                    BaseType::Heavy
                } else if roll < 60 {
                    BaseType::Shielded
                } else if roll < 90 {
                    BaseType::Fortress
                } else {
                    BaseType::Outpost
                }
            }
        }
    }
    
    /// Check if a position is valid for a new base
    fn is_valid_base_position(&self, pos: Vec3, existing_bases: &[Base], new_bases: &[Base]) -> bool {
        let min_distance = 300.0;
        
        // Check against existing bases
        for base in existing_bases {
            if (base.pos - pos).length() < min_distance {
                return false;
            }
        }
        
        // Check against newly planned bases
        for base in new_bases {
            if (base.pos - pos).length() < min_distance {
                return false;
            }
        }
        
        true
    }
}