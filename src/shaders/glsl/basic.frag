#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(color, 1.0); // Bright version for edges
        }
    }
}