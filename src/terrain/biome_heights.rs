// Biome height calculation functions
// This file contains the canonical implementations of biome height functions
// These are shared between CPU and GPU implementations

use glam::Vec2;
use std::f32::consts::PI;

// Canyon biome height function
pub fn canyon_height(p: Vec2) -> f32 {
    // River-like canyon systems with dramatic depth variations
    let scale1 = 0.0015; // Main river course
    let scale2 = 0.003;  // Tributaries
    let scale3 = 0.006;  // Rapids and falls
    let scale4 = 0.0005; // Canyon width variation
    
    // Main river channel - continuous flowing pattern
    let river_flow = (p.x * scale1).sin() * 0.7 + (p.y * scale1 * 0.8).cos() * 0.5;
    let river_meander = ((p.x * scale1 * 0.5 + p.y * scale1 * 0.3).sin() + 
                        (p.x * scale1 * 0.3 - p.y * scale1 * 0.4).cos()) * 0.4;
    
    // Tributary channels joining the main river
    let tributary1 = ((p.x * scale2 - p.y * scale2 * 0.6).sin() + 
                     (p.x * scale2 * 0.4 + p.y * scale2).cos()) * 0.3;
    let tributary2 = ((p.x * scale2 * 1.2 + p.y * scale2 * 0.5).sin() * 
                     (p.x * scale2 * 0.8 - p.y * scale2 * 0.7).cos()) * 0.25;
    
    // River confluence points - deeper where rivers meet
    let confluence = (tributary1 * river_flow).abs() + (tributary2 * river_flow).abs() * 0.5;
    
    // Canyon width varies like a real river
    let width_pattern = (p.x * scale4 + p.y * scale4 * 0.7).sin();
    let canyon_width = 0.3 + width_pattern.abs() * 0.7 + confluence * 0.3;
    
    // River depth with pools and rapids
    let pool_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs();
    let rapids = ((p.x * scale3 * 2.0 + p.y * scale3 * 1.5).sin().abs()).powf(3.0) * 0.3;
    
    // Combine all river features
    let river_depth = (river_flow + river_meander).abs() * canyon_width + 
                     tributary1.abs() * 0.5 + tributary2.abs() * 0.5 + 
                     confluence + pool_pattern * 0.4 - rapids;
    
    // Create dramatic canyon walls with overhangs
    let wall_slope = 1.5 + width_pattern * 0.5;
    let canyon_cut = river_depth.abs().powf(wall_slope) * 2.0; // Deeper canyons
    
    // Terraced canyon walls
    let terraces = (canyon_cut * 6.0).floor() / 6.0;
    let final_depth = canyon_cut * 0.4 + terraces * 0.6;
    
    // High mesas between canyons
    let mesa_height = ((p.x * scale4 * 0.5).sin().powf(2.0) + (p.y * scale4 * 0.5).cos().powf(2.0)) * 40.0;
    let plateau_base = 200.0; // Base height for dramatic effect
    
    // Create dramatic height difference
    plateau_base + mesa_height - final_depth * 160.0
}

pub fn plateau_height(p: Vec2) -> f32 {
    // Dramatic mesa and plateau formations with sheer cliffs
    let scale1 = 0.0008;
    let scale2 = 0.0005;
    let scale3 = 0.002;
    let scale4 = 0.0003;
    
    // Create distinct mesa formations
    let mesa1 = ((p.x * scale1).sin() * (p.y * scale1 * 0.9).cos()).abs();
    let mesa2 = ((p.x * scale2 + 200.0).sin() * (p.y * scale2 - 150.0).cos()).abs();
    let mesa3 = ((p.x * scale4 * 1.3).cos() * (p.y * scale4 + 100.0).sin()).abs();
    
    // Sharp cliff edges
    let cliff_sharpness = 8.0; // Very sharp transitions
    let mesa_top1 = if mesa1 > 0.4 { 1.0 } else { (mesa1 / 0.4).powf(cliff_sharpness) };
    let mesa_top2 = if mesa2 > 0.5 { 1.0 } else { (mesa2 / 0.5).powf(cliff_sharpness) };
    let mesa_top3 = if mesa3 > 0.6 { 1.0 } else { (mesa3 / 0.6).powf(cliff_sharpness) };
    
    // Dramatic height differences between plateau levels
    let base_elevation = -50.0;
    let tier1_height = 120.0;
    let tier2_height = 180.0;
    let tier3_height = 250.0;
    
    // Calculate mesa heights
    let h1 = base_elevation + mesa_top1 * tier1_height;
    let h2 = base_elevation + mesa_top2 * tier2_height;
    let h3 = base_elevation + mesa_top3 * tier3_height;
    
    // Natural bridges and arches
    let arch_pattern = ((p.x * scale3 + p.y * scale3 * 0.7).sin() * 
                       (p.x * scale3 * 1.2 - p.y * scale3 * 0.5).cos()).abs();
    let arch_cut = if arch_pattern > 0.7 { arch_pattern.powf(4.0) * -50.0 } else { 0.0 };
    
    // Rock spires and hoodoos
    let spire_pattern = ((p.x * scale3 * 2.0).sin() * (p.y * scale3 * 2.0).cos()).abs();
    let spire_height = if spire_pattern > 0.8 { spire_pattern.powf(3.0) * 60.0 } else { 0.0 };
    
    // Boulder fields and rockfalls
    let boulder_noise = ((p.x * 0.01).sin() * (p.y * 0.01).cos()).abs() * 15.0;
    
    // Combine features with proper layering
    h1.max(h2).max(h3) + spire_height + arch_cut + boulder_noise
}

pub fn arctic_height(p: Vec2) -> f32 {
    // Icy glacial terrain with crevasses and ice shelves
    let scale1 = 0.001;
    let scale2 = 0.002;
    let scale3 = 0.005;
    let scale4 = 0.0003;
    
    // Glacial flow patterns
    let flow_angle = (p.y * scale4).sin() * 0.3;
    let flow_x = p.x * flow_angle.cos() - p.y * flow_angle.sin();
    let flow_y = p.x * flow_angle.sin() + p.y * flow_angle.cos();
    
    // Ice sheet base
    let ice_sheet = ((flow_x * scale1).sin() + (flow_y * scale1 * 0.7).cos()) * 40.0;
    let ice_ridges = ((flow_x * scale2).sin() * (flow_y * scale2).cos()).abs() * 60.0;
    
    // Deep crevasses
    let crevasse1 = ((flow_x * scale3).sin() * 4.0).sin();
    let crevasse2 = ((flow_y * scale3 * 1.2).cos() * 3.5).cos();
    let crevasse_depth = (crevasse1.abs() + crevasse2.abs()).max(1.0).ln() * -30.0;
    
    // Pressure ridges where ice sheets collide
    let pressure_ridge = ((p.x * scale2 + p.y * scale2 * 0.6).sin().abs() + 
                         (p.x * scale2 * 0.8 - p.y * scale2).cos().abs()).powf(2.0) * 80.0;
    
    // Icebergs and seracs
    let serac_pattern = ((p.x * scale3 * 2.0).sin() * (p.y * scale3 * 2.0).cos()).abs();
    let serac_height = if serac_pattern > 0.7 { serac_pattern * 100.0 } else { 0.0 };
    
    // Ice caves and tunnels
    let cave_pattern = ((p.x * 0.008).sin() + (p.y * 0.008).cos()).abs();
    let ice_caves = if cave_pattern < 0.3 { cave_pattern * -50.0 } else { 0.0 };
    
    // Combine all features
    let base = 100.0; // Arctic plateau base height
    base + ice_sheet + ice_ridges + crevasse_depth + pressure_ridge + serac_height + ice_caves
}

pub fn grassland_height(p: Vec2) -> f32 {
    // Gently rolling hills with occasional rocky outcrops
    let scale1 = 0.0008;
    let scale2 = 0.0015;
    let scale3 = 0.003;
    let scale4 = 0.0001;
    
    // Rolling hills - main terrain
    let hills1 = (p.x * scale1).sin() * (p.y * scale1 * 0.9).cos() * 25.0;
    let hills2 = (p.x * scale2 * 0.7 + 100.0).sin() * (p.y * scale2 + 50.0).cos() * 15.0;
    let hills3 = (p.x * scale3 + 200.0).cos() * (p.y * scale3 * 1.2 - 100.0).sin() * 8.0;
    
    // Gentle valleys
    let valley_pattern = ((p.x * scale4 + p.y * scale4 * 0.7).sin() + 
                         (p.x * scale4 * 0.5 - p.y * scale4 * 0.8).cos()) * 0.5;
    let valley_depth = valley_pattern.abs() * -20.0;
    
    // Rocky outcrops and tors
    let rock_pattern = ((p.x * 0.002).sin() * (p.y * 0.002).cos()).abs();
    let rocks = if rock_pattern > 0.85 { rock_pattern.powf(4.0) * 40.0 } else { 0.0 };
    
    // Ancient stone circles and monuments
    let monument_pattern = ((p.x * 0.001 + p.y * 0.001).sin() * 
                           (p.x * 0.001 - p.y * 0.001).cos()).abs();
    let monuments = if monument_pattern > 0.95 { 15.0 } else { 0.0 };
    
    // Meandering streams
    let stream = ((p.x * 0.005).sin() + (p.y * 0.004).cos()).abs();
    let stream_cut = if stream < 0.2 { stream * -15.0 } else { 0.0 };
    
    // Wind erosion patterns
    let wind_erosion = ((p.x * 0.01 + p.y * 0.002).sin() * 0.7 + 
                       (p.x * 0.008 - p.y * 0.01).cos() * 0.3).abs() * 5.0;
    
    50.0 + hills1 + hills2 + hills3 + valley_depth + rocks + monuments + stream_cut + wind_erosion
}

pub fn forest_height(p: Vec2) -> f32 {
    // Dense forest terrain with varied canopy heights
    let scale1 = 0.001;
    let scale2 = 0.002;
    let scale3 = 0.004;
    let scale4 = 0.0002;
    
    // Forest floor undulation
    let floor1 = (p.x * scale1).sin() * (p.y * scale1 * 1.1).cos() * 30.0;
    let floor2 = (p.x * scale2 + 300.0).cos() * (p.y * scale2 - 200.0).sin() * 20.0;
    
    // Ancient tree groves - create circular clearings
    let grove_pattern = ((p.x * scale4).sin().powf(2.0) + (p.y * scale4).cos().powf(2.0)).sqrt();
    let grove_height = (1.0 - grove_pattern).max(0.0) * 40.0;
    
    // Fallen logs and root systems
    let root_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 0.8).cos() + 
                       (p.x * scale3 * 1.2).cos() * (p.y * scale3).sin()).abs();
    let roots = root_pattern * 15.0;
    
    // Rocky streams through forest
    let stream_main = (p.x * 0.003 + p.y * 0.002).sin();
    let stream_branch = (p.x * 0.004 - p.y * 0.003).cos();
    let stream_depth = (stream_main.abs() + stream_branch.abs() * 0.5).min(0.5) * -25.0;
    
    // Clearings and meadows
    let clearing_pattern = ((p.x * 0.0008).sin() * (p.y * 0.0008).cos()).abs();
    let clearing = if clearing_pattern > 0.8 { -10.0 } else { 0.0 };
    
    // Hidden ravines
    let ravine = ((p.x * 0.002 + p.y * 0.001).sin() * 3.0).sin();
    let ravine_cut = if ravine.abs() > 0.9 { ravine.abs() * -40.0 } else { 0.0 };
    
    80.0 + floor1 + floor2 + grove_height + roots + stream_depth + clearing + ravine_cut
}

pub fn volcanic_height(p: Vec2) -> f32 {
    // Volcanic terrain with lava flows and calderas
    let scale1 = 0.0005;
    let scale2 = 0.001;
    let scale3 = 0.003;
    let scale4 = 0.0001;
    
    // Main volcanic cone
    let dist_from_center = ((p.x * scale4).powf(2.0) + (p.y * scale4).powf(2.0)).sqrt();
    let cone_height = (10.0 - dist_from_center).max(0.0) * 30.0;
    
    // Caldera formation
    let caldera = if dist_from_center < 2.0 { -80.0 } else { 0.0 };
    
    // Lava flow channels
    let flow_angle = p.y.atan2(p.x) + (dist_from_center * 0.5).sin() * 0.3;
    let flow_pattern = (flow_angle * 8.0).sin().abs();
    let lava_channels = flow_pattern * (8.0 - dist_from_center).max(0.0) * -20.0;
    
    // Secondary cinder cones
    let cone_pattern = ((p.x * scale2).sin() * (p.y * scale2).cos()).abs();
    let cinder_cones = if cone_pattern > 0.85 { 
        let local_dist = ((p.x * scale2 % 1.0).powf(2.0) + (p.y * scale2 % 1.0).powf(2.0)).sqrt();
        (1.0 - local_dist * 2.0).max(0.0) * 60.0 
    } else { 0.0 };
    
    // Basalt columns and formations
    let column_pattern = ((p.x * scale3).sin() + (p.y * scale3).cos()).abs();
    let columns = if column_pattern < 0.3 { 20.0 } else { 0.0 };
    
    // Ash fields and pumice deposits
    let ash_noise = ((p.x * 0.01).sin() * (p.y * 0.01).cos() + 
                    (p.x * 0.02).cos() * (p.y * 0.02).sin()).abs() * 10.0;
    
    // Geothermal features
    let thermal_pattern = ((p.x * 0.005).sin() * (p.y * 0.005).cos()).abs();
    let hot_springs = if thermal_pattern > 0.9 { thermal_pattern * -15.0 } else { 0.0 };
    
    150.0 + cone_height + caldera + lava_channels + cinder_cones + columns + ash_noise + hot_springs
}

pub fn desert_height(p: Vec2) -> f32 {
    // Sand dunes and rock formations
    let scale1 = 0.0008;
    let scale2 = 0.0015;
    let scale3 = 0.003;
    let scale4 = 0.0004;
    
    // Major dune ridges - star dunes
    let wind_dir1 = 0.7;
    let wind_dir2 = -0.5;
    let dune1 = ((p.x * scale1 + p.y * scale1 * wind_dir1).sin()).abs() * 80.0;
    let dune2 = ((p.x * scale2 * 0.8 + p.y * scale2 * wind_dir2).sin()).abs() * 60.0;
    
    // Barchan dunes - crescent shaped
    let barchan = ((p.x * scale2).sin() * (p.y * scale2 * 1.2).cos()).abs();
    let barchan_height = if barchan > 0.6 { barchan.powf(2.0) * 40.0 } else { 0.0 };
    
    // Rock outcrops and inselbergs
    let rock_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 0.9).cos()).abs();
    let inselberg = if rock_pattern > 0.7 { 
        rock_pattern.powf(0.5) * 120.0 
    } else { 0.0 };
    
    // Wadi systems - dry riverbeds
    let wadi_main = (p.x * scale3 + p.y * scale3 * 0.4).sin();
    let wadi_branch = (p.x * scale3 * 1.5 - p.y * scale3 * 0.7).cos();
    let wadi_depth = (wadi_main.abs() + wadi_branch.abs() * 0.3).min(0.5) * -30.0;
    
    // Salt flats and playas
    let playa_pattern = ((p.x * 0.0002 + p.y * 0.0002).sin() * 
                        (p.x * 0.0002 - p.y * 0.0002).cos()).abs();
    let salt_flat = if playa_pattern < 0.2 { -20.0 } else { 0.0 };
    
    // Ripple patterns
    let ripples = ((p.x * 0.02).sin() + (p.y * 0.02 + 1.0).cos()) * 2.0;
    
    20.0 + dune1 + dune2 + barchan_height + inselberg + wadi_depth + salt_flat + ripples
}

pub fn ocean_height(p: Vec2) -> f32 {
    // Deep ocean with underwater features
    let scale1 = 0.0003;
    let scale2 = 0.0008;
    let scale3 = 0.002;
    
    // Ocean floor base depth
    let base_depth = -200.0;
    
    // Underwater mountain ranges
    let ridge1 = ((p.x * scale1).sin() + (p.y * scale1 * 0.7).cos()).abs() * 80.0;
    let ridge2 = ((p.x * scale1 * 1.3 - 500.0).cos() * (p.y * scale1 + 300.0).sin()).abs() * 60.0;
    
    // Deep ocean trenches
    let trench_pattern = ((p.x * scale2 + p.y * scale2 * 0.3).sin() * 2.0).sin();
    let trench = if trench_pattern.abs() > 0.9 { trench_pattern.abs() * -150.0 } else { 0.0 };
    
    // Seamounts and underwater volcanoes
    let seamount_pattern = ((p.x * scale3).sin() * (p.y * scale3).cos()).abs();
    let seamount = if seamount_pattern > 0.85 { 
        let cone = 1.0 - seamount_pattern;
        cone * 180.0 
    } else { 0.0 };
    
    // Coral reef formations (in shallower areas)
    let reef_pattern = ((p.x * 0.005).sin() + (p.y * 0.005).cos()).abs();
    let coral_reef = if ridge1 + ridge2 > 100.0 && reef_pattern > 0.7 { 50.0 } else { 0.0 };
    
    // Abyssal plains
    let plain_smoothing = ((p.x * 0.0001).sin() + (p.y * 0.0001).cos()) * 10.0;
    
    base_depth + ridge1 + ridge2 + trench + seamount + coral_reef + plain_smoothing
}

pub fn alien_height(p: Vec2) -> f32 {
    // Bizarre alien landscape with impossible geometry
    let scale1 = 0.001;
    let scale2 = 0.002;
    let scale3 = 0.004;
    let scale4 = 0.0005;
    
    // Crystalline growths
    let crystal1 = ((p.x * scale1).sin() * 3.0).floor() / 3.0 * 50.0;
    let crystal2 = ((p.y * scale1 * 1.2).cos() * 3.0).floor() / 3.0 * 40.0;
    
    // Floating rock formations
    let float_pattern = ((p.x * scale2).sin() + (p.y * scale2).cos()).abs();
    let float_height = float_pattern.powf(0.3) * 100.0 + (float_pattern * 10.0).sin() * 30.0;
    
    // Spiral towers
    let angle = p.y.atan2(p.x);
    let radius = (p.x * p.x + p.y * p.y).sqrt() * scale4;
    let spiral = ((angle + radius * 2.0).sin() * (radius * 0.5).cos()).abs();
    let tower = if spiral > 0.8 { spiral * 150.0 } else { 0.0 };
    
    // Inverted valleys
    let valley_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 0.8).cos()).abs();
    let inverted = valley_pattern * 80.0;
    
    // Geometric patterns
    let hex_x = (p.x * 0.001 * 1.732).round();
    let hex_y = (p.y * 0.001).round();
    let hex_pattern = ((hex_x + hex_y).sin() * (hex_x - hex_y).cos()).abs();
    let geometric = hex_pattern * 40.0;
    
    // Pulsating ground
    let pulse = ((p.x * 0.0001 + p.y * 0.0001).sin() * PI).sin() * 20.0;
    
    100.0 + crystal1 + crystal2 + float_height + tower + inverted + geometric + pulse
}

pub fn crystal_height(p: Vec2) -> f32 {
    // Crystalline caverns and formations
    let scale1 = 0.0015;
    let scale2 = 0.003;
    let scale3 = 0.006;
    
    // Large crystal clusters
    let cluster_pattern = ((p.x * scale1).sin().powf(2.0) + (p.y * scale1).cos().powf(2.0)).sqrt();
    let main_clusters = (1.0 - cluster_pattern * 2.0).max(0.0).powf(2.0) * 120.0;
    
    // Crystal facets - sharp angular surfaces
    let facet1 = ((p.x * scale2).sin() * 2.0).floor() / 2.0;
    let facet2 = ((p.y * scale2 * 1.1).cos() * 2.0).floor() / 2.0;
    let facet_height = (facet1 + facet2).abs() * 30.0;
    
    // Geode formations
    let geode_pattern = ((p.x * scale3).sin() * (p.y * scale3).cos()).abs();
    let geode_hollow = if geode_pattern > 0.7 { 
        let center = 1.0 - geode_pattern;
        center * -80.0 
    } else { 0.0 };
    
    // Light refraction patterns
    let refraction = ((p.x * 0.01 + p.y * 0.005).sin() + 
                     (p.x * 0.005 - p.y * 0.01).cos()).abs() * 15.0;
    
    // Crystalline bridges
    let bridge_pattern = ((p.x * scale2 + p.y * scale2 * 0.3).sin()).abs();
    let bridges = if bridge_pattern < 0.2 { 40.0 } else { 0.0 };
    
    // Shattered areas
    let shatter = ((p.x * 0.02).sin() * (p.y * 0.02).cos() * 8.0).fract() * 20.0;
    
    50.0 + main_clusters + facet_height + geode_hollow + refraction + bridges + shatter
}

pub fn void_height(p: Vec2) -> f32 {
    // Abstract void dimension with reality distortions
    let scale1 = 0.001;
    let scale2 = 0.002;
    let scale3 = 0.0001;
    
    // Reality tears
    let tear1 = ((p.x * scale1).sin() * 5.0).sin() * 100.0;
    let tear2 = ((p.y * scale1 * 1.3).cos() * 4.0).cos() * 80.0;
    
    // Null zones - areas of absolute nothing
    let null_pattern = ((p.x * scale3 + p.y * scale3).sin() * 
                       (p.x * scale3 - p.y * scale3).cos()).abs();
    let null_depth = if null_pattern < 0.3 { -500.0 } else { 0.0 };
    
    // Dimensional fragments
    let frag_x = (p.x * scale2).sin() * (p.x * scale2 * 2.0).cos();
    let frag_y = (p.y * scale2 * 0.8).cos() * (p.y * scale2 * 1.5).sin();
    let fragments = (frag_x * frag_y).abs() * 150.0;
    
    // Gravity anomalies
    let gravity_well = ((p.x * 0.0005).sin().powf(2.0) + (p.y * 0.0005).cos().powf(2.0)).sqrt();
    let anomaly = (1.0 / (gravity_well + 0.1)) * 20.0;
    
    // Static interference
    let static_noise = ((p.x * 0.1).sin() * (p.y * 0.1).cos() * 1000.0).fract() * 30.0;
    
    0.0 + tear1 + tear2 + null_depth + fragments + anomaly + static_noise
}

pub fn cyber_height(p: Vec2) -> f32 {
    // Digital/cyberpunk terrain with data structures
    let scale1 = 0.001;
    let scale2 = 0.002;
    let scale3 = 0.0005;
    
    // Data towers - rectangular grid
    let grid_x = (p.x * scale1).floor();
    let grid_y = (p.y * scale1).floor();
    let tower_hash = ((grid_x * 73.0 + grid_y * 179.0).sin() * 1000.0).fract();
    let tower_height = if tower_hash > 0.7 { tower_hash * 200.0 } else { 0.0 };
    
    // Circuit paths
    let circuit_x = ((p.x * scale2).sin() * 3.0).floor() / 3.0;
    let circuit_y = ((p.y * scale2 * 1.2).cos() * 3.0).floor() / 3.0;
    let circuit_depth = (circuit_x.abs() + circuit_y.abs()) * -10.0;
    
    // Data streams
    let stream_pattern = ((p.x * scale3 + p.y * scale3 * 0.5).sin() * 
                         (p.x * scale3 * 0.7 - p.y * scale3).cos()).abs();
    let data_flow = stream_pattern * 30.0;
    
    // Firewall barriers
    let wall_pattern = ((p.x * 0.0003).sin() * 10.0).floor() / 10.0;
    let firewall = if wall_pattern.fract() < 0.1 { 80.0 } else { 0.0 };
    
    // Glitch artifacts
    let glitch = ((p.x * 0.01 + p.y * 0.01).sin() * 1000.0).fract();
    let artifact = if glitch > 0.95 { glitch * 100.0 } else { 0.0 };
    
    // Matrix rain effect
    let rain = ((p.y * 0.005 + p.x * 0.001).sin() * 5.0).fract() * 15.0;
    
    20.0 + tower_height + circuit_depth + data_flow + firewall + artifact + rain
}

pub fn floating_height(p: Vec2) -> f32 {
    // Floating islands and sky terrain
    let scale1 = 0.0005;
    let scale2 = 0.001;
    let scale3 = 0.002;
    
    // Major floating continents
    let continent_pattern = ((p.x * scale1).sin() * (p.y * scale1 * 0.8).cos()).abs();
    let continent = if continent_pattern > 0.6 { 
        let edge_falloff = ((continent_pattern - 0.6) / 0.4).powf(0.5);
        edge_falloff * 150.0 
    } else { -1000.0 }; // Void below
    
    // Smaller floating islands
    let island_pattern = ((p.x * scale2).sin() + (p.y * scale2).cos()).abs();
    let islands = if island_pattern > 0.8 && continent < 0.0 { 
        (island_pattern - 0.8) * 400.0 
    } else { 0.0 };
    
    // Sky bridges between islands
    let bridge_x = ((p.x * scale3).sin() * 4.0).abs();
    let bridge_y = ((p.y * scale3 * 1.2).cos() * 4.0).abs();
    let bridges = if bridge_x < 0.3 || bridge_y < 0.3 { 50.0 } else { 0.0 };
    
    // Waterfalls flowing into void
    let waterfall = ((p.x * 0.005).sin() + (p.y * 0.005).cos()).abs();
    let falls = if waterfall < 0.2 && continent > 50.0 { 
        waterfall * -100.0 
    } else { 0.0 };
    
    // Cloud layers
    let cloud_layer = ((p.x * 0.0001 + p.y * 0.0001).sin() * 0.5 + 0.5) * 30.0;
    
    // Gravitational distortions
    let grav_distort = ((p.x * 0.001).sin() * (p.y * 0.001).cos() * 3.0).sin() * 20.0;
    
    let base = continent.max(islands) + bridges + falls + cloud_layer + grav_distort;
    
    // Ensure minimum separation from void
    if base > -900.0 { base } else { -1000.0 }
}

// Smoothstep function for blending
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}