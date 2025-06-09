use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Biome {
    Plains,       // Gentle rolling hills (default/current terrain)
    Canyon,       // Deep cuts with steep walls
    Plateau,      // Flat elevated areas with cliff edges
    Crystalline,  // Spiky crystal formations
    Volcanic,     // Rough terrain with lava flows
    Mountains,    // Tall peaks and valleys
    Desert,       // Dune-like formations with ripples
    Arctic,       // Icy spikes and crevasses
    Badlands,     // Eroded pillars and mesas
    Floating,     // Suspended islands and platforms
    Caverns,      // Pockmarked with holes and caves
    Swamp,        // Low, undulating wetlands
}

impl Biome {
    // Get the dominant color tint for each biome
    pub fn color_tint(&self) -> [f32; 3] {
        match self {
            Biome::Plains => [0.0, 1.0, 0.0],        // Green
            Biome::Canyon => [0.8, 0.5, 0.2],        // Orange-brown
            Biome::Plateau => [0.6, 0.6, 0.4],       // Sandy beige
            Biome::Crystalline => [0.6, 0.3, 1.0],   // Purple
            Biome::Volcanic => [1.0, 0.3, 0.0],      // Orange-red
            Biome::Mountains => [0.7, 0.7, 0.8],     // Blue-grey
            Biome::Desert => [1.0, 0.9, 0.5],        // Yellow-sand
            Biome::Arctic => [0.8, 0.9, 1.0],        // Ice blue
            Biome::Badlands => [0.9, 0.4, 0.2],      // Red-orange
            Biome::Floating => [0.3, 0.8, 1.0],      // Sky blue
            Biome::Caverns => [0.4, 0.3, 0.5],       // Dark purple
            Biome::Swamp => [0.2, 0.6, 0.3],         // Dark green
        }
    }
    
    // Height generation parameters for each biome
    pub fn height_params(&self) -> BiomeHeightParams {
        match self {
            Biome::Plains => BiomeHeightParams {
                base_amplitude: 60.0,
                frequency_multiplier: 1.0,
                roughness: 0.3,
                min_height: -100.0,
                max_height: 100.0,
            },
            Biome::Canyon => BiomeHeightParams {
                base_amplitude: 150.0,
                frequency_multiplier: 1.5,
                roughness: 0.1,
                min_height: -300.0,
                max_height: 50.0,
            },
            Biome::Plateau => BiomeHeightParams {
                base_amplitude: 40.0,
                frequency_multiplier: 0.5,
                roughness: 0.1,
                min_height: 200.0,
                max_height: 300.0,
            },
            Biome::Crystalline => BiomeHeightParams {
                base_amplitude: 100.0,
                frequency_multiplier: 3.0,
                roughness: 0.9,
                min_height: -50.0,
                max_height: 250.0,
            },
            Biome::Volcanic => BiomeHeightParams {
                base_amplitude: 80.0,
                frequency_multiplier: 2.0,
                roughness: 0.7,
                min_height: -50.0,
                max_height: 150.0,
            },
            Biome::Mountains => BiomeHeightParams {
                base_amplitude: 200.0,
                frequency_multiplier: 0.8,
                roughness: 0.5,
                min_height: -100.0,
                max_height: 400.0,
            },
            Biome::Desert => BiomeHeightParams {
                base_amplitude: 50.0,
                frequency_multiplier: 2.5,
                roughness: 0.2,
                min_height: -20.0,
                max_height: 120.0,
            },
            Biome::Arctic => BiomeHeightParams {
                base_amplitude: 120.0,
                frequency_multiplier: 1.8,
                roughness: 0.8,
                min_height: 50.0,
                max_height: 300.0,
            },
            Biome::Badlands => BiomeHeightParams {
                base_amplitude: 90.0,
                frequency_multiplier: 2.2,
                roughness: 0.9,
                min_height: -150.0,
                max_height: 200.0,
            },
            Biome::Floating => BiomeHeightParams {
                base_amplitude: 60.0,
                frequency_multiplier: 0.3,
                roughness: 0.1,
                min_height: 150.0,
                max_height: 350.0,
            },
            Biome::Caverns => BiomeHeightParams {
                base_amplitude: 100.0,
                frequency_multiplier: 3.5,
                roughness: 1.0,
                min_height: -200.0,
                max_height: 100.0,
            },
            Biome::Swamp => BiomeHeightParams {
                base_amplitude: 30.0,
                frequency_multiplier: 4.0,
                roughness: 0.6,
                min_height: -50.0,
                max_height: 20.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiomeHeightParams {
    pub base_amplitude: f32,
    pub frequency_multiplier: f32,
    pub roughness: f32,
    pub min_height: f32,
    pub max_height: f32,
}

pub struct BiomeMap {
    // Biome centers for Voronoi-based regions
    biome_centers: Vec<(Vec2, Biome)>,
}

impl BiomeMap {
    pub fn new(world_size: f32, seed: u32) -> Self {
        let mut biome_centers = Vec::new();
        
        // Use smaller regions for more diversity
        let region_size = 3000.0; // Smaller biome regions
        let half_world = world_size * 0.5;
        
        // Place biome centers in a grid pattern with some offset
        let grid_count = (world_size / region_size) as i32;
        let mut biome_index = 0;
        let biomes = [
            Biome::Plains,
            Biome::Canyon,
            Biome::Plateau,
            Biome::Crystalline,
            Biome::Volcanic,
            Biome::Mountains,
            Biome::Desert,
            Biome::Arctic,
            Biome::Badlands,
            Biome::Floating,
            Biome::Caverns,
            Biome::Swamp,
        ];
        
        // Create a more organic distribution with multiple scales
        for i in 0..grid_count {
            for j in 0..grid_count {
                let base_x = -half_world + (i as f32 + 0.5) * region_size;
                let base_z = -half_world + (j as f32 + 0.5) * region_size;
                
                // Multi-scale noise for more organic placement
                let noise1 = ((seed + i as u32 * 7 + j as u32 * 13) % 1000) as f32 / 1000.0;
                let noise2 = ((seed + i as u32 * 23 + j as u32 * 31) % 1000) as f32 / 1000.0;
                let noise3 = ((seed + i as u32 * 47 + j as u32 * 53) % 1000) as f32 / 1000.0;
                
                // Combine noise at different scales
                let offset_x = (noise1 - 0.5) * 0.8 + (noise2 - 0.5) * 0.3;
                let offset_z = (noise2 - 0.5) * 0.8 + (noise3 - 0.5) * 0.3;
                
                let x = base_x + offset_x * region_size * 0.6;
                let z = base_z + offset_z * region_size * 0.6;
                
                // Use noise to select biome type for more organic distribution
                let biome_selector = (noise1 + noise2 + noise3) / 3.0;
                let biome_idx = (biome_selector * biomes.len() as f32) as usize % biomes.len();
                let biome = biomes[biome_idx];
                
                biome_centers.push((Vec2::new(x, z), biome));
                
                // Add some extra random points for more variation
                if noise1 > 0.7 {
                    let extra_x = x + (noise2 - 0.5) * region_size * 0.5;
                    let extra_z = z + (noise3 - 0.5) * region_size * 0.5;
                    let extra_biome = biomes[(biome_idx + 3) % biomes.len()];
                    biome_centers.push((Vec2::new(extra_x, extra_z), extra_biome));
                }
                
                biome_index += 1;
            }
        }
        
        Self { biome_centers }
    }
    
    // Get biome at a specific world position
    pub fn get_biome_at(&self, x: f32, z: f32) -> Biome {
        let pos = Vec2::new(x, z);
        
        // Find closest biome center (Voronoi)
        let mut closest_biome = Biome::Plains;
        let mut closest_distance = f32::MAX;
        
        for (center, biome) in &self.biome_centers {
            let distance = pos.distance(*center);
            if distance < closest_distance {
                closest_distance = distance;
                closest_biome = *biome;
            }
        }
        
        closest_biome
    }
    
    // Get biome influence at a position (for smooth transitions)
    pub fn get_biome_weights(&self, x: f32, z: f32) -> Vec<(Biome, f32)> {
        let pos = Vec2::new(x, z);
        let mut distances: Vec<(Biome, f32)> = Vec::new();
        
        // Calculate distances to all biome centers
        for (center, biome) in &self.biome_centers {
            let distance = pos.distance(*center);
            distances.push((*biome, distance));
        }
        
        // Sort by distance
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Take the 3 closest biomes for blending
        let mut weights = Vec::new();
        if distances.len() >= 3 {
            let d1 = distances[0].1;
            let d2 = distances[1].1;
            let d3 = distances[2].1;
            
            // Convert distances to weights using inverse distance weighting
            let total = 1.0 / d1 + 1.0 / d2 + 1.0 / d3;
            
            weights.push((distances[0].0, (1.0 / d1) / total));
            weights.push((distances[1].0, (1.0 / d2) / total));
            weights.push((distances[2].0, (1.0 / d3) / total));
        } else if !distances.is_empty() {
            weights.push((distances[0].0, 1.0));
        }
        
        weights
    }
}