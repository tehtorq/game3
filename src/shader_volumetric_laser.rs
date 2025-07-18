// Simplified volumetric laser shader using a cylindrical volume approach
pub const VERTEX_VOLUMETRIC_LASER: &str = r#"#version 100
precision highp float;

attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;
uniform vec3 laserStart;
uniform vec3 laserEnd;
uniform float laserRadius;

varying vec3 vPos;
varying vec3 vBarycentric;
varying vec3 vWorldPos;
varying vec3 vLaserStart;
varying vec3 vLaserEnd;
varying float vLaserRadius;

void main() {
    gl_Position = mvp * vec4(pos, 1.0);
    vPos = pos;
    vBarycentric = barycentric;
    vWorldPos = pos;
    vLaserStart = laserStart;
    vLaserEnd = laserEnd;
    vLaserRadius = laserRadius;
}
"#;

pub const FRAGMENT_VOLUMETRIC_LASER: &str = r#"#version 100
precision mediump float;

varying vec3 vPos;
varying vec3 vBarycentric;
varying vec3 vWorldPos;
varying vec3 vLaserStart;
varying vec3 vLaserEnd;
varying float vLaserRadius;

uniform vec3 color;
uniform float time;
uniform vec3 cameraPos;

// Ray marching parameters - reduced for performance
const int MAX_STEPS = 16;
const float STEP_SIZE = 20.0;

// Distance to line segment
float distanceToLineSegment(vec3 p, vec3 a, vec3 b) {
    vec3 ab = b - a;
    vec3 ap = p - a;
    float t = clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
    vec3 closest = a + t * ab;
    return length(p - closest);
}

// Simple volumetric density
float getDensity(vec3 pos) {
    float dist = distanceToLineSegment(pos, vLaserStart, vLaserEnd);
    
    // Sharp falloff for performance
    if(dist > vLaserRadius * 3.0) return 0.0;
    
    // Core beam
    float core = exp(-dist * dist / (vLaserRadius * vLaserRadius));
    
    // Outer glow
    float glow = exp(-dist / (vLaserRadius * 2.5));
    
    // Add some variation
    float variation = sin(pos.x * 0.02 + time * 2.0) * 0.1 + 0.9;
    
    return (core * 2.0 + glow * 0.5) * variation;
}

void main() {
    // Ray from camera to fragment
    vec3 rayDir = normalize(vWorldPos - cameraPos);
    vec3 rayPos = cameraPos;
    
    // Simple ray marching
    float totalDensity = 0.0;
    
    for(int i = 0; i < MAX_STEPS; i++) {
        vec3 samplePos = rayPos + rayDir * (float(i) * STEP_SIZE);
        
        // Check if we're near the laser
        float dist = distanceToLineSegment(samplePos, vLaserStart, vLaserEnd);
        if(dist < vLaserRadius * 4.0) {
            totalDensity += getDensity(samplePos) * STEP_SIZE * 0.02;
        }
    }
    
    // Convert density to color
    float intensity = 1.0 - exp(-totalDensity * 3.0);
    vec3 laserColor = color * intensity;
    
    // Add bright core
    if(intensity > 0.5) {
        laserColor += vec3(1.0, 1.0, 0.5) * (intensity - 0.5) * 2.0;
    }
    
    // Apply to geometry with wireframe
    float minBary = min(min(vBarycentric.x, vBarycentric.y), vBarycentric.z);
    float wireframe = smoothstep(0.0, 0.02, minBary);
    
    gl_FragColor = vec4(laserColor * wireframe, intensity * 0.8);
}
"#;