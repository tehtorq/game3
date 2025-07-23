#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_bloom_threshold;
uniform float u_bloom_intensity;
uniform float u_time;

varying vec2 v_uv;

vec3 sample_blur(vec2 uv, float blur_size) {
    vec2 texel_size = 1.0 / u_screen_size;
    vec3 result = vec3(0.0);
    float total = 0.0;
    
    // 9-tap gaussian-like blur
    for (float x = -2.0; x <= 2.0; x += 1.0) {
        for (float y = -2.0; y <= 2.0; y += 1.0) {
            vec2 offset = vec2(x, y) * texel_size * blur_size;
            float weight = exp(-(x*x + y*y) * 0.1);
            vec3 sample_color = texture2D(u_scene_texture, uv + offset).rgb;
            
            // Extract bright areas
            float brightness = dot(sample_color, vec3(0.299, 0.587, 0.114));
            if (brightness > u_bloom_threshold) {
                result += sample_color * weight;
                total += weight;
            }
        }
    }
    
    return total > 0.0 ? result / total : vec3(0.0);
}

void main() {
    vec3 color = texture2D(u_scene_texture, v_uv).rgb;
    
    // Sample and blur bright areas
    vec3 bloom = sample_blur(v_uv, 4.0) * u_bloom_intensity;
    
    // Additive blending
    gl_FragColor = vec4(color + bloom, 1.0);
}