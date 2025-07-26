#version 100
precision highp float;

attribute vec3 pos;
attribute vec3 barycentric;
attribute vec3 instance_position;
attribute vec3 instance_scale_height; // x=width, y=height, z=depth
attribute float instance_rotation;

uniform mat4 mvp;
uniform vec3 color;
uniform vec3 camera_pos;

varying vec3 v_barycentric;
varying vec3 v_world_pos;

void main() {
    v_barycentric = barycentric;
    
    // Apply rotation around Y axis
    float cos_r = cos(instance_rotation);
    float sin_r = sin(instance_rotation);
    vec3 rotated_pos = vec3(
        pos.x * cos_r - pos.z * sin_r,
        pos.y,
        pos.x * sin_r + pos.z * cos_r
    );
    
    // Scale and translate
    vec3 world_pos = rotated_pos * instance_scale_height + instance_position;
    v_world_pos = world_pos;
    
    // Apply camera-relative positioning to avoid floating-point precision issues
    vec3 relative_pos = world_pos - camera_pos;
    gl_Position = mvp * vec4(relative_pos, 1.0);
}