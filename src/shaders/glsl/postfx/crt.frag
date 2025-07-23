#version 100
precision highp float;

uniform sampler2D u_scene_texture;
uniform vec2 u_screen_size;
uniform float u_scanline_intensity;
uniform float u_curvature_amount;
uniform float u_vignette_intensity;
uniform float u_time;

varying vec2 v_uv;

vec2 curve_uv(vec2 uv, float amount) {
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / vec2(6.0, 4.0);
    uv = uv + uv * offset * offset * amount;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void main() {
    // Apply CRT curvature
    vec2 curved_uv = curve_uv(v_uv, u_curvature_amount);
    
    // Check if we're outside the screen
    if (curved_uv.x < 0.0 || curved_uv.x > 1.0 || curved_uv.y < 0.0 || curved_uv.y > 1.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    
    vec3 color = texture2D(u_scene_texture, curved_uv).rgb;
    
    // Scanlines
    float scanline = sin(curved_uv.y * u_screen_size.y * 2.0) * 0.04;
    color -= scanline * u_scanline_intensity;
    
    // RGB phosphor pattern
    vec3 phosphor = vec3(
        sin(curved_uv.x * u_screen_size.x * 3.0) * 0.5 + 0.5,
        sin(curved_uv.x * u_screen_size.x * 3.0 + 2.094) * 0.5 + 0.5,
        sin(curved_uv.x * u_screen_size.x * 3.0 + 4.188) * 0.5 + 0.5
    );
    color *= mix(vec3(1.0), phosphor, 0.1);
    
    // Vignette
    float vignette = length(v_uv - 0.5) * 1.4;
    vignette = 1.0 - vignette * vignette;
    color *= mix(1.0, vignette, u_vignette_intensity);
    
    // Slight flicker
    color *= 0.95 + 0.05 * sin(u_time * 60.0);
    
    gl_FragColor = vec4(color, 1.0);
}