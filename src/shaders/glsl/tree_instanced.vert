#version 100
attribute vec3 pos;
attribute vec3 barycentric;
attribute vec3 instance_position;
attribute vec3 instance_scale_height; // x=width, y=height, z=depth
attribute float instance_rotation;

uniform mat4 mvp;

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
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}