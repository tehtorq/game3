#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // Calculate distance to nearest edge
    float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
    
    // Create glow that fades IN towards edges (opposite of normal)
    // This creates a hazy aura around the bullet
    float edgeDist = 1.0 - minBary * 3.0;
    edgeDist = clamp(edgeDist, 0.0, 1.0);
    
    // Smooth falloff for hazy appearance
    float glow = pow(edgeDist, 2.0);
    
    // Make the center transparent and edges glowy
    float alpha = glow * 0.6;
    
    // Bright, saturated color for the glow
    vec3 glowColor = color * 2.0;
    
    // Output with alpha for hazy edges
    gl_FragColor = vec4(glowColor, alpha);
}