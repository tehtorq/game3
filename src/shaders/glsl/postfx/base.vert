#version 100
attribute vec2 pos;
attribute vec2 uv;

varying vec2 v_uv;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    v_uv = uv;
}