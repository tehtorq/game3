use miniquad::*;
use glam::Mat4;

pub const VERTEX: &str = r#"#version 100
attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    gl_Position = mvp * vec4(pos, 1.0);
}"#;

pub const VERTEX_INSTANCED_TERRAIN: &str = r#"#version 100
precision mediump float;

attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;
uniform sampler2D height_texture;

// Biome height generation functions
float canyon_height(vec2 p) {
    // River-like canyon systems with dramatic depth variations
    float scale1 = 0.0015; // Main river course
    float scale2 = 0.003;  // Tributaries
    float scale3 = 0.006;  // Rapids and falls
    float scale4 = 0.0005; // Canyon width variation
    
    // Main river channel - continuous flowing pattern
    float river_flow = sin(p.x * scale1) * 0.7 + cos(p.y * scale1 * 0.8) * 0.5;
    float river_meander = (sin(p.x * scale1 * 0.5 + p.y * scale1 * 0.3) + 
                          cos(p.x * scale1 * 0.3 - p.y * scale1 * 0.4)) * 0.4;
    
    // Tributary channels joining the main river
    float tributary1 = (sin(p.x * scale2 - p.y * scale2 * 0.6) + 
                       cos(p.x * scale2 * 0.4 + p.y * scale2)) * 0.3;
    float tributary2 = (sin(p.x * scale2 * 1.2 + p.y * scale2 * 0.5) * 
                       cos(p.x * scale2 * 0.8 - p.y * scale2 * 0.7)) * 0.25;
    
    // River confluence points - deeper where rivers meet
    float confluence = (abs(tributary1 * river_flow) + abs(tributary2 * river_flow)) * 0.5;
    
    // Canyon width varies like a real river
    float width_pattern = sin(p.x * scale4 + p.y * scale4 * 0.7);
    float canyon_width = 0.3 + abs(width_pattern) * 0.7 + confluence * 0.3;
    
    // River depth with pools and rapids
    float pool_pattern = abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2));
    float rapids = pow(abs(sin(p.x * scale3 * 2.0 + p.y * scale3 * 1.5)), 3.0) * 0.3;
    
    // Combine all river features
    float river_depth = abs(river_flow + river_meander) * canyon_width + 
                       abs(tributary1) * 0.5 + abs(tributary2) * 0.5 + 
                       confluence + pool_pattern * 0.4 - rapids;
    
    // Create dramatic canyon walls with overhangs
    float wall_slope = 1.5 + width_pattern * 0.5;
    float canyon_cut = pow(abs(river_depth), wall_slope) * 2.0; // Deeper canyons
    
    // Terraced canyon walls
    float terraces = max(floor(canyon_cut * 6.0) / 6.0, 0.0);
    float final_depth = canyon_cut * 0.4 + terraces * 0.6;
    
    // High mesas between canyons
    float mesa_height = (pow(sin(p.x * scale4 * 0.5), 2.0) + pow(cos(p.y * scale4 * 0.5), 2.0)) * 40.0;
    float plateau_base = 200.0; // Base height for dramatic effect
    
    // Create dramatic height difference
    return plateau_base + mesa_height - final_depth * 160.0;
}

float plateau_height(vec2 p) {
    // Dramatic mesa and plateau formations with sheer cliffs
    float scale1 = 0.0008;
    float scale2 = 0.0005;
    float scale3 = 0.002;
    float scale4 = 0.0003;
    
    // Create distinct mesa formations
    float mesa1 = abs(sin(p.x * scale1) * cos(p.y * scale1 * 0.9));
    float mesa2 = abs(sin(p.x * scale2 + 200.0) * cos(p.y * scale2 - 150.0));
    float mesa3 = abs(cos(p.x * scale4 * 1.3) * sin(p.y * scale4 + 100.0));
    
    // Sharp cliff edges
    float cliff_sharpness = 8.0; // Very sharp transitions
    float mesa_top1 = mesa1 > 0.4 ? 1.0 : pow(mesa1 / 0.4, cliff_sharpness);
    float mesa_top2 = mesa2 > 0.5 ? 1.0 : pow(mesa2 / 0.5, cliff_sharpness);
    float mesa_top3 = mesa3 > 0.6 ? 1.0 : pow(mesa3 / 0.6, cliff_sharpness);
    
    // Dramatic height differences between plateau levels
    float base_elevation = -50.0;
    float tier1_height = 120.0;
    float tier2_height = 180.0;
    float tier3_height = 250.0;
    
    // Calculate mesa heights
    float h1 = base_elevation + mesa_top1 * tier1_height;
    float h2 = base_elevation + mesa_top2 * tier2_height;
    float h3 = base_elevation + mesa_top3 * tier3_height;
    
    // Natural bridges and arches
    float arch_pattern = abs(sin(p.x * scale3 + p.y * scale3 * 0.7) * 
                            cos(p.x * scale3 * 1.2 - p.y * scale3 * 0.5));
    float arch_cut = arch_pattern > 0.7 ? pow(arch_pattern, 4.0) * -50.0 : 0.0;
    
    // Rock spires and hoodoos
    float spire_pattern = abs(sin(p.x * scale3 * 2.0) * cos(p.y * scale3 * 2.0));
    float spires = pow(spire_pattern, 6.0) * 40.0;
    
    // Weathering and erosion patterns
    float erosion = (sin(p.x * 0.01) + cos(p.y * 0.01)) * 10.0 * (1.0 - max(max(mesa_top1, mesa_top2), mesa_top3));
    
    // Combine all plateau features
    float height = max(max(h1, h2), h3) + spires + arch_cut + erosion;
    
    // Add dramatic vertical relief
    return clamp(height, -100.0, 350.0);
}

float crystalline_height(vec2 p) {
    // Varied spiky crystal formations
    float scale1 = 0.01;
    float scale2 = 0.02;
    float scale3 = 0.05;
    float scale4 = 0.007;
    
    // Vary spike sharpness based on position
    float sharpness1 = 0.8 + (sin(p.x * 0.001) * cos(p.y * 0.001) * 0.4);
    float sharpness2 = 1.2 + (sin(p.x * 0.002 + 100.0) * cos(p.y * 0.002) * 0.6);
    
    // Different crystal cluster patterns
    float spike1 = pow(abs(sin(p.x * scale1) * cos(p.y * scale1)), sharpness1) * 150.0;
    float spike2 = pow(abs(cos(p.x * scale2 + 50.0) * sin(p.y * scale2 - 30.0)), sharpness2) * 105.0;
    float spike3 = abs(sin(p.x * scale3 - 20.0) * cos(p.y * scale3 + 40.0)) * 60.0;
    
    // Add larger crystal formations
    float large_crystal = sqrt(pow(sin(p.x * scale4), 2.0) + pow(cos(p.y * scale4), 2.0));
    float crystal_height = pow(max(1.0 - large_crystal, 0.0), 1.5) * 120.0;
    
    // Base elevation variation
    float base_variation = sin(p.x * 0.003) * cos(p.y * 0.003) * 15.0;
    
    float height = spike1 + spike2 + spike3 + crystal_height + base_variation;
    return clamp(height, -100.0, 400.0);
}

float volcanic_height(vec2 p) {
    // Rough terrain with crater-like formations
    float scale1 = 0.004;
    float scale2 = 0.008;
    float scale3 = 0.002;
    float scale4 = 0.001;
    
    // Multiple volcanic craters with varying sizes
    float crater1 = sqrt(pow(sin(p.x * scale1), 2.0) + pow(cos(p.y * scale1), 2.0));
    float crater2 = sqrt(pow(sin(p.x * scale3 + 100.0), 2.0) + pow(cos(p.y * scale3 - 50.0), 2.0));
    float crater3 = sqrt(pow(sin(p.x * scale4 * 1.5), 2.0) + pow(cos(p.y * scale4 * 1.2), 2.0));
    
    // Dramatic volcanic cones
    float h1 = (1.0 - crater1) * 180.0;
    float h2 = (1.0 - crater2) * 120.0;
    float h3 = (1.0 - crater3) * 250.0; // Main massive volcano
    
    // Rough lava flows and volcanic debris
    float rough = sin(p.x * scale2) * cos(p.y * scale2) * 80.0;
    float lava_flow = abs(sin(p.x * 0.003 + p.y * 0.002)) * 40.0;
    
    // Caldera formations
    float caldera = crater1 < 0.3 ? -60.0 : 0.0;
    float caldera2 = crater3 < 0.4 ? -80.0 : 0.0;
    
    // Volcanic ridges and fissures
    float ridge = pow(abs(sin(p.x * 0.005 - p.y * 0.003)), 2.0) * 60.0;
    
    float base_height = max(max(h1, h2), h3) + rough + lava_flow + ridge + caldera + caldera2;
    return clamp(base_height, -150.0, 350.0);
}

float mountain_height(vec2 p) {
    // Dramatic mountain ranges with connected peaks and ridgelines
    float scale1 = 0.0008;  // Major range direction
    float scale2 = 0.0015;  // Individual peaks
    float scale3 = 0.0003;  // Range backbone
    float scale4 = 0.004;   // Rocky details
    float scale5 = 0.0001;  // Continental scale
    
    // Major mountain range ridgeline - continuous spine
    float range_angle = 0.4; // Northwest to southeast trend
    float ridge_main = pow(sin(p.x * scale3 * cos(range_angle) + p.y * scale3 * sin(range_angle)) * 0.5 + 0.5, 3.0);
    float ridge_secondary = pow(cos(p.x * scale3 * 1.2 - p.y * scale3 * 0.7) * 0.5 + 0.5, 2.5);
    
    // Connected peak system along the ridges
    float peak_spacing = 0.0012;
    float peak_line1 = sqrt(pow(sin(p.x * peak_spacing * cos(range_angle) + p.y * peak_spacing * sin(range_angle)), 2.0) + 
                           pow(cos(p.x * peak_spacing * sin(range_angle) - p.y * peak_spacing * cos(range_angle)), 2.0));
    float peak_line2 = sqrt(pow(sin(p.x * peak_spacing * 1.3 + 100.0), 2.0) + 
                           pow(cos(p.y * peak_spacing * 1.3 - 50.0), 2.0));
    
    // Create dramatic pointed peaks
    float peak_sharpness = 2.5; // Higher = sharper peaks
    float h1 = pow(max(1.0 - peak_line1, 0.0), peak_sharpness) * 270.0;
    float h2 = pow(max(1.0 - peak_line2, 0.0), peak_sharpness * 0.8) * 225.0;
    
    // Ridge height variations - peaks are higher along the ridge
    float ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
    
    // Deep valleys between ridges
    float valley_pattern = sin(p.x * scale2 + p.y * scale2 * 0.6) + 
                          cos(p.x * scale2 * 0.8 - p.y * scale2 * 0.5);
    float valley_depth = pow(abs(valley_pattern), 2.0) * -60.0;
    
    // Dramatic cliffs and rock faces
    float cliff_pattern = abs(sin(p.x * scale4) * cos(p.y * scale4 * 1.2));
    float cliffs = pow(cliff_pattern, 4.0) * 80.0;
    
    // Snow fields and glacial valleys
    float glacial_valley = pow(abs(sin(p.x * scale2 * 0.5 + p.y * scale2 * 0.7)), 0.5) * -40.0;
    float snow_cap = max(h1 + h2 + ridge_height, 225.0) * 0.2;
    
    // Foothills that gradually rise to meet the mountains
    float distance_to_ridge = min(abs(ridge_main - 0.5) + abs(ridge_secondary - 0.5), 1.0);
    float foothill_height = pow(1.0 - distance_to_ridge, 0.5) * 60.0;
    
    // Continental mountain building
    float tectonic = (sin(p.x * scale5) + cos(p.y * scale5 * 0.8)) * 45.0;
    
    float height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                  glacial_valley + snow_cap + foothill_height + tectonic;
                  
    // Ensure dramatic height variations
    return clamp(height, -250.0, 450.0);
}

float plains_height(vec2 p) {
    // Rolling plains with more variation
    float scale1 = 0.002;
    float scale2 = 0.007;
    float scale3 = 0.015;
    
    // Larger rolling hills
    float h1 = sin(p.x * scale1) * cos(p.y * scale1) * 40.0;
    float h2 = sin(p.x * scale2 + 100.0) * sin(p.y * scale2 + 100.0) * 20.0;
    float h3 = cos(p.x * scale3 + 200.0) * cos(p.y * scale3 + 200.0) * 10.0;
    
    // Add some occasional low ridges
    float ridge = pow(abs(sin(p.x * 0.0005 + p.y * 0.0003)), 3.0) * 30.0;
    
    // Gentle valleys and depressions
    float depression = smoothstep(0.6, 0.3, abs(sin(p.x * 0.0008) * cos(p.y * 0.0006))) * -20.0;
    
    return h1 + h2 + h3 + ridge + depression;
}

float desert_height(vec2 p) {
    // Sand dune formations with dramatic heights
    float scale1 = 0.005;
    float scale2 = 0.01;
    float scale3 = 0.03;
    
    // Large dramatic dunes
    float dunes = abs(sin(p.x * scale1) * cos(p.y * scale1 * 1.2)) * 80.0;
    
    // Secondary dune fields
    float secondary = cos(p.x * scale2 + 30.0) * sin(p.y * scale2 - 20.0) * 30.0;
    
    // Sand ripples and waves
    float ripples = sin(p.x * scale3) * cos(p.y * scale3) * 10.0;
    
    // Occasional rock outcroppings
    float rocks = pow(abs(sin(p.x * 0.002) * cos(p.y * 0.002)), 4.0) * 60.0;
    
    // Wind-carved hollows
    float hollows = smoothstep(0.7, 0.5, abs(sin(p.x * 0.004 + p.y * 0.003))) * -30.0;
    
    float height = -20.0 + dunes + secondary + ripples + rocks + hollows;
    return clamp(height, -100.0, 200.0);
}

float arctic_height(vec2 p) {
    // Dramatic glacial formations with towering ice
    float scale1 = 0.005;
    float scale2 = 0.015;
    float scale3 = 0.03;
    float scale4 = 0.002;
    float scale5 = 0.0008;
    
    // Massive glacial sheets with dramatic elevation
    float glacier_flow = (sin(p.x * scale5) + cos(p.y * scale5 * 0.7)) * 120.0;
    float glacier_thickness = (pow(sin(p.x * scale5 * 0.5), 2.0) + pow(cos(p.y * scale5 * 0.5), 2.0)) * 90.0;
    
    // Towering ice spires and seracs
    float serac_sharpness = 3.0 + (sin(p.x * 0.001) * cos(p.y * 0.001) * 2.0);
    float seracs = pow(abs(sin(p.x * scale2) * cos(p.y * scale2)), serac_sharpness) * 225.0;
    
    // Massive icebergs and pressure ridges
    float pressure_ridge1 = pow(abs(sin(p.x * scale1 + p.y * scale1 * 0.5)), 2.0) * 180.0;
    float pressure_ridge2 = pow(abs(cos(p.x * scale1 * 0.8 - p.y * scale1 * 0.6)), 2.0) * 135.0;
    
    // Deep crevasses and moulins
    float crevasse_pattern = sin(p.x * scale3) + cos(p.y * scale3 * 1.2);
    float crevasse_depth = pow(abs(crevasse_pattern), 4.0) * 120.0;
    float moulin = pow(abs(sin(p.x * scale2 * 2.0 + p.y * scale2 * 1.5) * 
                          cos(p.x * scale2 * 1.5 - p.y * scale2 * 2.0)), 6.0) * -60.0;
    
    // Ice caverns and tunnels
    float cave_pattern = abs(sin(p.x * scale4) * cos(p.y * scale4 * 0.8));
    float ice_caves = cave_pattern > 0.6 ? pow(cave_pattern, 3.0) * -40.0 : 0.0;
    
    // Frozen waterfalls and ice walls
    float ice_wall = pow(abs(sin(p.x * scale1 * 0.3 + p.y * scale1 * 0.9)), 5.0) * 105.0;
    
    float height = -20.0 + glacier_flow + glacier_thickness + 
                  seracs + pressure_ridge1 + pressure_ridge2 + ice_wall - 
                  crevasse_depth + moulin + ice_caves;
                  
    return clamp(height, -150.0, 420.0);
}

float badlands_height(vec2 p) {
    // Dramatic eroded landscape with towering formations
    float scale1 = 0.004;
    float scale2 = 0.01;
    float scale3 = 0.025;
    float scale4 = 0.0015;
    float scale5 = 0.0006;
    
    // Massive mesa formations with sheer cliffs
    float mesa_pattern = abs(sin(p.x * scale1) * cos(p.y * scale1 * 0.8));
    float mesa_height = mesa_pattern > 0.3 ? 
        pow(mesa_pattern, 0.2) * 300.0 : 
        mesa_pattern * 75.0;
    
    // Deep erosion channels and slot canyons
    float erosion_main = sin(p.x * scale2) + cos(p.y * scale2 * 1.2);
    float erosion_branch = sin(p.x * scale2 * 1.5 - p.y * scale2 * 0.7) * 
                          cos(p.x * scale2 * 0.8 + p.y * scale2 * 1.3);
    float slot_canyon = pow(abs(erosion_main), 3.0) * 80.0 + 
                       pow(abs(erosion_branch), 4.0) * 60.0;
    
    // Towering hoodoos and rock spires
    float hoodoo_field = abs(sin(p.x * scale3) * cos(p.y * scale3));
    float hoodoo_height = pow(hoodoo_field, 5.0) * 270.0;
    float spire_cluster = pow(abs(sin(p.x * scale3 * 1.5 + 100.0) * 
                               cos(p.y * scale3 * 1.5 - 100.0)), 6.0) * 225.0;
    
    // Natural arches and bridges
    float arch_base = abs(sin(p.x * scale4 + p.y * scale4 * 0.6) * 
                         cos(p.x * scale4 * 0.7 - p.y * scale4));
    float arch_void = (arch_base > 0.7 && mesa_pattern > 0.5) ? 
        pow(arch_base, 3.0) * -60.0 : 
        0.0;
    
    // Dramatic layered rock strata
    float strata_tilt = sin(p.x * 0.0001 + p.y * 0.00015) * 0.3;
    float strata = sin(p.y * scale4 + p.x * strata_tilt) * 0.5 + 0.5;
    float layer_height = floor(strata * 12.0) * 10.0;
    
    // Scree slopes and talus fields
    float scree = sin(p.x * scale5) * cos(p.y * scale5 * 1.1) * 20.0 * (1.0 - mesa_pattern);
    
    float height = mesa_height + hoodoo_height + spire_cluster + 
                  layer_height + arch_void - slot_canyon + scree;
                  
    return clamp(height, -250.0, 480.0);
}

float floating_height(vec2 p) {
    // Large floating island formations at extreme heights
    float scale1 = 0.0008;
    float scale2 = 0.0015;
    float scale3 = 0.003;
    float scale4 = 0.0002;
    
    // Main floating continents
    float continent1 = pow(abs(sin(p.x * scale4) * cos(p.y * scale4)), 0.5) * 200.0;
    float continent2 = pow(abs(cos(p.x * scale4 * 1.3 + 100.0) * sin(p.y * scale4 * 0.9 - 50.0)), 0.6) * 150.0;
    
    // Individual floating islands
    float island1 = smoothstep(0.3, 0.8, abs(sin(p.x * scale1) * cos(p.y * scale1))) * 120.0;
    float island2 = smoothstep(0.4, 0.7, abs(cos(p.x * scale2 + 0.8) * sin(p.y * scale2 * 0.9))) * 90.0;
    
    // Rocky spires on the islands
    float spires = pow(abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2)), 4.0) * 80.0;
    
    // Hanging gardens and waterfalls (negative values for overhangs)
    float overhang = smoothstep(0.7, 0.9, abs(sin(p.x * scale2 * 2.0 + p.y * scale2))) * -40.0;
    
    // Crystal formations on underside
    float crystals = pow(abs(sin(p.x * 0.01) * cos(p.y * 0.01)), 3.0) * 60.0;
    
    // Base altitude for floating effect
    float base_altitude = 250.0;
    
    // Combine all features
    float height = base_altitude + max(continent1, continent2) + 
                  max(island1, island2) + spires + crystals + overhang;
    
    return clamp(height, 150.0, 500.0);
}

float caverns_height(vec2 p) {
    // Extensive underground cavern networks
    float scale1 = 0.004;
    float scale2 = 0.008;
    float scale3 = 0.02;
    float scale4 = 0.001;
    
    // Rolling karst terrain base
    float base = sin(p.x * scale1) * cos(p.y * scale1 * 0.8) * 60.0;
    
    // Major sinkholes and cave entrances
    float sinkhole1 = smoothstep(0.7, 0.2, abs(sin(p.x * scale2) * cos(p.y * scale2))) * -120.0;
    float sinkhole2 = smoothstep(0.6, 0.15, abs(cos(p.x * scale2 * 1.3 + 1.0) * sin(p.y * scale2 * 0.9))) * -100.0;
    float sinkhole3 = smoothstep(0.8, 0.3, abs(sin(p.x * scale4 + p.y * scale4 * 0.5))) * -150.0;
    
    // Collapsed cavern ceilings
    float collapse_pattern = abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2));
    float collapsed = collapse_pattern > 0.6 ? pow(collapse_pattern, 2.0) * -80.0 : 0.0;
    
    // Underground rivers and channels
    float river_channel = pow(abs(sin(p.x * 0.003 + p.y * 0.002)), 3.0) * -40.0;
    
    // Stalactite and stalagmite fields (surface roughness)
    float formations = abs(sin(p.x * 0.05) * cos(p.y * 0.05)) * 30.0;
    
    // Natural bridges over caverns
    float bridge = smoothstep(0.8, 0.95, abs(sin(p.x * scale2 * 0.7 - p.y * scale2 * 0.5))) * 60.0;
    
    float height = base + sinkhole1 + sinkhole2 + sinkhole3 + 
                  collapsed + river_channel + formations + bridge;
    
    return clamp(height, -300.0, 150.0);
}

float swamp_height(vec2 p) {
    // Murky swamp terrain with varied water features
    float scale1 = 0.005;
    float scale2 = 0.01;
    float scale3 = 0.03;
    float scale4 = 0.002;
    
    // Gentle base undulations
    float undulation = sin(p.x * scale1) * cos(p.y * scale1 * 0.9) * 25.0;
    
    // Deep water channels and pools
    float pools = smoothstep(0.4, 0.7, abs(sin(p.x * scale2) * cos(p.y * scale2 * 1.1))) * -40.0;
    float channels = pow(abs(sin(p.x * scale4 + p.y * scale4 * 0.7)), 2.0) * -30.0;
    
    // Raised hummocks and dry land
    float hummocks = pow(abs(sin(p.x * scale2 * 1.5) * cos(p.y * scale2 * 1.3)), 3.0) * 35.0;
    
    // Dead trees and root systems (small bumps)
    float roots = abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2)) * 15.0;
    
    // Bog pits and quicksand
    float bog_pattern = abs(sin(p.x * 0.008 - p.y * 0.006) * cos(p.x * 0.007 + p.y * 0.009));
    float bog_pits = bog_pattern > 0.7 ? pow(bog_pattern, 2.0) * -25.0 : 0.0;
    
    // Thick vegetation mounds
    float vegetation = smoothstep(0.3, 0.6, abs(sin(p.x * scale1 * 2.0) * cos(p.y * scale1 * 1.8))) * 20.0;
    
    float height = -10.0 + undulation + pools + channels + hummocks + 
                  roots + bog_pits + vegetation;
    
    return clamp(height, -100.0, 80.0);
}

// Enhanced biome selection with more variety and smaller regions
float get_biome_height(vec2 p) {
    // Multi-scale noise for more organic biome distribution
    float noise1 = sin(p.x * 0.0003) * cos(p.y * 0.0003);
    float noise2 = sin(p.x * 0.0007 + 1.3) * sin(p.y * 0.0006 - 0.7);
    float noise3 = cos(p.x * 0.0013 - 2.1) * sin(p.y * 0.0011 + 1.9);
    
    // Combine noises for complex patterns
    float biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Add local variation for sub-biomes
    float local_var = sin(p.x * 0.01) * cos(p.y * 0.01) * 0.1;
    biome_noise += local_var;
    
    // 12 biome types distributed across the noise range
    if (biome_noise < -0.7) {
        return canyon_height(p);
    } else if (biome_noise < -0.5) {
        return caverns_height(p);
    } else if (biome_noise < -0.3) {
        return badlands_height(p);
    } else if (biome_noise < -0.1) {
        return plateau_height(p);
    } else if (biome_noise < 0.1) {
        return plains_height(p);
    } else if (biome_noise < 0.25) {
        return desert_height(p);
    } else if (biome_noise < 0.4) {
        return swamp_height(p);
    } else if (biome_noise < 0.5) {
        return crystalline_height(p);
    } else if (biome_noise < 0.6) {
        return volcanic_height(p);
    } else if (biome_noise < 0.7) {
        return arctic_height(p);
    } else if (biome_noise < 0.8) {
        return floating_height(p);
    } else {
        return mountain_height(p);
    }
}

// Smooth blending between biomes
float get_blended_biome_height(vec2 p) {
    // Sample multiple nearby points for smoother transitions
    float sample_dist = 100.0; // Increased for larger features
    float h_center = get_biome_height(p);
    float h_north = get_biome_height(p + vec2(0.0, sample_dist));
    float h_south = get_biome_height(p + vec2(0.0, -sample_dist));
    float h_east = get_biome_height(p + vec2(sample_dist, 0.0));
    float h_west = get_biome_height(p + vec2(-sample_dist, 0.0));
    
    // Weighted average for smoother transitions
    float primary_height = (h_center * 3.0 + h_north + h_south + h_east + h_west) / 7.0;
    
    // Enhanced fractal noise with more octaves for detail
    float fractal_noise = 0.0;
    float amplitude = 60.0; // Increased base amplitude
    float frequency = 0.0005;
    for (int i = 0; i < 7; i++) { // More octaves for finer detail
        fractal_noise += sin(p.x * frequency) * cos(p.y * frequency) * amplitude;
        fractal_noise += sin(p.x * frequency * 1.7 + 100.0) * cos(p.y * frequency * 1.7 + 100.0) * amplitude * 0.7;
        amplitude *= 0.45; // Slower falloff for more influence from each octave
        frequency *= 2.3;
    }
    
    // Larger scale continental features
    float continent_scale = 0.0001; // Even larger scale
    float continental = (sin(p.x * continent_scale) * cos(p.y * continent_scale * 0.8) + 
                        cos(p.x * continent_scale * 0.3) * sin(p.y * continent_scale * 1.2)) * 120.0; // More dramatic
    
    // Erosion simulation - smooth out steep areas
    float slope_factor = abs(sin(p.x * 0.005) - sin(p.x * 0.005 + 1.0)) + 
                        abs(sin(p.y * 0.005) - sin(p.y * 0.005 + 1.0));
    float erosion = min(slope_factor, 1.0) * 0.3;
    
    // Terracing effect - make it much more subtle
    float terrace_height = 100.0; // Increased from 40 to make terraces less frequent
    float terraced = primary_height > 0.0 ?
        floor(primary_height / terrace_height) * terrace_height + 
        pow(fract(primary_height / terrace_height), 2.0) * terrace_height :
        primary_height;
    
    // Mix terraced and smooth terrain - reduce terrace influence significantly
    float terrace_influence = clamp(sin(p.x * 0.001 + p.y * 0.0008) * 0.5 + 0.5, 0.0, 1.0) * 0.2; // Max 20% terrace influence
    float height = terraced * terrace_influence + primary_height * (1.0 - terrace_influence);
    
    return height + fractal_noise + continental * (1.0 - erosion);
}

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y, pos.z + instance_offset.y);
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Sample height from texture instead of procedural generation
    vec2 uv = (world_pos.xz / terrain_scale) + 0.5;
    
    // Debug: Check if UV coordinates are varying
    // If terrain_scale is 3360 and world_pos.x ranges from about -1680 to +1680
    // then uv should range from 0 to 1
    
    vec4 height_sample = texture2D(height_texture, uv);
    
    // Decode 24-bit height value from RGB channels
    float height_normalized = (height_sample.r * 255.0 * 65536.0 + 
                              height_sample.g * 255.0 * 256.0 + 
                              height_sample.b * 255.0) / 16777215.0;
    
    // Convert from normalized [0,1] to actual height range [-500, 500]
    float height = (height_normalized * 1000.0) - 500.0;
    
    // Since vertex texture fetch isn't working properly, use procedural generation
    // This will use the same improved terrain generation functions
    world_pos.y = get_blended_biome_height(world_pos.xz);
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    // Debug: visualize UV coordinates as height to check if they're correct
    // world_pos.y = uv.x * 100.0 - 50.0;  // Uncomment to debug UV.x
    // world_pos.y = uv.y * 100.0 - 50.0;  // Uncomment to debug UV.y
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

pub const FRAGMENT_TERRAIN: &str = r#"#version 100
precision mediump float;

uniform vec3 color;
uniform sampler2D height_texture;
uniform sampler2D biome_texture;
uniform float terrain_scale;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

void main() {
    // Sample biome color from texture
    vec2 uv = (v_world_xz / terrain_scale) + 0.5;
    vec3 biome_color = texture2D(biome_texture, uv).rgb;
    
    // DEBUG: Sample height in fragment shader to verify texture is working
    vec4 height_sample = texture2D(height_texture, uv);
    float debug_height = height_sample.r;
    
    // Calculate height-based brightness (higher = brighter)
    // Terrain can range from about -500 to +500, normalize to this range
    float height_factor = (v_height + 500.0) / 1000.0; // Normalize to 0-1 range
    height_factor = clamp(height_factor, 0.0, 1.0);
    height_factor = 0.3 + height_factor * 0.7; // Map to 0.3-1.0 range for more contrast
    
    // Blend base color with biome color
    vec3 blended_color = mix(color, biome_color, 0.7); // 70% biome color, 30% base color
    vec3 adjusted_color = blended_color * height_factor;
    
    // DEBUG: Visualize the height texture in red channel
    // adjusted_color.r = debug_height;
    
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(adjusted_color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(adjusted_color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(adjusted_color, 1.0); // Bright version for edges
        }
    }
}"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(color, 1.0); // Bright version for edges
        }
    }
}"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
            ],
        },
    }
}

pub fn meta_terrain() -> ShaderMeta {
    ShaderMeta {
        images: vec!["height_texture".to_string(), "biome_texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
                UniformDesc::new("terrain_scale", UniformType::Float1),
                UniformDesc::new("terrain_y_base", UniformType::Float1),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
}

impl Uniforms {
    pub fn new(mvp: Mat4, color: [f32; 3]) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
        }
    }
}

#[repr(C)]
pub struct UniformsTerrain {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
    pub terrain_scale: f32,
    pub terrain_y_base: f32,
    pub _padding2: [f32; 2],
}

impl UniformsTerrain {
    pub fn new(mvp: Mat4, color: [f32; 3], terrain_scale: f32, terrain_y_base: f32) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
            terrain_scale,
            terrain_y_base,
            _padding2: [0.0, 0.0],
        }
    }
}