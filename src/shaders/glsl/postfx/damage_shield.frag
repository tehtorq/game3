#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_damage_intensity;  // 0-1, pulses when hit
uniform float u_shield_intensity;  // 0-1, active when shields up
uniform vec3 u_damage_color;       // Usually red
uniform vec3 u_shield_color;       // Usually blue/cyan
uniform float u_time;

varying vec2 v_uv;

void main() {
    vec4 color = texture2D(u_scene_texture, v_uv);
    
    // Damage effect - red vignette that pulses
    if (u_damage_intensity > 0.0) {
        float dist = length(v_uv - 0.5) * 2.0;
        float damage_vignette = smoothstep(0.5, 1.5, dist);
        
        // Pulse effect
        float pulse = sin(u_time * 10.0) * 0.1 + 0.9;
        damage_vignette *= u_damage_intensity * pulse;
        
        // Mix in damage color
        color.rgb = mix(color.rgb, u_damage_color, damage_vignette * 0.6);
    }
    
    // Shield effect - hexagonal pattern overlay
    if (u_shield_intensity > 0.0) {
        // Create hexagonal grid
        vec2 p = v_uv * 20.0;
        float hex_x = p.x * 0.866025;
        float hex_y = p.y + p.x * 0.5;
        
        vec2 hex = vec2(floor(hex_x + 0.5), floor(hex_y + 0.5));
        vec2 hex_center = vec2((hex.x - hex.y * 0.5) / 0.866025, hex.y);
        
        float dist = length(p - hex_center);
        float hex_pattern = 1.0 - smoothstep(0.4, 0.5, dist);
        
        // Animated shield shimmer
        float shimmer = sin(u_time * 3.0 + hex.x * 0.5 + hex.y * 0.3) * 0.5 + 0.5;
        hex_pattern *= shimmer;
        
        // Apply shield overlay
        vec3 shield_overlay = u_shield_color * hex_pattern * u_shield_intensity * 0.3;
        color.rgb += shield_overlay;
    }
    
    gl_FragColor = color;
}