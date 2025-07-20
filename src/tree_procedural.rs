use glam::Vec3;
use crate::tree::{Tree, TreeType};
use crate::terrain;

/// Procedural tree generation system that creates trees deterministically based on position
pub struct ProceduralTreeSystem {
    chunk_size: f32,
    tree_spacing: f32,
    view_distance: f32,
}

impl ProceduralTreeSystem {
    pub fn new() -> Self {
        Self {
            chunk_size: 512.0,      // Larger chunks for fewer iterations
            tree_spacing: 60.0,     // Slightly more spacing to reduce tree count
            view_distance: 2500.0,  // Reduced view distance for performance
        }
    }
    
    /// Generate all trees visible from the given position
    pub fn generate_visible_trees(&self, viewer_pos: Vec3) -> Vec<Tree> {
        let mut trees = Vec::with_capacity(2000); // Pre-allocate for performance
        
        // Calculate chunk bounds
        let min_chunk_x = ((viewer_pos.x - self.view_distance) / self.chunk_size).floor() as i32;
        let max_chunk_x = ((viewer_pos.x + self.view_distance) / self.chunk_size).ceil() as i32;
        let min_chunk_z = ((viewer_pos.z - self.view_distance) / self.chunk_size).floor() as i32;
        let max_chunk_z = ((viewer_pos.z + self.view_distance) / self.chunk_size).ceil() as i32;
        
        // Process chunks in spiral order from center outward
        let center_chunk_x = (viewer_pos.x / self.chunk_size).round() as i32;
        let center_chunk_z = (viewer_pos.z / self.chunk_size).round() as i32;
        
        // First add trees from center chunk
        let chunk_trees = self.generate_trees_for_chunk(center_chunk_x, center_chunk_z, viewer_pos);
        trees.extend(chunk_trees);
        
        // Then spiral outward
        let max_radius = ((max_chunk_x - min_chunk_x).max(max_chunk_z - min_chunk_z) / 2) + 1;
        
        for radius in 1..=max_radius {
            // Top edge
            for x in -radius..=radius {
                let chunk_x = center_chunk_x + x;
                let chunk_z = center_chunk_z - radius;
                if chunk_x >= min_chunk_x && chunk_x <= max_chunk_x && chunk_z >= min_chunk_z && chunk_z <= max_chunk_z {
                    let chunk_trees = self.generate_trees_for_chunk(chunk_x, chunk_z, viewer_pos);
                    trees.extend(chunk_trees);
                }
            }
            
            // Right edge
            for z in -radius + 1..=radius {
                let chunk_x = center_chunk_x + radius;
                let chunk_z = center_chunk_z + z;
                if chunk_x >= min_chunk_x && chunk_x <= max_chunk_x && chunk_z >= min_chunk_z && chunk_z <= max_chunk_z {
                    let chunk_trees = self.generate_trees_for_chunk(chunk_x, chunk_z, viewer_pos);
                    trees.extend(chunk_trees);
                }
            }
            
            // Bottom edge
            for x in (-radius..radius).rev() {
                let chunk_x = center_chunk_x + x;
                let chunk_z = center_chunk_z + radius;
                if chunk_x >= min_chunk_x && chunk_x <= max_chunk_x && chunk_z >= min_chunk_z && chunk_z <= max_chunk_z {
                    let chunk_trees = self.generate_trees_for_chunk(chunk_x, chunk_z, viewer_pos);
                    trees.extend(chunk_trees);
                }
            }
            
            // Left edge
            for z in (-radius + 1..radius).rev() {
                let chunk_x = center_chunk_x - radius;
                let chunk_z = center_chunk_z + z;
                if chunk_x >= min_chunk_x && chunk_x <= max_chunk_x && chunk_z >= min_chunk_z && chunk_z <= max_chunk_z {
                    let chunk_trees = self.generate_trees_for_chunk(chunk_x, chunk_z, viewer_pos);
                    trees.extend(chunk_trees);
                }
            }
        }
        
        trees
    }
    
    /// Generate trees for a specific chunk
    fn generate_trees_for_chunk(&self, chunk_x: i32, chunk_z: i32, viewer_pos: Vec3) -> Vec<Tree> {
        let mut trees = Vec::new();
        
        let chunk_world_x = chunk_x as f32 * self.chunk_size;
        let chunk_world_z = chunk_z as f32 * self.chunk_size;
        
        // Early chunk culling - check if chunk center is in range
        let chunk_center_x = chunk_world_x + self.chunk_size * 0.5;
        let chunk_center_z = chunk_world_z + self.chunk_size * 0.5;
        let dx = chunk_center_x - viewer_pos.x;
        let dz = chunk_center_z - viewer_pos.z;
        let chunk_dist_sq = dx * dx + dz * dz;
        
        // Skip entire chunk if too far (with some margin for chunk size)
        let max_dist = self.view_distance + self.chunk_size;
        if chunk_dist_sq > max_dist * max_dist {
            return trees;
        }
        
        // Use chunk coordinates as seed for deterministic generation
        let chunk_seed = hash_coords(chunk_x, chunk_z);
        
        // Grid within chunk
        let trees_per_chunk = (self.chunk_size / self.tree_spacing) as i32;
        
        for local_x in 0..trees_per_chunk {
            for local_z in 0..trees_per_chunk {
                // Calculate tree position with jitter
                let base_x = chunk_world_x + local_x as f32 * self.tree_spacing;
                let base_z = chunk_world_z + local_z as f32 * self.tree_spacing;
                
                // Add deterministic jitter
                let jitter_seed = hash_coords(
                    chunk_x * 1000 + local_x,
                    chunk_z * 1000 + local_z
                );
                let jitter_x = (pseudo_random(jitter_seed) - 0.5) * self.tree_spacing * 0.8;
                let jitter_z = (pseudo_random(jitter_seed + 1) - 0.5) * self.tree_spacing * 0.8;
                
                let tree_x = base_x + jitter_x;
                let tree_z = base_z + jitter_z;
                
                // Check distance to viewer for culling
                let dx = tree_x - viewer_pos.x;
                let dz = tree_z - viewer_pos.z;
                let dist_sq = dx * dx + dz * dz;
                
                if dist_sq > self.view_distance * self.view_distance {
                    continue;
                }
                
                // LOD: Skip some trees at distance
                let dist = dist_sq.sqrt();
                if dist > 1500.0 {
                    // Skip 50% of distant trees
                    let skip_seed = hash_coords(local_x * 13, local_z * 17);
                    if pseudo_random(skip_seed) < 0.5 {
                        continue;
                    }
                } else if dist > 2000.0 {
                    // Skip 75% of very distant trees
                    let skip_seed = hash_coords(local_x * 13, local_z * 17);
                    if pseudo_random(skip_seed) < 0.75 {
                        continue;
                    }
                }
                
                // Determine if tree should spawn here
                if let Some(tree) = self.should_spawn_tree_at(tree_x, tree_z, chunk_seed + local_x + local_z * 100) {
                    trees.push(tree);
                }
            }
        }
        
        trees
    }
    
    /// Determine if a tree should spawn at the given position
    fn should_spawn_tree_at(&self, x: f32, z: f32, seed: i32) -> Option<Tree> {
        // Get terrain height
        let height = terrain::height_at(x, z);
        
        // Don't spawn underwater
        if height < -20.0 {
            return None;
        }
        
        // Calculate slope
        let delta = 5.0;
        let h_n = terrain::height_at(x, z + delta);
        let h_s = terrain::height_at(x, z - delta);
        let h_e = terrain::height_at(x + delta, z);
        let h_w = terrain::height_at(x - delta, z);
        let slope = ((h_n - h_s).abs() + (h_e - h_w).abs()) / (2.0 * delta);
        
        // Don't spawn on steep slopes
        if slope > 0.5 {
            return None;
        }
        
        // Get biome
        let biome = terrain::get_biome_at(x, z);
        
        // Determine spawn chance based on biome
        let spawn_chance = match biome {
            crate::biome::Biome::Plains => 0.3,
            crate::biome::Biome::Mountains => 0.2,
            crate::biome::Biome::Arctic => 0.15,
            crate::biome::Biome::Desert => 0.05,
            crate::biome::Biome::Swamp => 0.4,
            crate::biome::Biome::Crystalline => 0.1,
            crate::biome::Biome::Badlands => 0.05,
            _ => 0.0,
        };
        
        // Use deterministic random based on position
        let random_value = pseudo_random(seed);
        if random_value >= spawn_chance {
            return None;
        }
        
        // Select tree type based on biome
        let tree_type = match biome {
            crate::biome::Biome::Plains => TreeType::Oak,
            crate::biome::Biome::Mountains => TreeType::Pine,
            crate::biome::Biome::Desert => TreeType::Cactus,
            crate::biome::Biome::Arctic => TreeType::Pine,
            crate::biome::Biome::Swamp => TreeType::Mushroom,
            crate::biome::Biome::Crystalline => TreeType::Crystal,
            crate::biome::Biome::Badlands => TreeType::Dead,
            _ => return None,
        };
        
        // Create tree with deterministic properties
        let tree_seed = hash_coords(x as i32, z as i32);
        let height_variation = pseudo_random(tree_seed) * 0.4 + 0.8; // 0.8 to 1.2
        let radius_variation = pseudo_random(tree_seed + 1) * 0.4 + 0.8;
        let rotation = pseudo_random(tree_seed + 2) * std::f32::consts::PI * 2.0;
        
        let (base_height, base_radius) = match tree_type {
            TreeType::Pine => (40.0, 10.0),
            TreeType::Oak => (32.0, 18.0),
            TreeType::Palm => (27.0, 4.0),
            TreeType::Crystal => (30.0, 6.0),
            TreeType::Cactus => (15.0, 5.0),
            TreeType::Mushroom => (15.0, 12.0),
            TreeType::Dead => (20.0, 7.0),
        };
        
        Some(Tree {
            pos: Vec3::new(x, height, z),
            height: base_height * height_variation,
            radius: base_radius * radius_variation,
            tree_type,
            rotation,
        })
    }
}

/// Simple hash function for coordinates
fn hash_coords(x: i32, z: i32) -> i32 {
    let mut hash = x.wrapping_mul(73856093) ^ z.wrapping_mul(19349663);
    hash = hash.wrapping_mul(hash);
    hash ^= hash >> 16;
    hash
}

/// Pseudo-random number generator (0.0 to 1.0)
fn pseudo_random(seed: i32) -> f32 {
    let x = (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 0x7fff;
    x as f32 / 32767.0
}