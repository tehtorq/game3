#version 100
attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    gl_Position = mvp * vec4(pos, 1.0);
}