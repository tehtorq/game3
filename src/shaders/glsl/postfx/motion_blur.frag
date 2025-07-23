#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform vec2 u_velocity;  // Player velocity in screen space
uniform float u_blur_strength;
uniform float u_time;

varying vec2 v_uv;

void main() {
    vec2 velocity = u_velocity * u_blur_strength;
    
    // Create distinct double/triple vision effect
    vec4 color = texture2D(u_scene_texture, v_uv);
    
    // Add ghost images when moving
    if (length(velocity) > 0.001) {
        // First ghost image (leading)
        vec4 ghost1 = texture2D(u_scene_texture, v_uv - velocity * 0.5);
        
        // Second ghost image (trailing)
        vec4 ghost2 = texture2D(u_scene_texture, v_uv + velocity * 0.5);
        
        // Optional third ghost for stronger effect
        vec4 ghost3 = texture2D(u_scene_texture, v_uv + velocity * 1.0);
        
        // Blend the images together
        // Main image gets highest weight
        color = color * 0.5 + ghost1 * 0.25 + ghost2 * 0.15 + ghost3 * 0.1;
        
        // Add chromatic aberration for more dramatic effect
        float aberration = length(velocity) * 2.0;
        color.r = texture2D(u_scene_texture, v_uv - velocity * 0.3).r * 0.3 + color.r * 0.7;
        color.b = texture2D(u_scene_texture, v_uv + velocity * 0.3).b * 0.3 + color.b * 0.7;
    }
    
    gl_FragColor = color;
}