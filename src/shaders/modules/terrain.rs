// Load terrain functions into the shaders at compile time
const TERRAIN_FUNCTIONS: &str = include_str!("../glsl/terrain_functions.glsl");
const TERRAIN_FUNCTIONS_EXTENDED: &str = include_str!("../glsl/terrain_functions_extended.glsl");

// Simple terrain shaders (GPU complete)
const VERTEX_SIMPLE_RAW: &str = include_str!("../glsl/terrain_simple.vert");
const FRAGMENT_SIMPLE_RAW: &str = include_str!("../glsl/terrain_simple.frag");

// Instanced terrain shaders
const VERTEX_INSTANCED_RAW: &str = include_str!("../glsl/terrain_instanced.vert");
const FRAGMENT_INSTANCED_RAW: &str = include_str!("../glsl/terrain_instanced.frag");

// Process shaders to include the terrain functions
pub fn get_vertex_simple() -> String {
    VERTEX_SIMPLE_RAW.replace("#include \"terrain_functions.glsl\"", TERRAIN_FUNCTIONS)
}

pub fn get_fragment_simple() -> String {
    FRAGMENT_SIMPLE_RAW.to_string()
}

pub fn get_vertex_instanced() -> String {
    VERTEX_INSTANCED_RAW.replace("#include \"terrain_functions_extended.glsl\"", TERRAIN_FUNCTIONS_EXTENDED)
}

pub fn get_fragment_instanced() -> String {
    FRAGMENT_INSTANCED_RAW.to_string()
}

// Expose as static strings for compatibility
lazy_static::lazy_static! {
    pub static ref VERTEX_SIMPLE: &'static str = Box::leak(get_vertex_simple().into_boxed_str());
    pub static ref FRAGMENT_SIMPLE: &'static str = Box::leak(get_fragment_simple().into_boxed_str());
    pub static ref VERTEX_INSTANCED: &'static str = Box::leak(get_vertex_instanced().into_boxed_str());
    pub static ref FRAGMENT_INSTANCED: &'static str = Box::leak(get_fragment_instanced().into_boxed_str());
}

use miniquad::*;

pub fn meta_simple() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
                UniformDesc::new("morph_factor", UniformType::Float1),
                UniformDesc::new("chunk_offset", UniformType::Float2),
                UniformDesc::new("lod_scale", UniformType::Float1),
                UniformDesc::new("camera_pos", UniformType::Float3),
                UniformDesc::new("time", UniformType::Float1),
            ],
        },
    }
}

pub fn meta_instanced() -> ShaderMeta {
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