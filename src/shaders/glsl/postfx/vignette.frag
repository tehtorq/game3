#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_vignette_radius;    // Inner radius where vignette starts
uniform float u_vignette_softness;  // How soft the transition is
uniform float u_vignette_intensity; // Overall strength
uniform vec3 u_vignette_color;      // Color of the vignette (usually black)
uniform float u_pulse_factor;       // For health-based pulsing
uniform float u_time;

varying vec2 v_uv;

void main() {
    vec4 color = texture2D(u_scene_texture, v_uv);
    
    // Calculate distance from center
    vec2 center = vec2(0.5, 0.5);
    float dist = length(v_uv - center);
    
    // Create vignette
    float vignette = smoothstep(u_vignette_radius, u_vignette_radius + u_vignette_softness, dist);
    
    // Add pulsing for low health
    if (u_pulse_factor > 0.0) {
        float pulse = sin(u_time * 4.0) * 0.5 + 0.5;
        vignette = mix(vignette, 1.0, pulse * u_pulse_factor * 0.3);
    }
    
    // Apply vignette
    color.rgb = mix(color.rgb, u_vignette_color, vignette * u_vignette_intensity);
    
    gl_FragColor = color;
}