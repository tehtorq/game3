#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // Calculate distance to nearest edge
    float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
    
    // Create a bright core with falloff
    float intensity = 1.0 - minBary * 2.0;
    intensity = max(0.0, intensity);
    
    // Add extra brightness to the center
    float coreBrightness = 1.0 + intensity * 2.0;
    
    // Output bright, saturated color with additive blending
    gl_FragColor = vec4(color * coreBrightness, intensity * 0.8);
}