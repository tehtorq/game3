#version 100
precision mediump float;

attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;
varying float v_ao; // Ambient occlusion factor

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;
uniform sampler2D height_texture;

// Include extended terrain generation functions
#include "terrain_functions_extended.glsl"

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y, pos.z + instance_offset.y);
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Sample height from texture instead of procedural generation
    vec2 uv = (world_pos.xz / terrain_scale) + 0.5;
    
    // Debug: Check if UV coordinates are varying
    // If terrain_scale is 3360 and world_pos.x ranges from about -1680 to +1680
    // then uv should range from 0 to 1
    
    vec4 height_sample = texture2D(height_texture, uv);
    
    // Decode 24-bit height value from RGB channels
    float height_normalized = (height_sample.r * 255.0 * 65536.0 + 
                              height_sample.g * 255.0 * 256.0 + 
                              height_sample.b * 255.0) / 16777215.0;
    
    // Convert from normalized [0,1] to actual height range [-500, 500]
    float height = (height_normalized * 1000.0) - 500.0;
    
    // Since vertex texture fetch isn't working properly, use procedural generation
    // This will use the same improved terrain generation functions
    world_pos.y = get_blended_biome_height_extended(world_pos.xz);
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    // Simple height-based ambient occlusion approximation
    // Lower areas are darker (valleys), higher areas are brighter (peaks)
    float height_normalized_ao = (world_pos.y + 300.0) / 600.0; // Normalize to 0-1 range
    v_ao = 0.5 + height_normalized_ao * 0.5; // Range from 0.5 to 1.0
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}