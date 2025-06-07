use miniquad::*;

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
use renderer::Renderer;
use camera::Camera;
use game::Game;

const SCREEN_WIDTH: f32 = 1600.0;
const SCREEN_HEIGHT: f32 = 1200.0;

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
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
        
        let pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 12, // 3 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
            ],
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Lines,
                depth_test: Comparison::Always,
                depth_write: false,
                ..Default::default()
            },
        );

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Vertex>(500000)
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<u16>(1000000)
        );
        
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        Self {
            ctx,
            pipeline,
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
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Draw the game
        let mut renderer = Renderer::new(&mut vertices, &mut indices);
        self.game.draw(&mut renderer);
        
        // Set up view and projection matrices
        let (width, height) = window::screen_size();
        let aspect = width / height;
        
        // Use proper projection and view matrices
        let proj = self.camera.get_projection_matrix(aspect);
        let view = self.camera.get_view_matrix(&self.game.player);
        let mvp = proj * view;
        
        // Render
        self.ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));

            self.ctx.apply_pipeline(&self.pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp)));
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