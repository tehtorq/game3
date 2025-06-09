// Enhanced terrain shader that uses vertex texture fetch when available
pub const VERTEX_INSTANCED_TERRAIN_ENHANCED: &str = r#"#version 100
attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;
uniform sampler2D height_texture;

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y + 20.0, pos.z + instance_offset.y);
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Sample height from texture - this will work on platforms that support vertex texture fetch
    vec2 uv = (world_pos.xz / terrain_scale) + 0.5;
    vec4 height_sample = texture2D(height_texture, uv);
    
    // Convert from normalized [0,1] to actual height range [-500, 500]
    world_pos.y += (height_sample.r * 1000.0) - 500.0;
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

// Check if vertex texture fetch is supported
pub fn check_vertex_texture_support(ctx: &dyn miniquad::RenderingBackend) -> bool {
    // This would need to be implemented by checking GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS
    // For now, we'll use the fallback shader
    false
}