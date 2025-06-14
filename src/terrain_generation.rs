use glam::Vec2;
use std::f32::consts::PI;

// Public interface for terrain height calculation
pub fn height_at(x: f32, z: f32) -> f32 {
    // Snap coordinates to fixed precision to ensure consistent results
    // This prevents gaps between chunks due to floating point differences
    let snapped_x = (x * 1000.0).round() / 1000.0;
    let snapped_z = (z * 1000.0).round() / 1000.0;
    get_blended_biome_height(Vec2::new(snapped_x, snapped_z))
}

// Biome height generation functions - ported from GLSL

fn canyon_height(p: Vec2) -> f32 {
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

fn plateau_height(p: Vec2) -> f32 {
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
    let spires = spire_pattern.powf(6.0) * 40.0;
    
    // Weathering and erosion patterns
    let erosion = ((p.x * 0.01).sin() + (p.y * 0.01).cos()) * 10.0 * (1.0 - h1.max(h2).max(h3));
    
    // Combine all plateau features
    let height = h1.max(h2).max(h3) + spires + arch_cut + erosion;
    
    // Add dramatic vertical relief
    height.clamp(-100.0, 350.0)
}

fn crystalline_height(p: Vec2) -> f32 {
    // Varied spiky crystal formations
    let scale1 = 0.01;
    let scale2 = 0.02;
    let scale3 = 0.05;
    let scale4 = 0.007;
    
    // Vary spike sharpness based on position
    let sharpness1 = 0.8 + ((p.x * 0.001).sin() * (p.y * 0.001).cos() * 0.4);
    let sharpness2 = 1.2 + ((p.x * 0.002 + 100.0).sin() * (p.y * 0.002).cos() * 0.6);
    
    // Different crystal cluster patterns
    let spike1 = ((p.x * scale1).sin() * (p.y * scale1).cos()).abs().powf(sharpness1) * 150.0;
    let spike2 = ((p.x * scale2 + 50.0).cos() * (p.y * scale2 - 30.0).sin()).abs().powf(sharpness2) * 105.0;
    let spike3 = ((p.x * scale3 - 20.0).sin() * (p.y * scale3 + 40.0).cos()).abs() * 60.0;
    
    // Add larger crystal formations
    let large_crystal = ((p.x * scale4).sin().powf(2.0) + (p.y * scale4).cos().powf(2.0)).sqrt();
    let crystal_height = (1.0 - large_crystal).max(0.0).powf(1.5) * 120.0;
    
    // Base elevation variation
    let base_variation = (p.x * 0.003).sin() * (p.y * 0.003).cos() * 15.0;
    
    let height = spike1 + spike2 + spike3 + crystal_height + base_variation;
    height.clamp(-100.0, 400.0)
}

fn volcanic_height(p: Vec2) -> f32 {
    // Rough terrain with crater-like formations
    let scale1 = 0.004;
    let scale2 = 0.008;
    let scale3 = 0.002;
    let scale4 = 0.001;
    
    // Multiple volcanic craters with varying sizes
    let crater1 = ((p.x * scale1).sin().powf(2.0) + (p.y * scale1).cos().powf(2.0)).sqrt();
    let crater2 = ((p.x * scale3 + 100.0).sin().powf(2.0) + (p.y * scale3 - 50.0).cos().powf(2.0)).sqrt();
    let crater3 = ((p.x * scale4 * 1.5).sin().powf(2.0) + (p.y * scale4 * 1.2).cos().powf(2.0)).sqrt();
    
    // Dramatic volcanic cones
    let h1 = (1.0 - crater1) * 180.0;
    let h2 = (1.0 - crater2) * 120.0;
    let h3 = (1.0 - crater3) * 250.0; // Main massive volcano
    
    // Rough lava flows and volcanic debris
    let rough = (p.x * scale2).sin() * (p.y * scale2).cos() * 80.0;
    let lava_flow = ((p.x * 0.003 + p.y * 0.002).sin()).abs() * 40.0;
    
    // Caldera formations
    let caldera = if crater1 < 0.3 { -60.0 } else { 0.0 };
    let caldera2 = if crater3 < 0.4 { -80.0 } else { 0.0 };
    
    // Volcanic ridges and fissures
    let ridge = ((p.x * 0.005 - p.y * 0.003).sin().abs()).powf(2.0) * 60.0;
    
    let base_height = h1.max(h2).max(h3) + rough + lava_flow + ridge + caldera + caldera2;
    base_height.clamp(-150.0, 350.0)
}

fn mountain_height(p: Vec2) -> f32 {
    // Dramatic mountain ranges with connected peaks and ridgelines
    let scale1 = 0.0008;  // Major range direction
    let scale2 = 0.0015;  // Individual peaks
    let scale3 = 0.0003;  // Range backbone
    let scale4 = 0.004;   // Rocky details
    let scale5 = 0.0001;  // Continental scale
    
    // Major mountain range ridgeline - continuous spine
    let range_angle: f32 = 0.4; // Northwest to southeast trend
    let ridge_main = ((p.x * scale3 * range_angle.cos() + p.y * scale3 * range_angle.sin()).sin() * 0.5 + 0.5).powf(3.0);
    let ridge_secondary = ((p.x * scale3 * 1.2 - p.y * scale3 * 0.7).cos() * 0.5 + 0.5).powf(2.5);
    
    // Connected peak system along the ridges
    let peak_spacing = 0.0012;
    let peak_line1 = ((p.x * peak_spacing * range_angle.cos() + p.y * peak_spacing * range_angle.sin()).sin().powf(2.0) + 
                     (p.x * peak_spacing * range_angle.sin() - p.y * peak_spacing * range_angle.cos()).cos().powf(2.0)).sqrt();
    let peak_line2 = ((p.x * peak_spacing * 1.3 + 100.0).sin().powf(2.0) + 
                     (p.y * peak_spacing * 1.3 - 50.0).cos().powf(2.0)).sqrt();
    
    // Create dramatic pointed peaks
    let peak_sharpness = 2.5; // Higher = sharper peaks
    let h1 = (1.0 - peak_line1).max(0.0).powf(peak_sharpness) * 270.0;
    let h2 = (1.0 - peak_line2).max(0.0).powf(peak_sharpness * 0.8) * 225.0;
    
    // Ridge height variations - peaks are higher along the ridge
    let ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
    
    // Deep valleys between ridges
    let valley_pattern = (p.x * scale2 + p.y * scale2 * 0.6).sin() + 
                        (p.x * scale2 * 0.8 - p.y * scale2 * 0.5).cos();
    let valley_depth = valley_pattern.abs().powf(2.0) * -60.0;
    
    // Dramatic cliffs and rock faces
    let cliff_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 1.2).cos()).abs();
    let cliffs = cliff_pattern.powf(4.0) * 80.0;
    
    // Snow fields and glacial valleys
    let glacial_valley = ((p.x * scale2 * 0.5 + p.y * scale2 * 0.7).sin().abs()).powf(0.5) * -40.0;
    let snow_cap = (h1 + h2 + ridge_height).max(225.0) * 0.2;
    
    // Foothills that gradually rise to meet the mountains
    let distance_to_ridge = ((ridge_main - 0.5).abs() + (ridge_secondary - 0.5).abs()).min(1.0);
    let foothill_height = (1.0 - distance_to_ridge).powf(0.5) * 60.0;
    
    // Continental mountain building
    let tectonic = ((p.x * scale5).sin() + (p.y * scale5 * 0.8).cos()) * 45.0;
    
    let height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                glacial_valley + snow_cap + foothill_height + tectonic;
                
    // Ensure dramatic height variations
    height.clamp(-250.0, 450.0)
}

fn plains_height(p: Vec2) -> f32 {
    // Rolling plains with more variation
    let scale1 = 0.002;
    let scale2 = 0.007;
    let scale3 = 0.015;
    
    // Larger rolling hills
    let h1 = (p.x * scale1).sin() * (p.y * scale1).cos() * 40.0;
    let h2 = (p.x * scale2 + 100.0).sin() * (p.y * scale2 + 100.0).sin() * 20.0;
    let h3 = (p.x * scale3 + 200.0).cos() * (p.y * scale3 + 200.0).cos() * 10.0;
    
    // Add some occasional low ridges
    let ridge = ((p.x * 0.0005 + p.y * 0.0003).sin().abs()).powf(3.0) * 30.0;
    
    // Gentle valleys and depressions
    let depression = smoothstep(0.6, 0.3, ((p.x * 0.0008).sin() * (p.y * 0.0006).cos()).abs()) * -20.0;
    
    h1 + h2 + h3 + ridge + depression
}

fn desert_height(p: Vec2) -> f32 {
    // Sand dune formations with dramatic heights
    let scale1 = 0.005;
    let scale2 = 0.01;
    let scale3 = 0.03;
    
    // Large dramatic dunes
    let dunes = ((p.x * scale1).sin() * (p.y * scale1 * 1.2).cos()).abs() * 80.0;
    
    // Secondary dune fields
    let secondary = (p.x * scale2 + 30.0).cos() * (p.y * scale2 - 20.0).sin() * 30.0;
    
    // Sand ripples and waves
    let ripples = (p.x * scale3).sin() * (p.y * scale3).cos() * 10.0;
    
    // Occasional rock outcroppings
    let rocks = ((p.x * 0.002).sin() * (p.y * 0.002).cos()).abs().powf(4.0) * 60.0;
    
    // Wind-carved hollows
    let hollows = smoothstep(0.7, 0.5, ((p.x * 0.004 + p.y * 0.003).sin()).abs()) * -30.0;
    
    let height = -20.0 + dunes + secondary + ripples + rocks + hollows;
    height.clamp(-100.0, 200.0)
}

fn arctic_height(p: Vec2) -> f32 {
    // Dramatic glacial formations with towering ice
    let scale1 = 0.005;
    let scale2 = 0.015;
    let scale3 = 0.03;
    let scale4 = 0.002;
    let scale5 = 0.0008;
    
    // Massive glacial sheets with dramatic elevation
    let glacier_flow = ((p.x * scale5).sin() + (p.y * scale5 * 0.7).cos()) * 120.0;
    let glacier_thickness = ((p.x * scale5 * 0.5).sin().powf(2.0) + (p.y * scale5 * 0.5).cos().powf(2.0)) * 90.0;
    
    // Towering ice spires and seracs
    let serac_sharpness = 3.0 + ((p.x * 0.001).sin() * (p.y * 0.001).cos() * 2.0);
    let seracs = ((p.x * scale2).sin() * (p.y * scale2).cos()).abs().powf(serac_sharpness) * 225.0;
    
    // Massive icebergs and pressure ridges
    let pressure_ridge1 = ((p.x * scale1 + p.y * scale1 * 0.5).sin().abs()).powf(2.0) * 180.0;
    let pressure_ridge2 = ((p.x * scale1 * 0.8 - p.y * scale1 * 0.6).cos().abs()).powf(2.0) * 135.0;
    
    // Deep crevasses and moulins
    let crevasse_pattern = (p.x * scale3).sin() + (p.y * scale3 * 1.2).cos();
    let crevasse_depth = crevasse_pattern.abs().powf(4.0) * 120.0;
    let moulin = ((p.x * scale2 * 2.0 + p.y * scale2 * 1.5).sin() * 
                  (p.x * scale2 * 1.5 - p.y * scale2 * 2.0).cos()).abs().powf(6.0) * -60.0;
    
    // Ice caverns and tunnels
    let cave_pattern = ((p.x * scale4).sin() * (p.y * scale4 * 0.8).cos()).abs();
    let ice_caves = if cave_pattern > 0.6 { cave_pattern.powf(3.0) * -40.0 } else { 0.0 };
    
    // Frozen waterfalls and ice walls
    let ice_wall = ((p.x * scale1 * 0.3 + p.y * scale1 * 0.9).sin().abs()).powf(5.0) * 105.0;
    
    let height = -20.0 + glacier_flow + glacier_thickness + 
                seracs + pressure_ridge1 + pressure_ridge2 + ice_wall - 
                crevasse_depth + moulin + ice_caves;
                
    height.clamp(-150.0, 420.0)
}

fn badlands_height(p: Vec2) -> f32 {
    // Dramatic eroded landscape with towering formations
    let scale1 = 0.004;
    let scale2 = 0.01;
    let scale3 = 0.025;
    let scale4 = 0.0015;
    let scale5 = 0.0006;
    
    // Massive mesa formations with sheer cliffs
    let mesa_pattern = ((p.x * scale1).sin() * (p.y * scale1 * 0.8).cos()).abs();
    let mesa_height = if mesa_pattern > 0.3 {
        mesa_pattern.powf(0.2) * 300.0
    } else {
        mesa_pattern * 75.0
    };
    
    // Deep erosion channels and slot canyons
    let erosion_main = (p.x * scale2).sin() + (p.y * scale2 * 1.2).cos();
    let erosion_branch = (p.x * scale2 * 1.5 - p.y * scale2 * 0.7).sin() * 
                        (p.x * scale2 * 0.8 + p.y * scale2 * 1.3).cos();
    let slot_canyon = erosion_main.abs().powf(3.0) * 80.0 + 
                     erosion_branch.abs().powf(4.0) * 60.0;
    
    // Towering hoodoos and rock spires
    let hoodoo_field = ((p.x * scale3).sin() * (p.y * scale3).cos()).abs();
    let hoodoo_height = hoodoo_field.powf(5.0) * 270.0;
    let spire_cluster = ((p.x * scale3 * 1.5 + 100.0).sin() * 
                        (p.y * scale3 * 1.5 - 100.0).cos()).abs().powf(6.0) * 225.0;
    
    // Natural arches and bridges
    let arch_base = ((p.x * scale4 + p.y * scale4 * 0.6).sin() * 
                    (p.x * scale4 * 0.7 - p.y * scale4).cos()).abs();
    let arch_void = if arch_base > 0.7 && mesa_pattern > 0.5 {
        arch_base.powf(3.0) * -60.0
    } else {
        0.0
    };
    
    // Dramatic layered rock strata
    let strata_tilt = (p.x * 0.0001 + p.y * 0.00015).sin() * 0.3;
    let strata = (p.y * scale4 + p.x * strata_tilt).sin() * 0.5 + 0.5;
    let layer_height = (strata * 12.0).floor() * 10.0;
    
    // Scree slopes and talus fields
    let scree = (p.x * scale5).sin() * (p.y * scale5 * 1.1).cos() * 20.0 * (1.0 - mesa_pattern);
    
    let height = mesa_height + hoodoo_height + spire_cluster + 
                layer_height + arch_void - slot_canyon + scree;
                
    height.clamp(-250.0, 480.0)
}

fn floating_height(p: Vec2) -> f32 {
    // Large floating island formations at extreme heights
    let scale1 = 0.0008;
    let scale2 = 0.0015;
    let scale3 = 0.003;
    let scale4 = 0.0002;
    
    // Main floating continents
    let continent1 = ((p.x * scale4).sin() * (p.y * scale4).cos()).abs().powf(0.5) * 200.0;
    let continent2 = ((p.x * scale4 * 1.3 + 100.0).cos() * (p.y * scale4 * 0.9 - 50.0).sin()).abs().powf(0.6) * 150.0;
    
    // Individual floating islands
    let island1 = smoothstep(0.3, 0.8, ((p.x * scale1).sin() * (p.y * scale1).cos()).abs()) * 120.0;
    let island2 = smoothstep(0.4, 0.7, ((p.x * scale2 + 0.8).cos() * (p.y * scale2 * 0.9).sin()).abs()) * 90.0;
    
    // Rocky spires on the islands
    let spires = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs().powf(4.0) * 80.0;
    
    // Hanging gardens and waterfalls (negative values for overhangs)
    let overhang = smoothstep(0.7, 0.9, ((p.x * scale2 * 2.0 + p.y * scale2).sin()).abs()) * -40.0;
    
    // Crystal formations on underside
    let crystals = ((p.x * 0.01).sin() * (p.y * 0.01).cos()).abs().powf(3.0) * 60.0;
    
    // Base altitude for floating effect
    let base_altitude = 250.0;
    
    // Combine all features
    let height = base_altitude + continent1.max(continent2) + 
                island1.max(island2) + spires + crystals + overhang;
    
    height.clamp(150.0, 500.0)
}

fn caverns_height(p: Vec2) -> f32 {
    // Extensive underground cavern networks
    let scale1 = 0.004;
    let scale2 = 0.008;
    let scale3 = 0.02;
    let scale4 = 0.001;
    
    // Rolling karst terrain base
    let base = (p.x * scale1).sin() * (p.y * scale1 * 0.8).cos() * 60.0;
    
    // Major sinkholes and cave entrances
    let sinkhole1 = smoothstep(0.7, 0.2, ((p.x * scale2).sin() * (p.y * scale2).cos()).abs()) * -120.0;
    let sinkhole2 = smoothstep(0.6, 0.15, ((p.x * scale2 * 1.3 + 1.0).cos() * (p.y * scale2 * 0.9).sin()).abs()) * -100.0;
    let sinkhole3 = smoothstep(0.8, 0.3, ((p.x * scale4 + p.y * scale4 * 0.5).sin()).abs()) * -150.0;
    
    // Collapsed cavern ceilings
    let collapse_pattern = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs();
    let collapsed = if collapse_pattern > 0.6 { collapse_pattern.powf(2.0) * -80.0 } else { 0.0 };
    
    // Underground rivers and channels
    let river_channel = ((p.x * 0.003 + p.y * 0.002).sin().abs()).powf(3.0) * -40.0;
    
    // Stalactite and stalagmite fields (surface roughness)
    let formations = ((p.x * 0.05).sin() * (p.y * 0.05).cos()).abs() * 30.0;
    
    // Natural bridges over caverns
    let bridge = smoothstep(0.8, 0.95, ((p.x * scale2 * 0.7 - p.y * scale2 * 0.5).sin()).abs()) * 60.0;
    
    let height = base + sinkhole1 + sinkhole2 + sinkhole3 + 
                collapsed + river_channel + formations + bridge;
    
    height.clamp(-300.0, 150.0)
}

fn swamp_height(p: Vec2) -> f32 {
    // Murky swamp terrain with varied water features
    let scale1 = 0.005;
    let scale2 = 0.01;
    let scale3 = 0.03;
    let scale4 = 0.002;
    
    // Gentle base undulations
    let undulation = (p.x * scale1).sin() * (p.y * scale1 * 0.9).cos() * 25.0;
    
    // Deep water channels and pools
    let pools = smoothstep(0.4, 0.7, ((p.x * scale2).sin() * (p.y * scale2 * 1.1).cos()).abs()) * -40.0;
    let channels = ((p.x * scale4 + p.y * scale4 * 0.7).sin().abs()).powf(2.0) * -30.0;
    
    // Raised hummocks and dry land
    let hummocks = ((p.x * scale2 * 1.5).sin() * (p.y * scale2 * 1.3).cos()).abs().powf(3.0) * 35.0;
    
    // Dead trees and root systems (small bumps)
    let roots = ((p.x * scale3).sin() * (p.y * scale3 * 1.2).cos()).abs() * 15.0;
    
    // Bog pits and quicksand
    let bog_pattern = ((p.x * 0.008 - p.y * 0.006).sin() * (p.x * 0.007 + p.y * 0.009).cos()).abs();
    let bog_pits = if bog_pattern > 0.7 { bog_pattern.powf(2.0) * -25.0 } else { 0.0 };
    
    // Thick vegetation mounds
    let vegetation = smoothstep(0.3, 0.6, ((p.x * scale1 * 2.0).sin() * (p.y * scale1 * 1.8).cos()).abs()) * 20.0;
    
    let height = -10.0 + undulation + pools + channels + hummocks + 
                roots + bog_pits + vegetation;
    
    height.clamp(-100.0, 80.0)
}

// Helper function - Rust doesn't have smoothstep built-in
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Enhanced biome selection with more variety and smaller regions
fn get_biome_height(p: Vec2) -> f32 {
    // Multi-scale noise for more organic biome distribution
    let noise1 = (p.x * 0.0003).sin() * (p.y * 0.0003).cos();
    let noise2 = (p.x * 0.0007 + 1.3).sin() * (p.y * 0.0006 - 0.7).sin();
    let noise3 = (p.x * 0.0013 - 2.1).cos() * (p.y * 0.0011 + 1.9).sin();
    
    // Combine noises for complex patterns
    let biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Add local variation for sub-biomes
    let local_var = (p.x * 0.01).sin() * (p.y * 0.01).cos() * 0.1;
    let biome_noise = biome_noise + local_var;
    
    // 12 biome types distributed across the noise range
    if biome_noise < -0.7 {
        canyon_height(p)
    } else if biome_noise < -0.5 {
        caverns_height(p)
    } else if biome_noise < -0.3 {
        badlands_height(p)
    } else if biome_noise < -0.1 {
        plateau_height(p)
    } else if biome_noise < 0.1 {
        plains_height(p)
    } else if biome_noise < 0.25 {
        desert_height(p)
    } else if biome_noise < 0.4 {
        swamp_height(p)
    } else if biome_noise < 0.5 {
        crystalline_height(p)
    } else if biome_noise < 0.6 {
        volcanic_height(p)
    } else if biome_noise < 0.7 {
        arctic_height(p)
    } else if biome_noise < 0.8 {
        floating_height(p)
    } else {
        mountain_height(p)
    }
}

// Smooth blending between biomes
pub fn get_blended_biome_height(p: Vec2) -> f32 {
    // Sample multiple nearby points for smoother transitions
    let sample_dist = 100.0; // Increased for larger features
    let h_center = get_biome_height(p);
    let h_north = get_biome_height(p + Vec2::new(0.0, sample_dist));
    let h_south = get_biome_height(p + Vec2::new(0.0, -sample_dist));
    let h_east = get_biome_height(p + Vec2::new(sample_dist, 0.0));
    let h_west = get_biome_height(p + Vec2::new(-sample_dist, 0.0));
    
    // Weighted average for smoother transitions
    let primary_height = (h_center * 3.0 + h_north + h_south + h_east + h_west) / 7.0;
    
    // Enhanced fractal noise with more octaves for detail
    let mut fractal_noise = 0.0;
    let mut amplitude = 60.0; // Increased base amplitude
    let mut frequency = 0.0005;
    for _ in 0..7 { // More octaves for finer detail
        fractal_noise += (p.x * frequency).sin() * (p.y * frequency).cos() * amplitude;
        fractal_noise += (p.x * frequency * 1.7 + 100.0).sin() * (p.y * frequency * 1.7 + 100.0).cos() * amplitude * 0.7;
        amplitude *= 0.45; // Slower falloff for more influence from each octave
        frequency *= 2.3;
    }
    
    // Larger scale continental features
    let continent_scale = 0.0001; // Even larger scale
    let continental = ((p.x * continent_scale).sin() * (p.y * continent_scale * 0.8).cos() + 
                      (p.x * continent_scale * 0.3).cos() * (p.y * continent_scale * 1.2).sin()) * 120.0; // More dramatic
    
    // Erosion simulation - smooth out steep areas
    let slope_factor = ((p.x * 0.005).sin() - (p.x * 0.005 + 1.0).sin()).abs() + 
                      ((p.y * 0.005).sin() - (p.y * 0.005 + 1.0).sin()).abs();
    let erosion = slope_factor.min(1.0) * 0.3;
    
    // Terracing effect - make it much more subtle
    let terrace_height = 100.0; // Increased from 40 to make terraces less frequent
    let terraced = if primary_height > 0.0 {
        (primary_height / terrace_height).floor() * terrace_height + 
        (primary_height / terrace_height).fract().powf(2.0) * terrace_height
    } else {
        primary_height
    };
    
    // Mix terraced and smooth terrain - reduce terrace influence significantly
    let terrace_influence = ((p.x * 0.001 + p.y * 0.0008).sin() * 0.5 + 0.5).clamp(0.0, 1.0) * 0.2; // Max 20% terrace influence
    let height = terraced * terrace_influence + primary_height * (1.0 - terrace_influence);
    
    height + fractal_noise + continental * (1.0 - erosion)
}