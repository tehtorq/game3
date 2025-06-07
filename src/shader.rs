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
attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y + 20.0, pos.z + instance_offset.y);
    
    // Large-scale terrain features - very broad valleys and mountains (4x amplified)
    float large_scale = sin(world_pos.x * 0.0005) * sin(world_pos.z * 0.0007) * 240.0;
    large_scale += cos(world_pos.x * 0.0003 + 1.5) * sin(world_pos.z * 0.0004 - 0.8) * 200.0;
    
    // Create base terrain with gentle slopes
    float gentle = sin(world_pos.x * 0.0031) * cos(world_pos.z * 0.0027) * 25.0;
    gentle += sin(world_pos.x * 0.0047) * sin(world_pos.z * 0.0053) * 20.0;
    
    // Create a "roughness map" that determines where bumpy areas appear
    float roughness = sin(world_pos.x * 0.0023 + 2.7) * cos(world_pos.z * 0.0019 - 1.3);
    roughness += sin(world_pos.x * 0.0041 - world_pos.z * 0.0037) * 0.5;
    roughness = (roughness + 1.5) / 3.0; // Normalize to ~0-1 range
    
    // Make roughness more sparse by thresholding
    roughness = smoothstep(0.6, 0.8, roughness); // Only areas above 0.6 become rough
    
    // Bumpy terrain details
    float bumps = 0.0;
    bumps += sin(world_pos.x * 0.0173) * sin(world_pos.z * 0.0199) * 20.0;
    bumps += cos(world_pos.x * 0.0293 + 2.1) * sin(world_pos.z * 0.0311 - 1.7) * 15.0;
    bumps += sin(world_pos.x * 0.0519 + world_pos.z * 0.0413) * 8.0;
    bumps += sin(world_pos.x * 0.0871 - world_pos.z * 0.0926) * 5.0;
    bumps += sin(world_pos.x * 0.137) * cos(world_pos.z * 0.149) * 3.0;
    
    // Combine all terrain features: large scale + gentle slopes + sparse bumpy areas
    world_pos.y += large_scale + gentle + (bumps * roughness);
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

pub const FRAGMENT_TERRAIN: &str = r#"#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;
varying float v_height;

void main() {
    // Calculate height-based brightness (higher = brighter)
    float height_factor = (v_height - 0.0) / 100.0; // Normalize height to 0-1 range
    height_factor = clamp(height_factor, 0.0, 1.0);
    height_factor = 0.3 + height_factor * 0.7; // Map to 0.3-1.0 range
    
    vec3 adjusted_color = color * height_factor;
    
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
        images: vec![],
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