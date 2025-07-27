#version 100
precision highp float;

attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;
uniform float morph_factor;  // For geomorphing between LODs
uniform vec2 chunk_offset;   // Chunk position in world
uniform float lod_scale;     // Grid spacing for current LOD
uniform vec3 camera_pos;     // Camera/player position for fog

varying vec3 v_barycentric;
varying float v_height;
varying vec3 v_normal;
varying vec3 v_world_pos;
varying vec3 v_biome_color;
varying vec2 v_local_pos;

// Include terrain generation functions
#include "terrain_functions.glsl"

void main() {
    v_barycentric = barycentric;
    
    // Store local position for fade calculation
    v_local_pos = pos.xz;
    
    // Calculate world position
    vec2 world_xz = pos.xz + chunk_offset;
    
    // Get terrain height
    float height = get_blended_biome_height(world_xz);
    
    // Apply LOD morphing if needed
    float morph_height = height;
    if (morph_factor > 0.001) {
        vec2 grid_size = vec2(lod_scale * 2.0);
        vec2 snapped_pos = floor(world_xz / grid_size + 0.5) * grid_size;
        float snapped_height = get_blended_biome_height(snapped_pos);
        morph_height = mix(height, snapped_height, morph_factor);
        world_xz = mix(world_xz, snapped_pos, morph_factor);
    }
    
    // Store original height for fragment shader
    v_height = morph_height;
    
    // Keep original terrain height - don't clamp
    // Fix: world_xz is vec2(x,z), so we need vec3(x, height, z)
    v_world_pos = vec3(world_xz.x, morph_height, world_xz.y);
    
    // Calculate normal - always use terrain normal
    float delta = mix(2.0, 8.0, morph_factor);
    float hL = get_blended_biome_height(world_xz - vec2(delta, 0.0));
    float hR = get_blended_biome_height(world_xz + vec2(delta, 0.0));
    float hD = get_blended_biome_height(world_xz - vec2(0.0, delta));
    float hU = get_blended_biome_height(world_xz + vec2(0.0, delta));
    
    v_normal = normalize(vec3(hL - hR, 2.0 * delta, hD - hU));
    
    // Get biome color
    v_biome_color = get_biome_color(world_xz, morph_height);
    
    
    // Apply camera-relative positioning to avoid floating-point precision issues
    // Swap Y and Z to fix coordinate system mismatch
    // Negate X to fix horizontal movement direction
    vec3 camera_corrected = vec3(-camera_pos.x, camera_pos.z, camera_pos.y);
    vec3 relative_pos = v_world_pos - camera_corrected;
    
    gl_Position = mvp * vec4(relative_pos, 1.0);
}