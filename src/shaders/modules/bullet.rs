pub const VERTEX_INSTANCED: &str = include_str!("../glsl/bullet_instanced.vert");

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