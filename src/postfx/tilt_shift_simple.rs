use miniquad::*;
use crate::shader;

pub struct TiltShiftEffect {
    pipeline: Pipeline,
    bindings: Bindings,
    screen_texture: TextureId,
    depth_texture: TextureId,
    render_pass: RenderPass,
    enabled: bool,
    screen_width: u32,
    screen_height: u32,
}

impl TiltShiftEffect {
    pub fn new(ctx: &mut Context) -> Self {
        let (width, height) = window::screen_size();
        let screen_width = width as u32;
        let screen_height = height as u32;

        // Create render target using the same approach as individual effects
        let screen_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        let depth_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::Depth,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        let render_pass = ctx.new_render_pass(screen_texture, Some(depth_texture));

        // Create shader using the existing tilt shift shader
        let shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: &*shader::VERTEX_TILT_SHIFT,
                fragment: &*shader::FRAGMENT_TILT_SHIFT,
            },
            shader::meta_tilt_shift(),
        ).expect("Failed to create tilt shift shader");

        let pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16, // 4 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::Never,
                depth_write: false,
                ..Default::default()
            },
        );

        // Create fullscreen quad
        #[repr(C)]
        struct QuadVertex {
            pos: [f32; 2],
            uv: [f32; 2],
        }

        let vertices = [
            QuadVertex { pos: [-1.0, -1.0], uv: [0.0, 1.0] },
            QuadVertex { pos: [ 1.0, -1.0], uv: [1.0, 1.0] },
            QuadVertex { pos: [ 1.0,  1.0], uv: [1.0, 0.0] },
            QuadVertex { pos: [-1.0,  1.0], uv: [0.0, 0.0] },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![screen_texture],
        };

        Self {
            pipeline,
            bindings,
            screen_texture,
            depth_texture,
            render_pass,
            enabled: false,
            screen_width,
            screen_height,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        println!("Tilt Shift: {}", if self.enabled { "ON" } else { "OFF" });
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn begin_frame(&mut self, ctx: &mut Context) {
        let (width, height) = window::screen_size();
        if self.screen_width != width as u32 || self.screen_height != height as u32 {
            self.resize(ctx, width as u32, height as u32);
        }

        if self.enabled {
            ctx.begin_pass(
                Some(self.render_pass),
                PassAction::Clear {
                    color: Some((0.0, 0.0, 0.0, 1.0)),
                    depth: Some(1.0),
                    stencil: None,
                },
            );
        } else {
            ctx.begin_default_pass(PassAction::Clear {
                color: Some((0.0, 0.0, 0.0, 1.0)),
                depth: Some(1.0),
                stencil: None,
            });
        }
    }

    pub fn apply(&mut self, ctx: &mut Context) {
        ctx.end_render_pass();
        
        if !self.enabled {
            return;
        }

        // Apply the tilt shift effect
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        
        #[repr(C)]
        struct Uniforms {
            screen_size: [f32; 2],
            focus_position: f32,
            focus_scale: f32,
            blur_amount: f32,
            saturation: f32,
            time: f32,
        }
        
        let uniforms = Uniforms {
            screen_size: [self.screen_width as f32, self.screen_height as f32],
            focus_position: 0.5,
            focus_scale: 0.3,
            blur_amount: 5.0,  // Enable blur
            saturation: 1.2,   // Slight saturation boost
            time: 0.0,
        };
        
        ctx.apply_uniforms(UniformsSource::table(&uniforms));
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;

        ctx.delete_texture(self.screen_texture);
        ctx.delete_texture(self.depth_texture);
        ctx.delete_render_pass(self.render_pass);

        self.screen_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                ..Default::default()
            },
        );

        self.depth_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::Depth,
                width,
                height,
                ..Default::default()
            },
        );

        self.render_pass = ctx.new_render_pass(self.screen_texture, Some(self.depth_texture));
        self.bindings.images[0] = self.screen_texture;
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        if keycode == KeyCode::T {
            self.toggle();
        }
    }
}