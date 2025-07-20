// Terrain generation module
// Central location for all terrain height calculations

use glam::Vec2;
use crate::biome::Biome;
use super::biome_heights::*;

// Public interface for terrain height calculation
pub fn height_at(x: f32, z: f32) -> f32 {
    // Snap coordinates to fixed precision to ensure consistent results
    // This prevents gaps between chunks due to floating point differences
    let snapped_x = (x * 1000.0).round() / 1000.0;
    let snapped_z = (z * 1000.0).round() / 1000.0;
    get_blended_biome_height(Vec2::new(snapped_x, snapped_z))
}

// Get blended height combining multiple biomes
fn get_blended_biome_height(p: Vec2) -> f32 {
    // Get base biome height
    let biome = get_biome_at_simple(p);
    let base_height = get_biome_height(p, biome);
    
    // Add continental scale height variation
    let continental = get_continental_height(p);
    
    // Different biomes react differently to continental features
    let biome_noise = (p.x * 0.0003).sin() * (p.y * 0.0003).cos() * 0.5 + 
                     (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin() * 0.3;
    
    let continental_influence = match biome {
        Biome::Mountains | Biome::Canyon => 1.5,  // Strong influence
        Biome::Arctic | Biome::Volcanic => 2.0,   // Very strong influence
        Biome::Plains | Biome::Desert => 0.7,     // Weaker influence
        _ => 1.0,                                  // Normal influence
    };
    
    // Add medium-scale fractal noise
    let mut fractal_noise = 0.0;
    let mut amplitude = 60.0;
    let mut frequency = 0.0005;
    for _ in 0..5 {
        fractal_noise += (p.x * frequency).sin() * (p.y * frequency).cos() * amplitude;
        fractal_noise += (p.x * frequency * 1.7 + 100.0).sin() * (p.y * frequency * 1.7 + 100.0).cos() * amplitude * 0.7;
        amplitude *= 0.45;
        frequency *= 2.3;
    }
    
    base_height + fractal_noise + continental * continental_influence
}

// Simple biome selection based on position
fn get_biome_at_simple(p: Vec2) -> Biome {
    // Create a simple pattern of biomes
    let scale = 0.0001;
    let noise = (p.x * scale).sin() * (p.y * scale).cos();
    
    if noise > 0.7 {
        Biome::Mountains
    } else if noise > 0.4 {
        Biome::Plateau
    } else if noise > 0.1 {
        Biome::Plains
    } else if noise > -0.2 {
        Biome::Desert
    } else if noise > -0.5 {
        Biome::Canyon
    } else {
        Biome::Arctic
    }
}

// Get height for a specific biome
fn get_biome_height(p: Vec2, biome: Biome) -> f32 {
    match biome {
        Biome::Canyon => canyon_height(p),
        Biome::Plateau => plateau_height(p),
        Biome::Arctic => arctic_height(p),
        Biome::Plains => grassland_height(p),  // Map Plains to grassland
        Biome::Volcanic => volcanic_height(p),
        Biome::Desert => desert_height(p),
        Biome::Mountains => plateau_height(p) * 1.5,  // Use scaled plateau for mountains
        Biome::Crystalline => crystal_height(p),
        Biome::Badlands => canyon_height(p) * 0.7 + desert_height(p) * 0.3,  // Mix canyon and desert
        Biome::Floating => floating_height(p),
        Biome::Caverns => void_height(p),  // Use void for caverns
        Biome::Swamp => ocean_height(p) * 0.5 + grassland_height(p) * 0.5,  // Mix ocean and grassland
    }
}

// Additional utility functions for terrain generation

/// Get the normal vector at a given position
pub fn get_normal_at(x: f32, z: f32) -> Vec2 {
    let epsilon = 1.0;
    let h_center = height_at(x, z);
    let h_right = height_at(x + epsilon, z);
    let h_up = height_at(x, z + epsilon);
    
    let dx = h_right - h_center;
    let dz = h_up - h_center;
    
    Vec2::new(-dx / epsilon, -dz / epsilon).normalize_or_zero()
}

/// Check if a position is underwater
pub fn is_underwater(x: f32, z: f32) -> bool {
    height_at(x, z) < super::WATER_LEVEL
}

/// Get terrain slope at position (0-1 range)
pub fn get_slope_at(x: f32, z: f32) -> f32 {
    let normal = get_normal_at(x, z);
    // Dot product with up vector gives us the slope
    1.0 - normal.y.abs()
}

/// Get biome transition strength (0-1) indicating how much we're in a transition zone
pub fn get_biome_blend_factor(x: f32, z: f32) -> f32 {
    let biome_scale = 8192.0;
    let fx = (x / biome_scale).fract();
    let fy = (z / biome_scale).fract();
    
    // Distance from biome center (0.5, 0.5)
    let dx = (fx - 0.5).abs() * 2.0;
    let dy = (fy - 0.5).abs() * 2.0;
    
    // Use the maximum distance to get blend factor
    let dist = dx.max(dy);
    
    // Smooth transition zone starts at 0.7 from center
    smoothstep(0.7, 1.0, dist)
}