// Terrain generation module
// Central location for all terrain height calculations

use glam::Vec2;
use crate::biome::Biome;
use super::biome_heights::get_continental_height;

// Utility function matching GLSL smoothstep
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Public interface for terrain height calculation
pub fn height_at(x: f32, z: f32) -> f32 {
    // Match the GPU shader implementation exactly - no coordinate snapping
    get_blended_biome_height(Vec2::new(x, z))
}

// Get blended height combining multiple biomes - matches GPU shader exactly
fn get_blended_biome_height(p: Vec2) -> f32 {
    let sample_dist = 100.0;
    let h_center = get_shader_biome_height(p);
    let h_north = get_shader_biome_height(Vec2::new(p.x, p.y + sample_dist));
    let h_south = get_shader_biome_height(Vec2::new(p.x, p.y - sample_dist));
    let h_east = get_shader_biome_height(Vec2::new(p.x + sample_dist, p.y));
    let h_west = get_shader_biome_height(Vec2::new(p.x - sample_dist, p.y));
    
    let primary_height = (h_center * 3.0 + h_north + h_south + h_east + h_west) / 7.0;
    
    // Fractal noise for medium-scale detail - matches shader
    let mut fractal_noise = 0.0;
    let mut amplitude = 60.0;
    let mut frequency = 0.0005;
    for _ in 0..5 {
        fractal_noise += (p.x * frequency).sin() * (p.y * frequency).cos() * amplitude;
        fractal_noise += (p.x * frequency * 1.7 + 100.0).sin() * (p.y * frequency * 1.7 + 100.0).cos() * amplitude * 0.7;
        amplitude *= 0.45;
        frequency *= 2.3;
    }
    
    // Get large-scale continental height variation
    let continental = get_continental_height(p);
    
    // Apply continental modulation to biome heights
    let biome_noise = (p.x * 0.0003).sin() * (p.y * 0.0003).cos() * 0.5 + 
                     (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin() * 0.3;
    
    // Mountains and canyons are more affected by continental features
    let continental_influence = if biome_noise < -0.3 || biome_noise > 0.5 {
        1.5 // Stronger influence in extreme biomes
    } else {
        1.0
    };
    
    primary_height + fractal_noise + continental * continental_influence
}

// Get biome height matching the shader implementation
fn get_shader_biome_height(p: Vec2) -> f32 {
    let noise1 = (p.x * 0.0003).sin() * (p.y * 0.0003).cos();
    let noise2 = (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin();
    let noise3 = (p.x * 0.0013 - 2.1).cos() * (p.y * 0.0011 + 1.9).sin();
    
    let biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    let local_var = (p.x * 0.01).sin() * (p.y * 0.01).cos() * 0.1;
    let biome_noise = biome_noise + local_var;
    
    // Simplified biome selection matching shader
    if biome_noise < -0.3 {
        shader_canyon_height(p)
    } else if biome_noise < 0.1 {
        shader_plains_height(p)
    } else if biome_noise < 0.5 {
        shader_desert_height(p)
    } else {
        shader_mountain_height(p)
    }
}

// Simple biome selection based on position - used for tree placement etc
fn get_biome_at_simple(p: Vec2) -> Biome {
    let noise1 = (p.x * 0.0003).sin() * (p.y * 0.0003).cos();
    let noise2 = (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin();
    let noise3 = (p.x * 0.0013 - 2.1).cos() * (p.y * 0.0011 + 1.9).sin();
    
    let biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    let local_var = (p.x * 0.01).sin() * (p.y * 0.01).cos() * 0.1;
    let biome_noise = biome_noise + local_var;
    
    // Match shader biome selection
    if biome_noise < -0.3 {
        Biome::Canyon
    } else if biome_noise < 0.1 {
        Biome::Plains
    } else if biome_noise < 0.5 {
        Biome::Desert
    } else {
        Biome::Mountains
    }
}

// Public interface for getting biome at a position
pub fn get_biome_at(x: f32, z: f32) -> Biome {
    // For now, use the simple biome selection
    // In the future, this could use a BiomeMap instance
    get_biome_at_simple(Vec2::new(x, z))
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

// Plains height function matching the shader
fn shader_plains_height(p: Vec2) -> f32 {
    let scale1 = 0.002;
    let scale2 = 0.007;
    let scale3 = 0.015;
    
    let h1 = (p.x * scale1).sin() * (p.y * scale1).cos() * 40.0;
    let h2 = (p.x * scale2 + 100.0).sin() * (p.y * scale2 + 100.0).sin() * 20.0;
    let h3 = (p.x * scale3 + 200.0).cos() * (p.y * scale3 + 200.0).cos() * 10.0;
    
    let ridge = ((p.x * 0.0005 + p.y * 0.0003).sin().abs()).powf(3.0) * 30.0;
    let depression = smoothstep(0.6, 0.3, ((p.x * 0.0008).sin() * (p.y * 0.0006).cos()).abs()) * -20.0;
    
    h1 + h2 + h3 + ridge + depression
}

// Mountain height function matching the shader
fn shader_mountain_height(p: Vec2) -> f32 {
    let scale1 = 0.0008;
    let scale2 = 0.0015;
    let scale3 = 0.0003;
    let scale4 = 0.004;
    let scale5 = 0.0001;
    
    let range_angle: f32 = 0.4;
    let ridge_main = ((p.x * scale3 * range_angle.cos() + p.y * scale3 * range_angle.sin()).sin() * 0.5 + 0.5).powf(3.0);
    let ridge_secondary = ((p.x * scale3 * 1.2 - p.y * scale3 * 0.7).cos() * 0.5 + 0.5).powf(2.5);
    
    let peak_spacing = 0.0012;
    let peak_line1 = ((p.x * peak_spacing * range_angle.cos() + p.y * peak_spacing * range_angle.sin()).sin().powi(2) + 
                     (p.x * peak_spacing * range_angle.sin() - p.y * peak_spacing * range_angle.cos()).cos().powi(2)).sqrt();
    let peak_line2 = ((p.x * peak_spacing * 1.3 + 100.0).sin().powi(2) + 
                     (p.y * peak_spacing * 1.3 - 50.0).cos().powi(2)).sqrt();
    
    let peak_sharpness = 2.5;
    let h1 = (1.0 - peak_line1).max(0.0).powf(peak_sharpness) * 270.0;
    let h2 = (1.0 - peak_line2).max(0.0).powf(peak_sharpness * 0.8) * 225.0;
    
    let ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
    
    let valley_pattern = (p.x * scale2 + p.y * scale2 * 0.6).sin() + 
                        (p.x * scale2 * 0.8 - p.y * scale2 * 0.5).cos();
    let valley_depth = valley_pattern.abs().powf(2.0) * -60.0;
    
    let cliff_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 1.2).cos()).abs();
    let cliffs = cliff_pattern.powf(4.0) * 80.0;
    
    let glacial_valley = ((p.x * scale2 * 0.5 + p.y * scale2 * 0.7).sin().abs()).powf(0.5) * -40.0;
    let snow_cap = (h1 + h2 + ridge_height).max(225.0) * 0.2;
    
    let distance_to_ridge = ((ridge_main - 0.5).abs() + (ridge_secondary - 0.5).abs()).min(1.0);
    let foothill_height = (1.0 - distance_to_ridge).powf(0.5) * 60.0;
    
    let tectonic = ((p.x * scale5).sin() + (p.y * scale5 * 0.8).cos()) * 45.0;
    
    let height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                glacial_valley + snow_cap + foothill_height + tectonic;
                
    height.clamp(-250.0, 450.0)
}

// Canyon height function matching the shader
fn shader_canyon_height(p: Vec2) -> f32 {
    let scale1 = 0.0015;
    let scale2 = 0.003;
    let scale3 = 0.006;
    let scale4 = 0.0005;
    
    let river_flow = (p.x * scale1).sin() * 0.7 + (p.y * scale1 * 0.8).cos() * 0.5;
    let river_meander = ((p.x * scale1 * 0.5 + p.y * scale1 * 0.3).sin() + 
                        (p.x * scale1 * 0.3 - p.y * scale1 * 0.4).cos()) * 0.4;
    
    let tributary1 = ((p.x * scale2 - p.y * scale2 * 0.6).sin() + 
                     (p.x * scale2 * 0.4 + p.y * scale2).cos()) * 0.3;
    let tributary2 = ((p.x * scale2 * 1.2 + p.y * scale2 * 0.5).sin() * 
                     (p.x * scale2 * 0.8 - p.y * scale2 * 0.7).cos()) * 0.25;
    
    let confluence = ((tributary1 * river_flow).abs() + (tributary2 * river_flow).abs()) * 0.5;
    
    let width_pattern = (p.x * scale4 + p.y * scale4 * 0.7).sin();
    let canyon_width = 0.3 + width_pattern.abs() * 0.7 + confluence * 0.3;
    
    let pool_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs();
    let rapids = ((p.x * scale3 * 2.0 + p.y * scale3 * 1.5).sin().abs()).powf(3.0) * 0.3;
    
    let river_depth = (river_flow + river_meander).abs() * canyon_width + 
                     tributary1.abs() * 0.5 + tributary2.abs() * 0.5 + 
                     confluence + pool_pattern * 0.4 - rapids;
    
    let wall_slope = 1.5 + width_pattern * 0.5;
    let canyon_cut = river_depth.abs().powf(wall_slope) * 2.0;
    
    let terraces = ((canyon_cut * 6.0).floor() / 6.0).max(0.0);
    let final_depth = canyon_cut * 0.4 + terraces * 0.6;
    
    let mesa_height = ((p.x * scale4 * 0.5).sin().powi(2) + (p.y * scale4 * 0.5).cos().powi(2)) * 40.0;
    let plateau_base = 200.0;
    
    plateau_base + mesa_height - final_depth * 160.0
}

// Desert height function matching the shader
fn shader_desert_height(p: Vec2) -> f32 {
    let scale1 = 0.005;
    let scale2 = 0.01;
    let scale3 = 0.03;
    
    let dunes = ((p.x * scale1).sin() * (p.y * scale1 * 1.2).cos()).abs() * 80.0;
    let secondary = (p.x * scale2 + 30.0).cos() * (p.y * scale2 - 20.0).sin() * 30.0;
    let ripples = (p.x * scale3).sin() * (p.y * scale3).cos() * 10.0;
    let rocks = ((p.x * 0.002).sin() * (p.y * 0.002).cos()).abs().powf(4.0) * 60.0;
    let hollows = smoothstep(0.7, 0.5, ((p.x * 0.004 + p.y * 0.003).sin()).abs()) * -30.0;
    
    -20.0 + dunes + secondary + ripples + rocks + hollows
}