#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_time;

// Simple passthrough - this would be replaced with specific effect combinations
varying vec2 v_uv;

void main() {
    gl_FragColor = texture2D(u_scene_texture, v_uv);
}