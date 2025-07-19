use miniquad::*;
use crate::game::Game;
use crate::camera::Camera;
use crate::hud::HUD;
use crate::sounds::SoundSystem;
use crate::bullet_instanced::BulletInstancingSystem;
use crate::terrain;
use crate::input::InputState;

pub struct AppState {
    pub ctx: Box<dyn RenderingBackend>,
    pub game: Game,
    pub camera: Camera,
    pub hud: HUD,
    pub sound_system: SoundSystem,
    pub bullet_instance_system: BulletInstancingSystem,
    
    // Timing
    pub start_time: f64,
    pub frame_count: u32,
    pub fps_timer: f64,
    pub last_frame_time: f64,
    
    // Window state
    pub fullscreen: bool,
    pub mouse_captured: bool,
    pub paused: bool,
    
    // Input
    pub input: InputState,
    
    // Terrain
    pub terrain_gpu_rings: Option<terrain::TerrainGPURings>,
    pub terrain_gpu_complete: Option<terrain::TerrainGPUComplete>,
}

impl AppState {
    pub fn new(mut ctx: Box<dyn RenderingBackend>) -> Self {
        let game = Game::new();
        let camera = Camera::new();
        let hud = HUD::new();
        let mut sound_system = SoundSystem::new();
        sound_system.init();
        
        let bullet_instance_system = BulletInstancingSystem::new(&mut *ctx, 1000);
        
        Self {
            ctx,
            game,
            camera,
            hud,
            sound_system,
            bullet_instance_system,
            start_time: miniquad::date::now(),
            frame_count: 0,
            fps_timer: 0.0,
            last_frame_time: miniquad::date::now(),
            fullscreen: false,
            mouse_captured: false,
            paused: false,
            input: InputState::new(),
            terrain_gpu_rings: None,
            terrain_gpu_complete: None,
        }
    }
    
    pub fn update_fps(&mut self, current_time: f64) {
        self.frame_count += 1;
        let delta = current_time - self.last_frame_time;
        self.fps_timer += delta;
        
        if self.fps_timer >= 1.0 {
            println!("FPS: {}", self.frame_count);
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }
        
        self.last_frame_time = current_time;
    }
    
    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
        window::set_fullscreen(self.fullscreen);
    }
    
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    pub fn capture_mouse(&mut self) {
        self.mouse_captured = true;
        window::show_mouse(false);
        window::set_cursor_grab(true);
    }
    
    pub fn release_mouse(&mut self) {
        self.mouse_captured = false;
        window::show_mouse(true);
        window::set_cursor_grab(false);
    }
}