use miniquad::*;
use glam::Vec3;

mod math;
mod vertex;
mod renderer;
mod terrain;
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

const SCREEN_WIDTH: f32 = 1600.0;
const SCREEN_HEIGHT: f32 = 1200.0;

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    line_pipeline: Pipeline,
    triangle_pipeline: Pipeline,
    bindings: Bindings,
    game: Game,
    camera: Camera,
    input: InputState,
    paused: bool,
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
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Back,
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
            BufferSource::empty::<u16>(4000000)
        );
        
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        Self {
            ctx,
            line_pipeline,
            triangle_pipeline,
            bindings,
            game: Game::new(),
            camera: Camera::new(),
            input: InputState::default(),
            paused: false,
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
        
        // Draw terrain in green
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            let player_pos = self.game.player.pos;
            let max_draw_distance = 800.0;
            
            for chunk in &self.game.terrain_chunks {
                let chunk_center = Vec3::new(chunk.x_offset, 0.0, chunk.z_offset);
                let distance = (chunk_center - player_pos).length();
                if distance < max_draw_distance {
                    chunk.draw(&mut renderer);
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [0.0, 1.0, 0.0]))); // Green
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw enemies in red
        vertices.clear();
        indices.clear();
        
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