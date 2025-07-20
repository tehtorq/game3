#version 100
precision highp float;

uniform vec3 color;
uniform float time;
uniform vec3 camera_pos;

varying vec3 v_barycentric;
varying float v_height;
varying vec3 v_normal;
varying vec3 v_world_pos;
varying vec3 v_biome_color;
varying vec2 v_local_pos;

void main() {
    // Use biome color as base
    vec3 terrain_color = v_biome_color;
    
    // Add height-based variation
    float height_factor = smoothstep(-100.0, 400.0, v_height);
    terrain_color = mix(terrain_color * 0.7, terrain_color * 1.1, height_factor);
    
    // Enhanced water rendering for low areas
    // Check if we're at or below water level
    float water_level = -30.0;
    if (v_height < water_level + 20.0) {
        // Create water pattern based on world position
        // This creates a static but realistic water appearance
        float wave1 = sin(v_world_pos.x * 0.02) * cos(v_world_pos.z * 0.015);
        float wave2 = sin(v_world_pos.x * 0.01 + v_world_pos.z * 0.01);
        float wave3 = cos(v_world_pos.x * 0.005) * sin(v_world_pos.z * 0.008);
        float wave_pattern = (wave1 * 0.4 + wave2 * 0.3 + wave3 * 0.3) * 0.5 + 0.5;
        
        // Calculate how close we are to the water surface
        float distance_to_surface = abs(v_height - water_level);
        float surface_proximity = 1.0 - smoothstep(0.0, 10.0, distance_to_surface);
        
        if (v_height < water_level) {
            // Underwater terrain - balanced blue tint
            vec3 shallow_underwater = vec3(0.3, 0.5, 0.8);
            vec3 deep_underwater = vec3(0.1, 0.2, 0.5);
            float depth = water_level - v_height;
            float depth_factor = smoothstep(0.0, 100.0, depth);
            
            // Mix between shallow and deep water colors
            vec3 underwater_tint = mix(shallow_underwater, deep_underwater, depth_factor);
            
            // Apply moderate blue tint - keep some terrain visibility
            terrain_color = terrain_color * 0.5 + underwater_tint * 0.5;
            
            // Add caustics effect
            float caustics = sin(v_world_pos.x * 0.1 + wave_pattern * 3.0) * 
                           cos(v_world_pos.z * 0.1 - wave_pattern * 2.0);
            caustics = caustics * 0.5 + 0.5;
            terrain_color += vec3(0.05, 0.1, 0.15) * caustics * (1.0 - depth_factor) * 0.3;
            
            // Subtle fog effect for deep water
            float fog_factor = smoothstep(100.0, 300.0, depth);
            terrain_color = mix(terrain_color, deep_underwater, fog_factor * 0.4);
        }
        
        // Add water surface effect when near water level
        if (surface_proximity > 0.01) {
            vec3 water_color = vec3(0.05, 0.25, 0.7);  // Deeper blue
            vec3 highlight_color = vec3(0.3, 0.6, 1.0);  // Brighter blue highlights
            
            // Surface highlights
            float surface_highlight = pow(wave_pattern, 2.0) * surface_proximity;
            vec3 surface_color = mix(water_color, highlight_color, surface_highlight);
            
            // Blend surface with terrain
            terrain_color = mix(terrain_color, surface_color, surface_proximity * 0.8);
        }
        
    }
    
    // Directional lighting with multiple light sources
    vec3 sun_dir = normalize(vec3(0.5, 1.0, 0.3));
    vec3 moon_dir = normalize(vec3(-0.3, 0.5, -0.7));
    
    float sun_light = max(dot(v_normal, sun_dir), 0.0);
    float moon_light = max(dot(v_normal, moon_dir), 0.0) * 0.3;
    
    // Check if this is near water surface
    float is_water = 1.0 - smoothstep(0.0, 10.0, abs(v_height - (-30.0)));
    
    // Enhanced lighting for water
    if (is_water > 0.01) {
        // Simulate specular highlights using sun direction and normal
        // Use a fixed view direction approximation
        vec3 approx_view_dir = normalize(vec3(0.0, 0.5, -1.0));
        vec3 reflect_dir = reflect(-sun_dir, v_normal);
        float spec = pow(max(dot(approx_view_dir, reflect_dir), 0.0), 16.0);
        
        // Water gets more ambient light and specular
        float water_light = sun_light * 0.6 + moon_light + 0.5 + spec * 0.5;
        float terrain_light = sun_light * 0.8 + moon_light + 0.3;
        
        // Mix between water and terrain lighting
        float light = mix(terrain_light, water_light, is_water);
        terrain_color *= light;
    } else {
        // Normal terrain lighting
        float light = sun_light * 0.8 + moon_light + 0.3;
        
        // Add rim lighting for terrain
        float rim = 1.0 - max(dot(v_normal, vec3(0.0, 1.0, 0.0)), 0.0);
        rim = pow(rim, 2.0) * 0.2;
        light += rim;
        
        terrain_color *= light;
    }
    
    // Add subtle detail texture using world position
    float detail = sin(v_world_pos.x * 0.1) * cos(v_world_pos.z * 0.1) * 0.05;
    terrain_color += vec3(detail);
    
    // Wireframe effect on edges (optional, subtle)
    float wire = min(v_barycentric.x, min(v_barycentric.y, v_barycentric.z));
    float line_width = 0.015;
    
    if (wire < line_width) {
        vec3 wire_color = terrain_color * 1.2 + vec3(0.1);
        terrain_color = mix(wire_color, terrain_color, wire / line_width);
    }
    
    // Circular fade-out effect
    // Use the local position (which is relative to the terrain grid center)
    // The terrain grid is always centered at the player, so local pos gives us
    // the distance from the player/center correctly
    float distance_from_center = length(v_local_pos);
    float max_visible_distance = 40000.0; // Maximum visible distance (about half of terrain size)
    float fade_start = max_visible_distance * 0.7; // Start fading at 70% of max distance
    
    // Create smooth fade-out
    float fade_factor = 1.0 - smoothstep(fade_start, max_visible_distance, distance_from_center);
    
    // Apply fade to alpha channel for circular terrain effect
    float alpha = fade_factor;
    
    // Also fade the color to black for better visual effect
    terrain_color *= fade_factor;
    
    // Tone mapping for better color range
    terrain_color = terrain_color / (terrain_color + vec3(1.0));
    terrain_color = pow(terrain_color, vec3(1.0/2.2)); // Gamma correction
    
    gl_FragColor = vec4(terrain_color, alpha);
}