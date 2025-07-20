pub const VERTEX_INSTANCED: &str = include_str!("../glsl/tree_instanced.vert");
pub const FRAGMENT_INSTANCED: &str = include_str!("../glsl/tree_instanced.frag");

use miniquad::*;

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