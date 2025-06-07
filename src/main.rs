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
                depth_test: Comparison::Always,
                depth_write: false,
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
                depth_test: Comparison::Always,
                depth_write: false,
                cull_face: CullFace::Nothing,
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
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
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

    fn draw(&mut self) {
        let mut line_vertices = Vec::new();
        let mut line_indices = Vec::new();
        let mut triangle_vertices = Vec::new();
        let mut triangle_indices = Vec::new();
        
        // Set up view and projection matrices
        let (width, height) = window::screen_size();
        let aspect = width / height;
        let proj = self.camera.get_projection_matrix(aspect);
        let view = self.camera.get_view_matrix(&self.game.player);
        let mvp = proj * view;
        
        // Collect terrain triangles
        let mut triangle_renderer = Renderer::new(&mut triangle_vertices, &mut triangle_indices);
        triangle_renderer.set_mode(RenderMode::Triangles);
        
        // Draw only terrain chunks with triangles
        let player_pos = self.game.player.pos;
        let max_draw_distance = 800.0;
        
        for chunk in &self.game.terrain_chunks {
            let chunk_center = Vec3::new(chunk.x_offset, 0.0, chunk.z_offset);
            let distance = (chunk_center - player_pos).length();
            if distance < max_draw_distance {
                chunk.draw(&mut triangle_renderer);
            }
        }
        
        // Collect everything else as lines
        let mut line_renderer = Renderer::new(&mut line_vertices, &mut line_indices);
        line_renderer.set_mode(RenderMode::Lines);
        
        // Draw debug cross
        line_renderer.draw_line(Vec3::new(-100.0, 0.0, 0.0), Vec3::new(100.0, 0.0, 0.0));
        line_renderer.draw_line(Vec3::new(0.0, -100.0, 0.0), Vec3::new(0.0, 100.0, 0.0));
        line_renderer.draw_line(Vec3::new(0.0, 0.0, -100.0), Vec3::new(0.0, 0.0, 100.0));
        
        // Draw game entities
        self.game.player.draw(&mut line_renderer);
        for enemy in &self.game.enemies {
            enemy.draw(&mut line_renderer);
        }
        for bullet in &self.game.bullets {
            bullet.draw(&mut line_renderer);
        }
        for particle in &self.game.particles {
            particle.draw(&mut line_renderer);
        }
        
        // Render
        self.ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        
        // Draw triangles
        if !triangle_vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&triangle_vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&triangle_indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp)));
            self.ctx.draw(0, triangle_indices.len() as i32, 1);
        }
        
        // Draw lines
        if !line_vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&line_vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&line_indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp)));
            self.ctx.draw(0, line_indices.len() as i32, 1);
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