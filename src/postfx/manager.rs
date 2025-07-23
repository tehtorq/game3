use miniquad::*;

pub struct PostFxManager {
    enabled: bool,
    offscreen_pass: RenderPass,
    screen_texture: TextureId,
    depth_texture: TextureId,
    screen_width: u32,
    screen_height: u32,
    // Pipeline for applying effects
    effect_pipeline: Pipeline,
    effect_bindings: Bindings,
}

impl PostFxManager {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        let (width, height) = window::screen_size();
        let screen_width = width as u32;
        let screen_height = height as u32;

        let screen_texture = ctx.new_render_texture(TextureParams {
            format: TextureFormat::RGBA8,
            width: screen_width,
            height: screen_height,
            ..Default::default()
        });

        let depth_texture = ctx.new_render_texture(TextureParams {
            format: TextureFormat::Depth,
            width: screen_width,
            height: screen_height,
            ..Default::default()
        });

        let offscreen_pass = ctx.new_render_pass(screen_texture, Some(depth_texture));

        // Create a debug shader to test
        let effect_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: r#"#version 100
                    attribute vec2 pos;
                    attribute vec2 uv;
                    
                    varying lowp vec2 texcoord;
                    
                    void main() {
                        gl_Position = vec4(pos, 0, 1);
                        texcoord = uv;
                    }"#,
                    fragment: r#"#version 100
                    precision lowp float;
                    
                    varying vec2 texcoord;
                    
                    uniform sampler2D tex;
                    
                    void main() {
                        vec4 color = texture2D(tex, texcoord);
                        // Just show the sampled color directly
                        gl_FragColor = color;
                    }"#,
                },
                ShaderMeta {
                    images: vec!["tex".to_string()],
                    uniforms: UniformBlockLayout { uniforms: vec![] },
                },
            )
            .expect("Failed to create postfx shader");

        let effect_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16, // 4 floats * 4 bytes = 16 bytes per vertex
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            effect_shader,
            PipelineParams {
                depth_test: Comparison::Never,
                depth_write: false,
                ..Default::default()
            },
        );

        #[repr(C)]
        struct QuadVertex {
            pos: [f32; 2],
            uv: [f32; 2],
        }

        let quad_vertices = [
            QuadVertex { pos: [-1.0, -1.0], uv: [0.0, 0.0] },
            QuadVertex { pos: [1.0, -1.0], uv: [1.0, 0.0] },
            QuadVertex { pos: [1.0, 1.0], uv: [1.0, 1.0] },
            QuadVertex { pos: [-1.0, 1.0], uv: [0.0, 1.0] },
        ];
        let quad_indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let quad_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_vertices),
        );
        let quad_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_indices),
        );

        let effect_bindings = Bindings {
            vertex_buffers: vec![quad_vertex_buffer],
            index_buffer: quad_index_buffer,
            images: vec![screen_texture],
        };

        Self {
            enabled: false,
            offscreen_pass,
            screen_texture,
            depth_texture,
            screen_width,
            screen_height,
            effect_pipeline,
            effect_bindings,
        }
    }

    pub fn begin_frame(&mut self, ctx: &mut dyn RenderingBackend) {
        let (width, height) = window::screen_size();
        if self.screen_width != width as u32 || self.screen_height != height as u32 {
            self.resize(ctx, width as u32, height as u32);
        }

        if self.enabled {
            println!("PostFX: Beginning offscreen pass");
            ctx.begin_pass(
                Some(self.offscreen_pass),
                PassAction::clear_color(1.0, 0.0, 1.0, 1.0), // Magenta to verify offscreen render
            );
            
            // Test: Draw a simple triangle to see if it appears
            // This is just to verify the render target works
            // We'll remove this once we confirm it works
        } else {
            ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        }
    }

    pub fn apply_effects(&mut self, ctx: &mut dyn RenderingBackend) {
        println!("PostFX: Ending render pass, enabled: {}", self.enabled);
        ctx.end_render_pass();
        
        if !self.enabled {
            return;
        }

        // Apply the effect
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.effect_pipeline);
        ctx.apply_bindings(&self.effect_bindings);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    pub fn resize(&mut self, ctx: &mut dyn RenderingBackend, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;

        ctx.delete_texture(self.screen_texture);
        ctx.delete_texture(self.depth_texture);

        self.screen_texture = ctx.new_render_texture(TextureParams {
            format: TextureFormat::RGBA8,
            width,
            height,
            ..Default::default()
        });

        self.depth_texture = ctx.new_render_texture(TextureParams {
            format: TextureFormat::Depth,
            width,
            height,
            ..Default::default()
        });

        ctx.delete_render_pass(self.offscreen_pass);
        self.offscreen_pass = ctx.new_render_pass(self.screen_texture, Some(self.depth_texture));
        
        // Update bindings with new texture
        self.effect_bindings.images[0] = self.screen_texture;
    }

    pub fn toggle_all(&mut self) {
        self.enabled = !self.enabled;
        println!("PostFX effects: {}", if self.enabled { "ON" } else { "OFF" });
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::T => self.toggle_all(),
            _ => {}
        }
    }
}