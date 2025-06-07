use miniquad::*;

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        #[repr(C)]
        struct Vec2 {
            x: f32,
            y: f32,
        }
        
        #[repr(C)]
        struct Vertex {
            pos: Vec2,
            color: [f32; 3],
        }

        let vertices: [Vertex; 3] = [
            Vertex { pos: Vec2 { x: -0.5, y: -0.5 }, color: [1., 0., 0.] },
            Vertex { pos: Vec2 { x:  0.5, y: -0.5 }, color: [0., 1., 0.] },
            Vertex { pos: Vec2 { x:  0.0, y:  0.5 }, color: [0., 0., 1.] },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 3] = [0, 1, 2];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Float3),
            ],
            shader,
            PipelineParams::default(),
        );

        Stage { pipeline, bindings }
    }
}

impl EventHandler for Stage {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.));

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.draw(0, 3, 1);
        ctx.end_render_pass();

        ctx.commit_frame();
    }
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec3 color0;
    varying lowp vec3 color;
    void main() {
        gl_Position = vec4(pos, 0, 1);
        color = color0;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec3 color;
    void main() {
        gl_FragColor = vec4(color, 1.0);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![],
            },
        }
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |ctx| Box::new(Stage::new(ctx)));
}