#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_focus_position;  // Y position of focus band (0.0 to 1.0)
uniform float u_focus_scale;     // Width of focused area
uniform float u_blur_amount;     // Maximum blur strength
uniform float u_saturation;      // Color saturation multiplier
uniform float u_time;           // For animated effects if needed

varying vec2 v_uv;

// Simple box blur for performance (can be replaced with gaussian)
vec4 blur_sample(vec2 uv, float blur_size) {
    vec2 texel_size = 1.0 / u_screen_size;
    vec4 result = vec4(0.0);
    float total = 0.0;
    
    // 9-tap box blur
    for (float x = -1.0; x <= 1.0; x += 1.0) {
        for (float y = -1.0; y <= 1.0; y += 1.0) {
            vec2 offset = vec2(x, y) * texel_size * blur_size;
            result += texture2D(u_scene_texture, uv + offset);
            total += 1.0;
        }
    }
    
    return result / total;
}

// Better quality blur using gaussian weights
vec4 gaussian_blur(vec2 uv, float blur_size) {
    vec2 texel_size = 1.0 / u_screen_size;
    vec4 result = vec4(0.0);
    float total = 0.0;
    
    // 13-tap gaussian approximation
    const int samples = 6;
    float weights[7];
    weights[0] = 0.227027;
    weights[1] = 0.1945946;
    weights[2] = 0.1216216;
    weights[3] = 0.054054;
    weights[4] = 0.016216;
    weights[5] = 0.003;
    weights[6] = 0.0005;
    
    // Center sample
    result += texture2D(u_scene_texture, uv) * weights[0];
    total += weights[0];
    
    // Blur in both directions
    for (int i = 1; i <= samples; i++) {
        vec2 offset = texel_size * float(i) * blur_size;
        
        // Horizontal and vertical samples
        result += texture2D(u_scene_texture, uv + vec2(offset.x, 0.0)) * weights[i];
        result += texture2D(u_scene_texture, uv - vec2(offset.x, 0.0)) * weights[i];
        result += texture2D(u_scene_texture, uv + vec2(0.0, offset.y)) * weights[i];
        result += texture2D(u_scene_texture, uv - vec2(0.0, offset.y)) * weights[i];
        
        total += weights[i] * 4.0;
    }
    
    return result / total;
}

// Adjust color saturation
vec3 adjust_saturation(vec3 color, float saturation) {
    float gray = dot(color, vec3(0.299, 0.587, 0.114));
    return mix(vec3(gray), color, saturation);
}

void main() {
    // Simple pass-through to test texture sampling
    vec4 color = texture2D(u_scene_texture, v_uv);
    
    // Only apply tilt-shift if blur amount > 0
    if (u_blur_amount > 0.01) {
        // Calculate distance from focus band
        float dist_from_focus = abs(v_uv.y - u_focus_position);
        
        // Create smooth blur falloff
        float blur_factor = smoothstep(0.0, u_focus_scale, dist_from_focus);
        
        // Make blur stronger at top and bottom of screen
        blur_factor = pow(blur_factor, 1.5);
        
        // Calculate blur amount
        float blur = blur_factor * u_blur_amount;
        
        // Sample the scene with blur
        if (blur > 0.1) {
            if (blur < 2.0) {
                // Light blur
                color = blur_sample(v_uv, blur);
            } else {
                // Heavy blur - use gaussian
                color = gaussian_blur(v_uv, blur * 0.5);
            }
        }
        
        // Enhance saturation for toy-like appearance
        color.rgb = adjust_saturation(color.rgb, u_saturation);
        
        // Add subtle vignetting
        float vignette = 1.0 - length(v_uv - 0.5) * 0.4;
        vignette = smoothstep(0.0, 1.0, vignette);
        color.rgb *= vignette;
        
        // Slight contrast boost
        color.rgb = pow(color.rgb, vec3(0.95));
    }
    
    gl_FragColor = color;
}