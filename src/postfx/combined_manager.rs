use miniquad::*;
use crate::shader;
use crate::shaders;

pub struct CombinedEffectsManager {
    // Main render target for scene
    scene_texture: TextureId,
    scene_depth: TextureId,
    scene_pass: RenderPass,
    
    // Two textures for ping-ponging between effects
    texture_a: TextureId,
    texture_b: TextureId,
    pass_a: RenderPass,
    pass_b: RenderPass,
    
    // Effects
    tilt_shift: EffectPipeline,
    motion_blur: EffectPipeline,
    
    // Final blit to screen
    blit_pipeline: Pipeline,
    blit_bindings: Bindings,
    
    // State
    effects_enabled: bool,
    tilt_shift_enabled: bool,
    motion_blur_enabled: bool,
    
    screen_width: u32,
    screen_height: u32,
    motion_blur_velocity: [f32; 2],
}

struct EffectPipeline {
    pipeline: Pipeline,
    bindings: Bindings,
}

#[repr(C)]
struct QuadVertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl CombinedEffectsManager {
    pub fn new(ctx: &mut Context) -> Self {
        let (width, height) = window::screen_size();
        let screen_width = width as u32;
        let screen_height = height as u32;

        // Create textures
        let scene_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        let scene_depth = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::Depth,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        let texture_a = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        let texture_b = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width: screen_width,
                height: screen_height,
                ..Default::default()
            },
        );

        // Create render passes
        let scene_pass = ctx.new_render_pass(scene_texture, Some(scene_depth));
        let pass_a = ctx.new_render_pass(texture_a, None);
        let pass_b = ctx.new_render_pass(texture_b, None);

        // Create fullscreen quad vertices
        let quad_vertices = Self::create_quad_vertices();
        let quad_indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_vertices),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_indices),
        );

        // Create tilt shift effect
        let tilt_shift_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: &*shader::VERTEX_TILT_SHIFT,
                fragment: &*shader::FRAGMENT_TILT_SHIFT,
            },
            shader::meta_tilt_shift(),
        ).expect("Failed to create tilt shift shader");

        let tilt_shift_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            tilt_shift_shader,
            PipelineParams {
                depth_test: Comparison::Never,
                depth_write: false,
                ..Default::default()
            },
        );

        let tilt_shift_bindings = Bindings {
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![scene_texture], // Will be updated dynamically
        };

        let tilt_shift = EffectPipeline {
            pipeline: tilt_shift_pipeline,
            bindings: tilt_shift_bindings,
        };

        // Create motion blur effect
        let motion_blur_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shaders::modules::postfx::motion_blur::VERTEX,
                fragment: shaders::modules::postfx::motion_blur::FRAGMENT,
            },
            ShaderMeta {
                images: vec!["u_scene_texture".to_string()],
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("u_screen_size", UniformType::Float2),
                        UniformDesc::new("u_velocity", UniformType::Float2),
                        UniformDesc::new("u_blur_strength", UniformType::Float1),
                        UniformDesc::new("u_time", UniformType::Float1),
                    ],
                },
            }
        ).expect("Failed to create motion blur shader");

        let motion_blur_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            motion_blur_shader,
            PipelineParams {
                depth_test: Comparison::Never,
                depth_write: false,
                ..Default::default()
            },
        );

        let motion_blur_bindings = Bindings {
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![scene_texture], // Will be updated dynamically
        };

        let motion_blur = EffectPipeline {
            pipeline: motion_blur_pipeline,
            bindings: motion_blur_bindings,
        };

        // Create simple blit shader for final output
        let blit_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: r#"#version 100
                attribute vec2 pos;
                attribute vec2 uv;
                varying vec2 v_uv;
                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0);
                    v_uv = uv;
                }"#,
                fragment: r#"#version 100
                precision mediump float;
                varying vec2 v_uv;
                uniform sampler2D u_texture;
                void main() {
                    gl_FragColor = texture2D(u_texture, v_uv);
                }"#,
            },
            ShaderMeta {
                images: vec!["u_texture".to_string()],
                uniforms: UniformBlockLayout { uniforms: vec![] },
            },
        ).expect("Failed to create blit shader");

        let blit_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 16,
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            blit_shader,
            PipelineParams {
                depth_test: Comparison::Never,
                depth_write: false,
                ..Default::default()
            },
        );

        let blit_bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![scene_texture], // Will be updated dynamically
        };

        Self {
            scene_texture,
            scene_depth,
            scene_pass,
            texture_a,
            texture_b,
            pass_a,
            pass_b,
            tilt_shift,
            motion_blur,
            blit_pipeline,
            blit_bindings,
            effects_enabled: false,
            tilt_shift_enabled: false,
            motion_blur_enabled: false,
            screen_width,
            screen_height,
            motion_blur_velocity: [0.0, 0.0],
        }
    }

    fn create_quad_vertices() -> [QuadVertex; 4] {
        [
            QuadVertex { pos: [-1.0, -1.0], uv: [0.0, 1.0] },
            QuadVertex { pos: [ 1.0, -1.0], uv: [1.0, 1.0] },
            QuadVertex { pos: [ 1.0,  1.0], uv: [1.0, 0.0] },
            QuadVertex { pos: [-1.0,  1.0], uv: [0.0, 0.0] },
        ]
    }

    pub fn begin_frame(&mut self, ctx: &mut Context) {
        let (width, height) = window::screen_size();
        if self.screen_width != width as u32 || self.screen_height != height as u32 {
            self.resize(ctx, width as u32, height as u32);
        }

        if self.any_effect_enabled() {
            ctx.begin_pass(
                Some(self.scene_pass),
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

    pub fn apply_effects(&mut self, ctx: &mut Context) {
        ctx.end_render_pass();
        
        if !self.any_effect_enabled() {
            return;
        }

        let mut current_source = self.scene_texture;
        let mut current_target_pass = Some(self.pass_a);
        let mut current_target_texture = self.texture_a;
        let mut use_a = true;

        // Apply tilt shift if enabled
        if self.tilt_shift_enabled {
            self.tilt_shift.bindings.images[0] = current_source;
            
            ctx.begin_pass(current_target_pass, PassAction::Nothing);
            ctx.apply_pipeline(&self.tilt_shift.pipeline);
            ctx.apply_bindings(&self.tilt_shift.bindings);
            
            let uniforms = TiltShiftUniforms {
                screen_size: [self.screen_width as f32, self.screen_height as f32],
                focus_position: 0.4,      // Focus slightly below center
                focus_scale: 0.15,        // Narrower focus band for better effect
                blur_amount: 3.0,         // Moderate blur
                saturation: 1.3,          // More vibrant colors
                time: 0.0,
            };
            
            ctx.apply_uniforms(UniformsSource::table(&uniforms));
            ctx.draw(0, 6, 1);
            ctx.end_render_pass();
            
            // Swap for next effect
            current_source = current_target_texture;
            use_a = !use_a;
            current_target_pass = if use_a { Some(self.pass_a) } else { Some(self.pass_b) };
            current_target_texture = if use_a { self.texture_a } else { self.texture_b };
        }

        // Apply motion blur if enabled
        if self.motion_blur_enabled {
            self.motion_blur.bindings.images[0] = current_source;
            
            ctx.begin_pass(current_target_pass, PassAction::Nothing);
            ctx.apply_pipeline(&self.motion_blur.pipeline);
            ctx.apply_bindings(&self.motion_blur.bindings);
            
            let uniforms = MotionBlurUniforms {
                screen_size: [self.screen_width as f32, self.screen_height as f32],
                velocity: self.motion_blur_velocity,
                blur_strength: 1.0,  // Strength is now built into the shader
                time: 0.0,
            };
            
            ctx.apply_uniforms(UniformsSource::table(&uniforms));
            ctx.draw(0, 6, 1);
            ctx.end_render_pass();
            
            current_source = current_target_texture;
        }

        // Blit final result to screen
        self.blit_bindings.images[0] = current_source;
        
        ctx.begin_default_pass(PassAction::Nothing);
        ctx.apply_pipeline(&self.blit_pipeline);
        ctx.apply_bindings(&self.blit_bindings);
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
    }

    pub fn toggle_all(&mut self) {
        self.effects_enabled = !self.effects_enabled;
        if self.effects_enabled {
            self.tilt_shift_enabled = true;
            self.motion_blur_enabled = true;
        } else {
            self.tilt_shift_enabled = false;
            self.motion_blur_enabled = false;
        }
        println!("All effects: {}", if self.effects_enabled { "ON" } else { "OFF" });
    }

    pub fn toggle_tilt_shift(&mut self) {
        self.tilt_shift_enabled = !self.tilt_shift_enabled;
        println!("Tilt shift: {}", if self.tilt_shift_enabled { "ON" } else { "OFF" });
    }

    pub fn toggle_motion_blur(&mut self) {
        self.motion_blur_enabled = !self.motion_blur_enabled;
        println!("Motion Blur: {}", if self.motion_blur_enabled { "ON" } else { "OFF" });
    }

    fn any_effect_enabled(&self) -> bool {
        self.tilt_shift_enabled || self.motion_blur_enabled
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;

        // Delete old resources
        ctx.delete_texture(self.scene_texture);
        ctx.delete_texture(self.scene_depth);
        ctx.delete_texture(self.texture_a);
        ctx.delete_texture(self.texture_b);
        ctx.delete_render_pass(self.scene_pass);
        ctx.delete_render_pass(self.pass_a);
        ctx.delete_render_pass(self.pass_b);

        // Recreate textures
        self.scene_texture = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                ..Default::default()
            },
        );

        self.scene_depth = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::Depth,
                width,
                height,
                ..Default::default()
            },
        );

        self.texture_a = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                ..Default::default()
            },
        );

        self.texture_b = ctx.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams {
                format: TextureFormat::RGBA8,
                width,
                height,
                ..Default::default()
            },
        );

        // Recreate render passes
        self.scene_pass = ctx.new_render_pass(self.scene_texture, Some(self.scene_depth));
        self.pass_a = ctx.new_render_pass(self.texture_a, None);
        self.pass_b = ctx.new_render_pass(self.texture_b, None);
    }

    pub fn key_down_event(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::T => self.toggle_all(),
            KeyCode::Y => self.toggle_tilt_shift(),
            KeyCode::U => self.toggle_motion_blur(),
            _ => {}
        }
    }

    pub fn update_motion_blur_velocity(&mut self, velocity: [f32; 2]) {
        self.motion_blur_velocity = velocity;
    }
}

#[repr(C)]
struct TiltShiftUniforms {
    screen_size: [f32; 2],
    focus_position: f32,
    focus_scale: f32,
    blur_amount: f32,
    saturation: f32,
    time: f32,
}

#[repr(C)]
struct MotionBlurUniforms {
    screen_size: [f32; 2],
    velocity: [f32; 2],
    blur_strength: f32,
    time: f32,
}