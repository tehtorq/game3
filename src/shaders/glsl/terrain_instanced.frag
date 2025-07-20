#version 100
precision mediump float;

uniform vec3 color;
uniform sampler2D height_texture;
uniform sampler2D biome_texture;
uniform float terrain_scale;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;
varying float v_ao;

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
    
    // Apply ambient occlusion for valley shadows
    float shadow_factor = mix(0.4, 1.0, v_ao); // AO ranges from 0.4 (darkest) to 1.0 (no shadow)
    
    // Add a subtle directional light effect (simulating sun from northwest)
    vec2 gradient_dir = vec2(1.0, 1.0); // Northwest light direction (not normalized)
    // Use world coordinates directly without normalizing to avoid quadrant artifacts
    float gradient = dot(v_world_xz * 0.00001, gradient_dir) + 0.9; // Very subtle gradient across the world
    gradient = clamp(gradient, 0.8, 1.0); // Keep it subtle - 80% to 100% brightness
    
    // Blend base color with biome color
    vec3 blended_color = mix(color, biome_color, 0.7); // 70% biome color, 30% base color
    vec3 adjusted_color = blended_color * height_factor * shadow_factor * gradient;
    
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
}