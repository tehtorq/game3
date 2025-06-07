use miniquad::*;
use glam::Mat4;

pub const VERTEX: &str = r#"#version 100
attribute vec3 pos;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(pos, 1.0);
}"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

void main() {
    // Bright green for vector graphics
    gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
}"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new(mvp: Mat4) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
        }
    }
}