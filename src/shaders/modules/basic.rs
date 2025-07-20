pub const VERTEX: &str = include_str!("../glsl/basic.vert");
pub const FRAGMENT: &str = include_str!("../glsl/basic.frag");

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