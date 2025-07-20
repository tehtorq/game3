// Shared terrain generation functions
// These are included in both terrain shaders

float plains_height(vec2 p) {
    float scale1 = 0.002;
    float scale2 = 0.007;
    float scale3 = 0.015;
    
    float h1 = sin(p.x * scale1) * cos(p.y * scale1) * 40.0;
    float h2 = sin(p.x * scale2 + 100.0) * sin(p.y * scale2 + 100.0) * 20.0;
    float h3 = cos(p.x * scale3 + 200.0) * cos(p.y * scale3 + 200.0) * 10.0;
    
    float ridge = pow(abs(sin(p.x * 0.0005 + p.y * 0.0003)), 3.0) * 30.0;
    float depression = smoothstep(0.6, 0.3, abs(sin(p.x * 0.0008) * cos(p.y * 0.0006))) * -20.0;
    
    return h1 + h2 + h3 + ridge + depression;
}

float canyon_height(vec2 p) {
    float scale1 = 0.0015;
    float scale2 = 0.003;
    float scale3 = 0.006;
    float scale4 = 0.0005;
    
    float river_flow = sin(p.x * scale1) * 0.7 + cos(p.y * scale1 * 0.8) * 0.5;
    float river_meander = (sin(p.x * scale1 * 0.5 + p.y * scale1 * 0.3) + 
                          cos(p.x * scale1 * 0.3 - p.y * scale1 * 0.4)) * 0.4;
    
    float tributary1 = (sin(p.x * scale2 - p.y * scale2 * 0.6) + 
                       cos(p.x * scale2 * 0.4 + p.y * scale2)) * 0.3;
    float tributary2 = (sin(p.x * scale2 * 1.2 + p.y * scale2 * 0.5) * 
                       cos(p.x * scale2 * 0.8 - p.y * scale2 * 0.7)) * 0.25;
    
    float confluence = (abs(tributary1 * river_flow) + abs(tributary2 * river_flow)) * 0.5;
    
    float width_pattern = sin(p.x * scale4 + p.y * scale4 * 0.7);
    float canyon_width = 0.3 + abs(width_pattern) * 0.7 + confluence * 0.3;
    
    float pool_pattern = abs(sin(p.x * scale3) * cos(p.y * scale3 * 1.2));
    float rapids = pow(abs(sin(p.x * scale3 * 2.0 + p.y * scale3 * 1.5)), 3.0) * 0.3;
    
    float river_depth = abs(river_flow + river_meander) * canyon_width + 
                       abs(tributary1) * 0.5 + abs(tributary2) * 0.5 + 
                       confluence + pool_pattern * 0.4 - rapids;
    
    float wall_slope = 1.5 + width_pattern * 0.5;
    float canyon_cut = pow(abs(river_depth), wall_slope) * 2.0;
    
    float terraces = max(floor(canyon_cut * 6.0) / 6.0, 0.0);
    float final_depth = canyon_cut * 0.4 + terraces * 0.6;
    
    float mesa_height = (pow(sin(p.x * scale4 * 0.5), 2.0) + pow(cos(p.y * scale4 * 0.5), 2.0)) * 40.0;
    float plateau_base = 200.0;
    
    return plateau_base + mesa_height - final_depth * 160.0;
}

float mountain_height(vec2 p) {
    float scale1 = 0.0008;
    float scale2 = 0.0015;
    float scale3 = 0.0003;
    float scale4 = 0.004;
    float scale5 = 0.0001;
    
    float range_angle = 0.4;
    float ridge_main = pow(sin(p.x * scale3 * cos(range_angle) + p.y * scale3 * sin(range_angle)) * 0.5 + 0.5, 3.0);
    float ridge_secondary = pow(cos(p.x * scale3 * 1.2 - p.y * scale3 * 0.7) * 0.5 + 0.5, 2.5);
    
    float peak_spacing = 0.0012;
    float peak_line1 = sqrt(pow(sin(p.x * peak_spacing * cos(range_angle) + p.y * peak_spacing * sin(range_angle)), 2.0) + 
                           pow(cos(p.x * peak_spacing * sin(range_angle) - p.y * peak_spacing * cos(range_angle)), 2.0));
    float peak_line2 = sqrt(pow(sin(p.x * peak_spacing * 1.3 + 100.0), 2.0) + 
                           pow(cos(p.y * peak_spacing * 1.3 - 50.0), 2.0));
    
    float peak_sharpness = 2.5;
    float h1 = pow(max(1.0 - peak_line1, 0.0), peak_sharpness) * 270.0;
    float h2 = pow(max(1.0 - peak_line2, 0.0), peak_sharpness * 0.8) * 225.0;
    
    float ridge_height = ridge_main * 180.0 + ridge_secondary * 120.0;
    
    float valley_pattern = sin(p.x * scale2 + p.y * scale2 * 0.6) + 
                          cos(p.x * scale2 * 0.8 - p.y * scale2 * 0.5);
    float valley_depth = pow(abs(valley_pattern), 2.0) * -60.0;
    
    float cliff_pattern = abs(sin(p.x * scale4) * cos(p.y * scale4 * 1.2));
    float cliffs = pow(cliff_pattern, 4.0) * 80.0;
    
    float glacial_valley = pow(abs(sin(p.x * scale2 * 0.5 + p.y * scale2 * 0.7)), 0.5) * -40.0;
    float snow_cap = max(h1 + h2 + ridge_height, 225.0) * 0.2;
    
    float distance_to_ridge = min(abs(ridge_main - 0.5) + abs(ridge_secondary - 0.5), 1.0);
    float foothill_height = pow(1.0 - distance_to_ridge, 0.5) * 60.0;
    
    float tectonic = (sin(p.x * scale5) + cos(p.y * scale5 * 0.8)) * 45.0;
    
    float height = ridge_height + h1 + h2 + valley_depth + cliffs + 
                  glacial_valley + snow_cap + foothill_height + tectonic;
                  
    return clamp(height, -250.0, 450.0);
}

float desert_height(vec2 p) {
    float scale1 = 0.005;
    float scale2 = 0.01;
    float scale3 = 0.03;
    
    float dunes = abs(sin(p.x * scale1) * cos(p.y * scale1 * 1.2)) * 80.0;
    float secondary = cos(p.x * scale2 + 30.0) * sin(p.y * scale2 - 20.0) * 30.0;
    float ripples = sin(p.x * scale3) * cos(p.y * scale3) * 10.0;
    float rocks = pow(abs(sin(p.x * 0.002) * cos(p.y * 0.002)), 4.0) * 60.0;
    float hollows = smoothstep(0.7, 0.5, abs(sin(p.x * 0.004 + p.y * 0.003))) * -30.0;
    
    return -20.0 + dunes + secondary + ripples + rocks + hollows;
}

// Get biome type and height
float get_biome_height(vec2 p) {
    float noise1 = sin(p.x * 0.0003) * cos(p.y * 0.0003);
    float noise2 = sin(p.x * 0.0007 + 1.3) * sin(p.y * 0.0006 - 0.7);
    float noise3 = cos(p.x * 0.0013 - 2.1) * sin(p.y * 0.0011 + 1.9);
    
    float biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    float local_var = sin(p.x * 0.01) * cos(p.y * 0.01) * 0.1;
    biome_noise += local_var;
    
    // Simplified biome selection for shader
    if (biome_noise < -0.3) {
        return canyon_height(p);
    } else if (biome_noise < 0.1) {
        return plains_height(p);
    } else if (biome_noise < 0.5) {
        return desert_height(p);
    } else {
        return mountain_height(p);
    }
}

// Get biome color
vec3 get_biome_color(vec2 p, float height) {
    float noise1 = sin(p.x * 0.0003) * cos(p.y * 0.0003);
    float noise2 = sin(p.x * 0.0007 + 1.3) * sin(p.y * 0.0006 - 0.7);
    float noise3 = cos(p.x * 0.0013 - 2.1) * sin(p.y * 0.0011 + 1.9);
    
    float biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    float local_var = sin(p.x * 0.01) * cos(p.y * 0.01) * 0.1;
    biome_noise += local_var;
    
    vec3 color;
    if (biome_noise < -0.3) {
        // Canyon - red/orange rock
        color = vec3(0.8, 0.5, 0.3);
    } else if (biome_noise < 0.1) {
        // Plains - green grass
        color = vec3(0.4, 0.7, 0.3);
    } else if (biome_noise < 0.5) {
        // Desert - sandy yellow
        color = vec3(0.9, 0.8, 0.5);
    } else {
        // Mountains - gray rock with snow
        color = mix(vec3(0.5, 0.5, 0.6), vec3(0.9, 0.9, 1.0), smoothstep(200.0, 300.0, height));
    }
    
    return color;
}

// Smooth blending between biomes
float get_blended_biome_height(vec2 p) {
    float sample_dist = 100.0;
    float h_center = get_biome_height(p);
    float h_north = get_biome_height(vec2(p.x, p.y + sample_dist));
    float h_south = get_biome_height(vec2(p.x, p.y - sample_dist));
    float h_east = get_biome_height(vec2(p.x + sample_dist, p.y));
    float h_west = get_biome_height(vec2(p.x - sample_dist, p.y));
    
    float primary_height = (h_center * 3.0 + h_north + h_south + h_east + h_west) / 7.0;
    
    // Fractal noise
    float fractal_noise = 0.0;
    float amplitude = 60.0;
    float frequency = 0.0005;
    for (int i = 0; i < 5; i++) {
        fractal_noise += sin(p.x * frequency) * cos(p.y * frequency) * amplitude;
        fractal_noise += sin(p.x * frequency * 1.7 + 100.0) * cos(p.y * frequency * 1.7 + 100.0) * amplitude * 0.7;
        amplitude *= 0.45;
        frequency *= 2.3;
    }
    
    float continent_scale = 0.0001;
    float continental = (sin(p.x * continent_scale) * cos(p.y * continent_scale * 0.8) + 
                        cos(p.x * continent_scale * 0.3) * sin(p.y * continent_scale * 1.2)) * 120.0;
    
    return primary_height + fractal_noise + continental;
}