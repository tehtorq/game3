use miniquad::*;
use glam::Vec3;

mod math;
mod vertex;
mod renderer;
mod terrain;
mod terrain_instanced;
mod player;
mod enemy;
mod bullet;
mod particle;
mod camera;
mod shader;
mod game;

use vertex::Vertex;
use renderer::{Renderer, RenderMode, Drawable};
use camera::Camera;
use game::Game;
use terrain_instanced::InstancedTerrain;

const SCREEN_WIDTH: f32 = 1600.0;
const SCREEN_HEIGHT: f32 = 1200.0;

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    line_pipeline: Pipeline,
    triangle_pipeline: Pipeline,
    terrain_pipeline: Pipeline,
    bindings: Bindings,
    terrain_bindings: Bindings,
    game: Game,
    camera: Camera,
    input: InputState,
    paused: bool,
    instanced_terrain: Option<InstancedTerrain>,
}

#[derive(Default)]
struct InputState {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    shoot: bool,
}

impl Stage {
    fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();
        
        let shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader::VERTEX,
                fragment: shader::FRAGMENT,
            },
            shader::meta()
        ).unwrap();
        
        let terrain_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader::VERTEX_INSTANCED_TERRAIN,
                fragment: shader::FRAGMENT,
            },
            shader::meta_terrain()
        ).unwrap();
        
        let line_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 24, // 6 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
            ],
            shader.clone(),
            PipelineParams {
                primitive_type: PrimitiveType::Lines,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );
        
        let triangle_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 24, // 6 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
            ],
            shader.clone(),
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing, // Disable culling for debugging
                ..Default::default()
            },
        );
        
        // Create terrain pipeline with instancing
        let terrain_pipeline = ctx.new_pipeline(
            &[
                BufferLayout {
                    step_func: VertexStep::PerVertex,
                    stride: 24, // 6 floats * 4 bytes
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    stride: 8, // 2 floats * 4 bytes for instance offset
                    ..Default::default()
                }
            ],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
                VertexAttribute::with_buffer("instance_offset", VertexFormat::Float2, 1),
            ],
            terrain_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing, // Disable culling for debugging
                ..Default::default()
            },
        );

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Vertex>(2000000)
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<u32>(4000000)
        );
        
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };
        
        // Create dummy terrain bindings - will be filled when terrain is created
        let dummy_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<u32>(1)
        );
        let terrain_bindings = Bindings {
            vertex_buffers: vec![],
            index_buffer: dummy_buffer,
            images: vec![],
        };

        Self {
            ctx,
            line_pipeline,
            triangle_pipeline,
            terrain_pipeline,
            bindings,
            terrain_bindings,
            game: Game::new(),
            camera: Camera::new(),
            input: InputState::default(),
            paused: false,
            instanced_terrain: None,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        if !self.paused {
            let dt = 1.0 / 60.0;
            self.game.update(
                self.input.left,
                self.input.right,
                self.input.up,
                self.input.down,
                self.input.shoot,
                dt
            );
        }
    }

    fn draw(&mut self) {
        // Initialize instanced terrain on first draw
        if self.instanced_terrain.is_none() {
            println!("Initializing instanced terrain...");
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            unsafe {
                let terrain = InstancedTerrain::new(&mut *ctx_ptr, 5); // Smaller for debugging
                
                // Create terrain bindings
                self.terrain_bindings = Bindings {
                    vertex_buffers: vec![terrain.base_vertex_buffer(), terrain.instance_buffer()],
                    index_buffer: terrain.index_buffer(),
                    images: vec![terrain.height_texture()],
                };
                
                println!("Terrain bindings created with {} vertex buffers", self.terrain_bindings.vertex_buffers.len());
                
                self.instanced_terrain = Some(terrain);
            }
        }
        
        // Set up view and projection matrices
        let (width, height) = window::screen_size();
        let aspect = width / height;
        let proj = self.camera.get_projection_matrix(aspect);
        let view = self.camera.get_view_matrix(&self.game.player);
        let mvp = proj * view;
        
        // Render
        self.ctx.begin_default_pass(PassAction::Clear {
            color: Some((0.0, 0.0, 0.0, 1.0)),
            depth: Some(1.0),
            stencil: None,
        });
        
        // Draw instanced terrain
        if let Some(terrain) = &self.instanced_terrain {
            println!("Drawing terrain with {} instances, {} indices per instance", terrain.instance_count(), terrain.index_count());
            
            self.ctx.apply_pipeline(&self.terrain_pipeline);
            self.ctx.apply_bindings(&self.terrain_bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::UniformsTerrain::new(
                mvp, 
                [0.0, 1.0, 0.0], // Bright green for visibility
                terrain.terrain_scale(),
                0.0 // terrain_y_base - start at Y=0
            )));
            
            // Draw all terrain instances
            self.ctx.draw(0, terrain.index_count(), terrain.instance_count());
        }
        
        // Draw enemies in red
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            for enemy in &self.game.enemies {
                enemy.draw(&mut renderer);
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.0, 0.0]))); // Red
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw player in cyan
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            self.game.player.draw(&mut renderer);
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [0.0, 1.0, 1.0]))); // Cyan
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw lines (bullets, particles, debug) in yellow
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            // Draw debug cross
            renderer.draw_line(Vec3::new(-100.0, 0.0, 0.0), Vec3::new(100.0, 0.0, 0.0));
            renderer.draw_line(Vec3::new(0.0, -100.0, 0.0), Vec3::new(0.0, 100.0, 0.0));
            renderer.draw_line(Vec3::new(0.0, 0.0, -100.0), Vec3::new(0.0, 0.0, 100.0));
            
            // Draw bullets and particles
            for bullet in &self.game.bullets {
                bullet.draw(&mut renderer);
            }
            for particle in &self.game.particles {
                particle.draw(&mut renderer);
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 0.0]))); // Yellow
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Left | KeyCode::A => self.input.left = true,
            KeyCode::Right | KeyCode::D => self.input.right = true,
            KeyCode::Up | KeyCode::W => self.input.up = true,
            KeyCode::Down | KeyCode::S => self.input.down = true,
            KeyCode::Space => self.input.shoot = true,
            KeyCode::Escape => window::request_quit(),
            KeyCode::P => self.paused = !self.paused,
            _ => {}
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _mods: KeyMods) {
        match keycode {
            KeyCode::Left | KeyCode::A => self.input.left = false,
            KeyCode::Right | KeyCode::D => self.input.right = false,
            KeyCode::Up | KeyCode::W => self.input.up = false,
            KeyCode::Down | KeyCode::S => self.input.down = false,
            KeyCode::Space => self.input.shoot = false,
            _ => {}
        }
    }
}

fn main() {
    miniquad::start(
        conf::Conf {
            window_title: "Vector Shooter 3D".to_string(),
            window_width: SCREEN_WIDTH as i32,
            window_height: SCREEN_HEIGHT as i32,
            ..Default::default()
        },
        || Box::new(Stage::new()),
    );
}