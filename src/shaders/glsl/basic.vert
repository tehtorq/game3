#version 100
precision highp float;

attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;
uniform vec3 color;
uniform vec3 camera_pos;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    // Apply camera-relative positioning to avoid floating-point precision issues
    vec3 relative_pos = pos - camera_pos;
    gl_Position = mvp * vec4(relative_pos, 1.0);
}