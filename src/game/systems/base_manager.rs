use rand::prelude::*;
use glam::Vec3;
use crate::base::{Base, BaseType};

pub struct BaseManager;

impl BaseManager {
    /// Generate initial bases around the map
    pub fn generate_initial_bases() -> Vec<Base> {
        let mut bases = Vec::new();
        let mut rng = thread_rng();
        
        // Smaller starting area - bases within 2000 units
        let base_count = 6; // Reduced from previous
        
        for _ in 0..base_count {
            let x = rng.gen_range(-1500.0..1500.0);
            let z = rng.gen_range(-1500.0..1500.0);
            
            let base_type = if rng.gen::<f32>() < 0.7 {
                BaseType::Basic
            } else {
                BaseType::Heavy
            };
            
            bases.push(Base::new(x, z, base_type));
        }
        
        bases
    }
    
    /// Find bases that need to spawn enemies
    pub fn get_bases_ready_to_spawn(bases: &[Base]) -> Vec<usize> {
        bases.iter()
            .enumerate()
            .filter(|(_, base)| base.is_active && base.can_spawn())
            .map(|(i, _)| i)
            .collect()
    }
    
    /// Update base spawn timers
    pub fn update_spawn_timers(bases: &mut [Base], dt: f32) {
        for base in bases.iter_mut() {
            base.update_spawn_timer(dt);
        }
    }
    
    /// Remove destroyed bases
    pub fn remove_destroyed_bases(bases: &mut Vec<Base>) {
        bases.retain(|base| base.is_active);
    }
    
    /// Check if all bases are destroyed
    pub fn all_bases_destroyed(bases: &[Base]) -> bool {
        bases.iter().all(|base| !base.is_active)
    }
}