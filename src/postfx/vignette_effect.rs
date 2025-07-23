use miniquad::*;
use crate::shaders;
use super::postfx::PostProcessingEffect;

pub struct VignetteEffect {
    pipeline: Pipeline,
    bindings: Bindings,
    output_texture: TextureId,
    render_pass: RenderPass,
    enabled: bool,
    intensity: f32,
    smoothness: f32,
}

impl PostProcessingEffect for VignetteEffect {
    fn new(ctx: &mut Context, screen_width: u32, screen_height: u32) -> Self {
        // Create render target
        let output_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );
        
        let render_pass = ctx.new_render_pass(output_texture, None);
        
        // Create shader
        let shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shaders::modules::postfx::vignette::VERTEX,
                fragment: shaders::modules::postfx::vignette::FRAGMENT,
            },
            ShaderMeta {
                images: vec!["u_scene_texture".to_string()],
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("u_screen_size", UniformType::Float2),
                        UniformDesc::new("u_intensity", UniformType::Float1),
                        UniformDesc::new("u_smoothness", UniformType::Float1),
                        UniformDesc::new("u_speed_factor", UniformType::Float1),
                        UniformDesc::new("u_time", UniformType::Float1),
                    ],
                },
            }
        ).expect("Failed to create vignette shader");
        
        let pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
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
            QuadVertex { pos: [-1.0,  1.0], uv: [0.0, 0.0] },
            QuadVertex { pos: [ 1.0,  1.0], uv: [1.0, 0.0] },
        ];
        
        let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];
        
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
            images: vec![output_texture], // This will be replaced by the input texture
        };
        
        Self {
            pipeline,
            bindings,
            output_texture,
            render_pass,
            enabled: false,
            intensity: 0.3,
            smoothness: 0.5,
        }
    }
    
    fn toggle(&mut self) {
        self.enabled = !self.enabled;
        println!("Vignette: {}", if self.enabled { "ON" } else { "OFF" });
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_input_texture(&mut self, texture: TextureId) {
        self.bindings.images[0] = texture;
    }
    
    fn get_output_texture(&self) -> TextureId {
        self.output_texture
    }
    
    fn apply_effect(
        &mut self,
        ctx: &mut Context,
        screen_width: f32,
        screen_height: f32,
        _input_texture: TextureId,
    ) {
        if !self.enabled {
            return;
        }
        
        ctx.begin_pass(
            Some(self.render_pass),
            PassAction::Clear {
                color: Some((0.0, 0.0, 0.0, 1.0)),
                depth: Some(1.0),
                stencil: None,
            },
        );
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        
        #[repr(C)]
        struct Uniforms {
            screen_size: [f32; 2],
            intensity: f32,
            smoothness: f32,
            speed_factor: f32,
            time: f32,
        }
        
        let uniforms = Uniforms {
            screen_size: [screen_width, screen_height],
            intensity: self.intensity,
            smoothness: self.smoothness,
            speed_factor: 0.0, // This should be updated from the game state
            time: 0.0,
        };
        
        ctx.apply_uniforms(UniformsSource::table(&uniforms));
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        ctx.delete_texture(self.output_texture);
        self.output_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                ..Default::default()
            },
        );
        ctx.delete_render_pass(self.render_pass);
        self.render_pass = ctx.new_render_pass(self.output_texture, None);
    }
}

impl VignetteEffect {
    pub fn increase_intensity(&mut self) {
        self.intensity = (self.intensity + 0.05).min(1.0);
        println!("Vignette intensity: {:.2}", self.intensity);
    }
    
    pub fn decrease_intensity(&mut self) {
        self.intensity = (self.intensity - 0.05).max(0.0);
        println!("Vignette intensity: {:.2}", self.intensity);
    }
}
