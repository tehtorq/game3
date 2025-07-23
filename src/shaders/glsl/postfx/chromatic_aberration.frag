#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_aberration_strength;
uniform float u_speed_factor;  // 0-1, increases effect with speed
uniform float u_time;

varying vec2 v_uv;

void main() {
    vec2 center = vec2(0.5, 0.5);
    vec2 offset = (v_uv - center) * u_aberration_strength * u_speed_factor;
    
    // Sample each color channel with different offsets
    float r = texture2D(u_scene_texture, v_uv + offset * 1.0).r;
    float g = texture2D(u_scene_texture, v_uv).g;
    float b = texture2D(u_scene_texture, v_uv - offset * 1.0).b;
    
    // Original alpha
    float a = texture2D(u_scene_texture, v_uv).a;
    
    gl_FragColor = vec4(r, g, b, a);
}