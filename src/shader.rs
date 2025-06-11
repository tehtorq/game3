use miniquad::*;
use glam::Mat4;

pub const VERTEX: &str = r#"#version 100
attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    gl_Position = mvp * vec4(pos, 1.0);
}"#;

pub const VERTEX_INSTANCED_TERRAIN: &str = r#"#version 100
precision mediump float;

attribute vec3 pos;
attribute vec3 barycentric;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

uniform mat4 mvp;
uniform float terrain_scale;

void main() {
    v_barycentric = barycentric;
    
    // Vertex positions already contain the correct heights
    vec3 world_pos = pos;
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

pub const FRAGMENT_TERRAIN: &str = r#"#version 100
precision mediump float;

uniform vec3 color;
uniform sampler2D height_texture;
uniform sampler2D biome_texture;
uniform float terrain_scale;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

void main() {
    // Sample biome color from texture
    vec2 uv = (v_world_xz / terrain_scale) + 0.5;
    vec3 biome_color = texture2D(biome_texture, uv).rgb;
    
    // Calculate height-based brightness (higher = brighter)
    // Terrain can range from about -500 to +500, normalize to this range
    float height_factor = (v_height + 500.0) / 1000.0; // Normalize to 0-1 range
    height_factor = clamp(height_factor, 0.0, 1.0);
    height_factor = 0.3 + height_factor * 0.7; // Map to 0.3-1.0 range for more contrast
    
    // Blend base color with biome color
    vec3 blended_color = mix(color, biome_color, 0.7); // 70% biome color, 30% base color
    vec3 adjusted_color = blended_color * height_factor;
    
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(adjusted_color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(adjusted_color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(adjusted_color, 1.0); // Bright version for edges
        }
    }
}"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(color, 1.0); // Bright version for edges
        }
    }
}"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
            ],
        },
    }
}

pub fn meta_terrain() -> ShaderMeta {
    ShaderMeta {
        images: vec!["height_texture".to_string(), "biome_texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
                UniformDesc::new("terrain_scale", UniformType::Float1),
                UniformDesc::new("terrain_y_base", UniformType::Float1),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
}

impl Uniforms {
    pub fn new(mvp: Mat4, color: [f32; 3]) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
        }
    }
}

#[repr(C)]
pub struct UniformsTerrain {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
    pub terrain_scale: f32,
    pub terrain_y_base: f32,
    pub _padding2: [f32; 2],
}

impl UniformsTerrain {
    pub fn new(mvp: Mat4, color: [f32; 3], terrain_scale: f32, terrain_y_base: f32) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
            terrain_scale,
            terrain_y_base,
            _padding2: [0.0, 0.0],
        }
    }
}