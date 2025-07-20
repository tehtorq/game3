#version 100
attribute vec3 pos;
attribute vec3 barycentric;
attribute vec3 instance_position;
attribute float instance_scale;

uniform mat4 mvp;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    vec3 world_pos = pos * instance_scale + instance_position;
    gl_Position = mvp * vec4(world_pos, 1.0);
}