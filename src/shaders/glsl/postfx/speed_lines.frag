#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_speed_factor;  // 0-1 based on current speed
uniform vec2 u_movement_dir;   // Direction of movement in screen space
uniform float u_time;

varying vec2 v_uv;

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void main() {
    vec4 color = texture2D(u_scene_texture, v_uv);
    
    if (u_speed_factor > 0.1) {
        // Convert to polar coordinates from center
        vec2 center = vec2(0.5, 0.5);
        vec2 dir = v_uv - center;
        float dist = length(dir);
        float angle = atan(dir.y, dir.x);
        
        // Create speed lines
        float line_density = 30.0;
        float line = sin(angle * line_density + u_time * 20.0);
        line = smoothstep(0.8, 0.95, abs(line));
        
        // Fade lines based on distance from center
        float fade = smoothstep(0.2, 0.8, dist);
        line *= fade;
        
        // Add some noise for variation
        line *= random(vec2(floor(angle * line_density), 0.0));
        
        // Create radial blur effect
        vec2 blur_dir = normalize(dir);
        vec3 blur_color = vec3(0.0);
        float blur_samples = 5.0;
        
        for (float i = 0.0; i < blur_samples; i++) {
            float t = i / blur_samples;
            vec2 sample_pos = v_uv + blur_dir * t * 0.05 * u_speed_factor;
            blur_color += texture2D(u_scene_texture, sample_pos).rgb;
        }
        blur_color /= blur_samples;
        
        // Mix original with blurred based on lines
        color.rgb = mix(color.rgb, blur_color, line * u_speed_factor * 0.7);
        
        // Add white streaks
        color.rgb += vec3(line * u_speed_factor * 0.3);
    }
    
    gl_FragColor = color;
}