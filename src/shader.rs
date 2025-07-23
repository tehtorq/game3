// Import shader modules from crate root
use crate::shaders;

// Re-export commonly used items at the top level for compatibility
pub use crate::shaders::modules::basic::{VERTEX, FRAGMENT};

// For terrain shaders, we need to expose the lazy static values as references
use lazy_static::lazy_static;
lazy_static! {
    pub static ref VERTEX_TERRAIN_SIMPLE: &'static str = *crate::shaders::modules::terrain::VERTEX_SIMPLE;
    pub static ref FRAGMENT_TERRAIN_SIMPLE: &'static str = *crate::shaders::modules::terrain::FRAGMENT_SIMPLE;
    
    pub static ref VERTEX_TILT_SHIFT: &'static str = crate::shaders::modules::tilt_shift::VERTEX;
    pub static ref FRAGMENT_TILT_SHIFT: &'static str = crate::shaders::modules::tilt_shift::FRAGMENT;
    pub static ref VERTEX_TERRAIN: &'static str = *crate::shaders::modules::terrain::VERTEX_INSTANCED;
    pub static ref FRAGMENT_TERRAIN: &'static str = *crate::shaders::modules::terrain::FRAGMENT_INSTANCED;
}

pub use crate::shaders::modules::bullet::VERTEX_INSTANCED as VERTEX_INSTANCED_BULLET;
pub use crate::shaders::modules::glow::FRAGMENT as FRAGMENT_GLOW;
pub use crate::shaders::modules::uniforms::{Uniforms, UniformsTerrainGPU, UniformsTerrain};

// Re-export metadata functions
pub use crate::shaders::modules::basic::meta;
pub use crate::shaders::modules::terrain::meta_simple as meta_terrain_simple;
pub use crate::shaders::modules::terrain::meta_instanced as meta_terrain;

// Volumetric laser meta (from shader_volumetric_laser.rs)
use miniquad::*;

pub fn meta_volumetric_laser() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
                UniformDesc::new("laserStart", UniformType::Float3),
                UniformDesc::new("laserEnd", UniformType::Float3),
                UniformDesc::new("laserRadius", UniformType::Float1),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("cameraPos", UniformType::Float3),
            ],
        },
    }
}

pub fn meta_tilt_shift() -> ShaderMeta {
    ShaderMeta {
        images: vec!["u_scene_texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("u_screen_size", UniformType::Float2),
                UniformDesc::new("u_focus_position", UniformType::Float1),
                UniformDesc::new("u_focus_scale", UniformType::Float1),
                UniformDesc::new("u_blur_amount", UniformType::Float1),
                UniformDesc::new("u_saturation", UniformType::Float1),
                UniformDesc::new("u_time", UniformType::Float1),
            ],
        },
    }
}