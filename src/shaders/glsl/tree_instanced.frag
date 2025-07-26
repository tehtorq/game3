#version 100
precision highp float;

uniform mat4 mvp;
uniform vec3 color;
uniform vec3 camera_pos;

varying vec3 v_barycentric;
varying vec3 v_world_pos;

void main() {
    // Simple shading based on height for variety
    float height_factor = clamp(v_world_pos.y / 100.0, 0.0, 1.0);
    vec3 final_color = color * (0.7 + height_factor * 0.3);
    
    // Add some edge highlighting for visibility
    float min_dist = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
    if (min_dist < 0.02) {
        final_color *= 0.8;
    }
    
    gl_FragColor = vec4(final_color, 1.0);
}