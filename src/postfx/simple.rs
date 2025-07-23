use miniquad::*;

pub struct SimplePostFx {
    pipeline: Pipeline,
    bindings: Bindings,
    render_pass: RenderPass,
    color_img: TextureId,
    depth_img: TextureId,
    enabled: bool,
}

impl SimplePostFx {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        let (w, h) = window::screen_size();
        
        let color_img = ctx.new_render_texture(TextureParams {
            width: w as _,
            height: h as _,
            format: TextureFormat::RGBA8,
            ..Default::default()
        });
        
        let depth_img = ctx.new_render_texture(TextureParams {
            width: w as _,
            height: h as _,
            format: TextureFormat::Depth,
            ..Default::default()
        });

        let render_pass = ctx.new_render_pass(color_img, Some(depth_img));

        // Create fullscreen quad
        let vertices: &[f32] = &[
            /* pos         uvs */
            -1.0, -1.0,    0.0, 0.0,
             1.0, -1.0,    1.0, 0.0,
             1.0,  1.0,    1.0, 1.0,
            -1.0,  1.0,    0.0, 1.0,
        ];

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![color_img],
        };

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default()
        );

        Self {
            pipeline,
            bindings,
            render_pass,
            color_img,
            depth_img,
            enabled: false,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        println!("PostFX: {}", if self.enabled { "ON" } else { "OFF" });
    }

    pub fn begin_frame(&mut self, ctx: &mut dyn RenderingBackend) {
        if self.enabled {
            ctx.begin_pass(
                Some(self.render_pass),
                PassAction::clear_color(0.1, 0.2, 0.3, 1.0),
            );
        } else {
            ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        }
    }

    pub fn end_frame(&mut self, ctx: &mut dyn RenderingBackend) {
        ctx.end_render_pass();
        
        if !self.enabled {
            return;
        }

        // Render the post-processing effect
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    pub fn resize(&mut self, ctx: &mut dyn RenderingBackend, width: f32, height: f32) {
        ctx.delete_texture(self.color_img);
        ctx.delete_texture(self.depth_img);
        ctx.delete_render_pass(self.render_pass);

        self.color_img = ctx.new_render_texture(TextureParams {
            width: width as _,
            height: height as _,
            format: TextureFormat::RGBA8,
            ..Default::default()
        });
        
        self.depth_img = ctx.new_render_texture(TextureParams {
            width: width as _,
            height: height as _,
            format: TextureFormat::Depth,
            ..Default::default()
        });

        self.render_pass = ctx.new_render_pass(self.color_img, Some(self.depth_img));
        self.bindings.images[0] = self.color_img;
    }
}

const VERTEX: &str = r#"#version 100
attribute vec2 pos;
attribute vec2 uv;

varying lowp vec2 texcoord;

void main() {
    gl_Position = vec4(pos, 0, 1);
    texcoord = uv;
}
"#;

const FRAGMENT: &str = r#"#version 100
precision lowp float;

varying vec2 texcoord;

uniform sampler2D tex;

void main() {
    vec4 color = texture2D(tex, texcoord);
    // Simple passthrough for now
    gl_FragColor = color;
}
"#;

fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["tex".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![],
        },
    }
}