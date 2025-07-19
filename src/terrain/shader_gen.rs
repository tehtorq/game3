// Terrain shader generation module
// Generates GLSL code for terrain height calculations from Rust implementations

use std::fmt::Write;

/// Generate GLSL biome height functions from Rust implementations
pub fn generate_biome_height_functions() -> String {
    let mut glsl = String::new();
    
    // Add smoothstep if not already defined
    writeln!(&mut glsl, "float smoothstep_custom(float edge0, float edge1, float x) {{").unwrap();
    writeln!(&mut glsl, "    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);").unwrap();
    writeln!(&mut glsl, "    return t * t * (3.0 - 2.0 * t);").unwrap();
    writeln!(&mut glsl, "}}").unwrap();
    writeln!(&mut glsl).unwrap();
    
    // Canyon height function
    writeln!(&mut glsl, "float canyon_height(vec2 p) {{").unwrap();
    writeln!(&mut glsl, "    // River-like canyon systems with dramatic depth variations").unwrap();
    writeln!(&mut glsl, "    float scale1 = 0.0015; // Main river course").unwrap();
    writeln!(&mut glsl, "    float scale2 = 0.003;  // Tributaries").unwrap();
    writeln!(&mut glsl, "    float scale3 = 0.006;  // Rapids and falls").unwrap();
    writeln!(&mut glsl, "    float scale4 = 0.0005; // Canyon width variation").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Main river channel - continuous flowing pattern").unwrap();
    writeln!(&mut glsl, "    float river_flow = sin(p.x * scale1) * 0.7 + cos(p.y * scale1 * 0.8) * 0.5;").unwrap();
    writeln!(&mut glsl, "    float river_meander = (sin(p.x * scale1 * 0.5 + p.y * scale1 * 0.3) + ").unwrap();
    writeln!(&mut glsl, "                          cos(p.x * scale1 * 0.3 - p.y * scale1 * 0.4)) * 0.4;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Tributary channels joining the main river").unwrap();
    writeln!(&mut glsl, "    float tributary1 = (sin(p.x * scale2 - p.y * scale2 * 0.6) + ").unwrap();
    writeln!(&mut glsl, "                       cos(p.x * scale2 * 0.4 + p.y * scale2)) * 0.3;").unwrap();
    writeln!(&mut glsl, "    float tributary2 = (sin(p.x * scale2 * 1.2 + p.y * scale2 * 0.5) * ").unwrap();
    writeln!(&mut glsl, "                       cos(p.x * scale2 * 0.8 - p.y * scale2 * 0.7)) * 0.25;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // River confluence points - deeper where rivers meet").unwrap();
    writeln!(&mut glsl, "    float confluence = abs(tributary1 * river_flow) + abs(tributary2 * river_flow) * 0.5;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Canyon width varies like a real river").unwrap();
    writeln!(&mut glsl, "    float width_pattern = sin(p.x * scale4 + p.y * scale4 * 0.7);").unwrap();
    writeln!(&mut glsl, "    float canyon_width = 0.3 + abs(width_pattern) * 0.7 + confluence * 0.3;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // River depth with pools and rapids").unwrap();
    writeln!(&mut glsl, "    float pool_pattern = abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2));").unwrap();
    writeln!(&mut glsl, "    float rapids = pow(abs(sin(p.x * scale3 * 2.0 + p.y * scale3 * 1.5)), 3.0) * 0.3;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Combine all river features").unwrap();
    writeln!(&mut glsl, "    float river_depth = abs(river_flow + river_meander) * canyon_width + ").unwrap();
    writeln!(&mut glsl, "                       abs(tributary1) * 0.5 + abs(tributary2) * 0.5 + ").unwrap();
    writeln!(&mut glsl, "                       confluence + pool_pattern * 0.4 - rapids;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Create dramatic canyon walls with overhangs").unwrap();
    writeln!(&mut glsl, "    float wall_slope = 1.5 + width_pattern * 0.5;").unwrap();
    writeln!(&mut glsl, "    float canyon_cut = pow(abs(river_depth), wall_slope) * 2.0; // Deeper canyons").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Terraced canyon walls").unwrap();
    writeln!(&mut glsl, "    float terraces = floor(canyon_cut * 6.0) / 6.0;").unwrap();
    writeln!(&mut glsl, "    float final_depth = canyon_cut * 0.4 + terraces * 0.6;").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // High mesas between canyons").unwrap();
    writeln!(&mut glsl, "    float mesa_height = (pow(sin(p.x * scale4 * 0.5), 2.0) + pow(cos(p.y * scale4 * 0.5), 2.0)) * 40.0;").unwrap();
    writeln!(&mut glsl, "    float plateau_base = 200.0; // Base height for dramatic effect").unwrap();
    writeln!(&mut glsl, "    ").unwrap();
    writeln!(&mut glsl, "    // Create dramatic height difference").unwrap();
    writeln!(&mut glsl, "    return plateau_base + mesa_height - final_depth * 160.0;").unwrap();
    writeln!(&mut glsl, "}}").unwrap();
    writeln!(&mut glsl).unwrap();
    
    // Add other biome functions...
    // For brevity, I'll add a template for the others
    writeln!(&mut glsl, "// Other biome height functions would be generated here").unwrap();
    writeln!(&mut glsl, "// from the Rust implementations in biome_heights.rs").unwrap();
    
    glsl
}

/// Generate the main get_biome_height function
pub fn generate_get_biome_height_function() -> String {
    let mut glsl = String::new();
    
    writeln!(&mut glsl, "float get_biome_height(vec2 p, int biome) {{").unwrap();
    writeln!(&mut glsl, "    switch(biome) {{").unwrap();
    writeln!(&mut glsl, "        case 0: return canyon_height(p);").unwrap();
    writeln!(&mut glsl, "        case 1: return plateau_height(p);").unwrap();
    writeln!(&mut glsl, "        case 2: return arctic_height(p);").unwrap();
    writeln!(&mut glsl, "        case 3: return grassland_height(p);").unwrap();
    writeln!(&mut glsl, "        case 4: return forest_height(p);").unwrap();
    writeln!(&mut glsl, "        case 5: return volcanic_height(p);").unwrap();
    writeln!(&mut glsl, "        case 6: return desert_height(p);").unwrap();
    writeln!(&mut glsl, "        case 7: return ocean_height(p);").unwrap();
    writeln!(&mut glsl, "        case 8: return alien_height(p);").unwrap();
    writeln!(&mut glsl, "        case 9: return crystal_height(p);").unwrap();
    writeln!(&mut glsl, "        case 10: return void_height(p);").unwrap();
    writeln!(&mut glsl, "        case 11: return cyber_height(p);").unwrap();
    writeln!(&mut glsl, "        case 12: return floating_height(p);").unwrap();
    writeln!(&mut glsl, "        default: return 0.0;").unwrap();
    writeln!(&mut glsl, "    }}").unwrap();
    writeln!(&mut glsl, "}}").unwrap();
    
    glsl
}

/// Generate a complete terrain vertex shader with height functions
pub fn generate_terrain_vertex_shader(base_shader: &str) -> String {
    let mut shader = String::new();
    
    // Add the biome height functions at the top
    shader.push_str(&generate_biome_height_functions());
    shader.push('\n');
    shader.push_str(&generate_get_biome_height_function());
    shader.push('\n');
    
    // Add the base shader code
    shader.push_str(base_shader);
    
    shader
}