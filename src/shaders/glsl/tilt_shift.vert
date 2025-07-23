#version 100
attribute vec2 pos;
attribute vec2 uv;

varying vec2 v_uv;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    // Flip V coordinate to correct for render target inversion
    v_uv = vec2(uv.x, 1.0 - uv.y);
}