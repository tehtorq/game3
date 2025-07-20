use miniquad::*;
use glam::{Vec3, Mat4};
use std::f32::consts::PI;
use std::collections::HashMap;

mod math;
mod vertex;
mod renderer;
mod terrain;
mod biome;
mod player;
mod enemy;
mod enemies;
mod bullet;
mod bullet_instanced;
mod particle;
mod mine;
mod crystal;
mod hud;
mod base;
mod camera;
mod shader;
mod shaders;
mod shader_volumetric_laser;
mod game;
mod constants;
mod sounds;
mod skybox;
mod tree;
mod tree_instanced;
mod tree_procedural;

use vertex::Vertex;
use renderer::{Renderer, RenderMode, Drawable};
use bullet::BulletType;
use bullet_instanced::BulletInstancingSystem;
use enemy::{EnemyType, AlertState};
use hud::HUD;
use camera::Camera;
use game::Game;
use constants::*;
use sounds::{SoundSystem, MusicState};


// Default screen dimensions are now dynamically calculated at 75% of monitor size

// Removed TerrainMode enum - only using GPUComplete now

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    line_pipeline: Pipeline,
    triangle_pipeline: Pipeline,
    bullet_pipeline: Pipeline,
    bullet_glow_pipeline: Pipeline,
    volumetric_laser_pipeline: Pipeline,
    bindings: Bindings,
    bullet_bindings: Bindings,
    game: Game,
    camera: Camera,
    input: InputState,
    paused: bool,
    terrain_gpu_complete: Option<terrain::TerrainGPUComplete>,
    terrain_simple_pipeline: Pipeline,
    hud: HUD,
    // FPS tracking fields
    frame_count: u32,
    fps_timer: f64,
    last_frame_time: f64,
    // Fullscreen state
    fullscreen: bool,
    // Mouse capture state
    mouse_captured: bool,
    // Sound system
    sound_system: SoundSystem,
    // Bullet instancing
    bullet_instance_system: BulletInstancingSystem,
    // Start time for animations
    start_time: f64,
    // Skybox
    skybox: skybox::Skybox,
    // Tree instancing
    tree_instance_system: tree_instanced::TreeInstancingSystem,
    tree_pipeline: Pipeline,
    tree_bindings: HashMap<tree::TreeType, Bindings>,
}


#[derive(Default)]
struct InputState {
    left: bool,
    right: bool,
    forward: bool,
    backward: bool,
    shoot: bool,
    boost: bool,
    up: bool,
    down: bool,  // Move down (opposite of up/space)
    mouse_target_x: f32,  // Mouse position relative to center (-1 to 1)
    mouse_target_y: f32,  // Mouse position relative to center (-1 to 1)
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
        
        // Create simple terrain shader for CPU-based chunks
        let terrain_simple_shader = match ctx.new_shader(
            ShaderSource::Glsl {
                vertex: &*shader::VERTEX_TERRAIN_SIMPLE,
                fragment: &*shader::FRAGMENT_TERRAIN_SIMPLE,
            },
            shader::meta_terrain_simple()
        ) {
            Ok(shader) => shader,
            Err(e) => {
                eprintln!("Failed to create terrain shader: {:?}", e);
                eprintln!("This usually means there's a mismatch between shader code and uniforms.");
                panic!("Shader compilation failed");
            }
        };
        
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
        
        // Create simple terrain pipeline for CPU chunks
        let terrain_simple_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 24, // 6 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
            ],
            terrain_simple_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing,
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
                )),
                ..Default::default()
            },
        );
        
        // Create bullet shader and pipeline
        let bullet_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader::VERTEX_INSTANCED_BULLET,
                fragment: shader::FRAGMENT,
            },
            shader::meta()
        ).expect("Failed to create bullet shader");
        
        let bullet_pipeline = ctx.new_pipeline(
            &[
                BufferLayout {
                    step_func: VertexStep::PerVertex,
                    stride: 24, // 6 floats * 4 bytes
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    stride: 16, // 4 floats * 4 bytes (3 for position + 1 for scale)
                    ..Default::default()
                }
            ],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
                VertexAttribute::with_buffer("instance_position", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("instance_scale", VertexFormat::Float1, 1),
            ],
            bullet_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Nothing,
                ..Default::default()
            },
        );
        
        // Create bullet glow shader and pipeline
        let bullet_glow_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader::VERTEX_INSTANCED_BULLET,
                fragment: shader::FRAGMENT_GLOW,
            },
            shader::meta()
        ).expect("Failed to create bullet glow shader");
        
        // Create volumetric laser shader and pipeline
        let volumetric_laser_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: shader_volumetric_laser::VERTEX_VOLUMETRIC_LASER,
                fragment: shader_volumetric_laser::FRAGMENT_VOLUMETRIC_LASER,
            },
            shader::meta_volumetric_laser()
        ).expect("Failed to create volumetric laser shader");
        
        let volumetric_laser_pipeline = ctx.new_pipeline(
            &[BufferLayout {
                step_func: VertexStep::PerVertex,
                stride: 24, // 6 floats * 4 bytes
                ..Default::default()
            }],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
            ],
            volumetric_laser_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: false, // Don't write depth for volumetric effect
                cull_face: CullFace::Nothing,
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::Zero  // Alpha blending with source alpha
                )),
                ..Default::default()
            },
        );
        
        let bullet_glow_pipeline = ctx.new_pipeline(
            &[
                BufferLayout {
                    step_func: VertexStep::PerVertex,
                    stride: 24, // 6 floats * 4 bytes
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    stride: 16, // 4 floats * 4 bytes (3 for position + 1 for scale)
                    ..Default::default()
                }
            ],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
                VertexAttribute::with_buffer("instance_position", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("instance_scale", VertexFormat::Float1, 1),
            ],
            bullet_glow_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: false, // Don't write depth for glow
                cull_face: CullFace::Nothing,
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::One  // Additive blending
                )),
                ..Default::default()
            },
        );

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<Vertex>(MAX_VERTICES)
        );
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Stream,
            BufferSource::empty::<u32>(MAX_INDICES)
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
        
        // Create bullet instance system with a reasonable max bullet count
        let ctx_ptr = &mut *ctx as *mut dyn RenderingBackend;
        let bullet_instance_system = unsafe {
            BulletInstancingSystem::new(&mut *ctx_ptr, 10000)
        };
        
        // Create bullet bindings
        let bullet_bindings = Bindings {
            vertex_buffers: vec![
                bullet_instance_system.vertex_buffer(),
                bullet_instance_system.instance_buffer()
            ],
            index_buffer: bullet_instance_system.index_buffer(),
            images: vec![],
        };
        
        // Create tree shader and pipeline
        let tree_shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: crate::shaders::modules::tree::VERTEX_INSTANCED,
                fragment: crate::shaders::modules::tree::FRAGMENT_INSTANCED,
            },
            crate::shaders::modules::tree::meta()
        ).expect("Failed to create tree shader");
        
        let tree_pipeline = ctx.new_pipeline(
            &[
                BufferLayout {
                    step_func: VertexStep::PerVertex,
                    stride: 24, // 6 floats * 4 bytes
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    stride: 28, // 7 floats * 4 bytes (3 pos + 3 scale + 1 rotation)
                    ..Default::default()
                }
            ],
            &[
                VertexAttribute::new("pos", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
                VertexAttribute::with_buffer("instance_position", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("instance_scale_height", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("instance_rotation", VertexFormat::Float1, 1),
            ],
            tree_shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                cull_face: CullFace::Back,
                ..Default::default()
            },
        );
        
        // Create tree instance system
        let ctx_ptr = &mut *ctx as *mut dyn RenderingBackend;
        let tree_instance_system = unsafe {
            tree_instanced::TreeInstancingSystem::new(&mut *ctx_ptr, 2000) // max 2000 trees per type
        };
        
        // Create tree bindings for each tree type
        let mut tree_bindings = HashMap::new();
        let tree_types = [
            tree::TreeType::Pine,
            tree::TreeType::Oak,
            tree::TreeType::Palm,
            tree::TreeType::Crystal,
            tree::TreeType::Cactus,
            tree::TreeType::Mushroom,
            tree::TreeType::Dead,
        ];
        
        for tree_type in &tree_types {
            if let Some((vertex_buffer, instance_buffer, index_buffer, _)) = 
                tree_instance_system.get_buffers(*tree_type) {
                tree_bindings.insert(*tree_type, Bindings {
                    vertex_buffers: vec![vertex_buffer, instance_buffer],
                    index_buffer,
                    images: vec![],
                });
            }
        }

        // Create GPUComplete terrain on startup
        let ctx_ptr = &mut *ctx as *mut dyn RenderingBackend;
        let mut terrain_gpu_complete = unsafe {
            terrain::TerrainGPUComplete::new(&mut *ctx_ptr)
        };
        
        // Load or generate textures for the complete terrain
        let ctx_ptr = &mut *ctx as *mut dyn RenderingBackend;
        unsafe {
            terrain_gpu_complete.load_or_generate_textures(&mut *ctx_ptr, Some("awesome_terrain"));
        }
        
        // GPU terrain alternatives
        let terrain_gpu_complete = Some(terrain_gpu_complete);
        
        // Create skybox before moving ctx
        let skybox = skybox::Skybox::new(&mut *ctx);
        
        let stage = Self {
            ctx,
            line_pipeline,
            triangle_pipeline,
            terrain_simple_pipeline,
            bullet_pipeline,
            bullet_glow_pipeline,
            volumetric_laser_pipeline,
            bindings,
            bullet_bindings,
            game: Game::new(),
            camera: Camera::new(),
            input: InputState::default(),
            paused: false,
            terrain_gpu_complete,
            hud: HUD::new(),
            frame_count: 0,
            fps_timer: 0.0,
            last_frame_time: miniquad::date::now(),
            fullscreen: true,
            mouse_captured: true,
            sound_system: SoundSystem::new(),
            bullet_instance_system,
            start_time: miniquad::date::now(),
            skybox,
            tree_instance_system,
            tree_pipeline,
            tree_bindings,
        };
        
        // Start in fullscreen with mouse captured
        let (screen_width, screen_height) = window::screen_size();
        window::set_window_size(screen_width as u32, screen_height as u32);
        window::set_fullscreen(true);
        window::show_mouse(false);
        
        println!("\n=== Terrain Rendering System ===");
        println!("GPUComplete terrain mode active - Full GPU terrain with biomes");
        println!("================================\n");
        
        stage
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        if !self.paused {
            let dt = 1.0 / 60.0;
            
            // Initialize sound system if needed
            self.sound_system.init();
            
            // Update sound system
            self.sound_system.update(dt);
            
            // Play shooting sound
            if self.input.shoot && self.game.shoot_cooldown <= 0.0 {
                self.sound_system.play_laser();
            }
            
            // Store enemy count before update
            let enemies_before = self.game.enemies.len();
            
            // Apply mouse aim to player
            self.game.player.apply_mouse_aim(self.input.mouse_target_x, self.input.mouse_target_y, dt);
            
            self.game.update(
                self.input.left,
                self.input.right,
                self.input.forward,
                self.input.backward,
                self.input.shoot,
                self.input.boost,
                self.input.up,
                self.input.down,
                dt,
                &mut self.sound_system
            );
            
            // Update trees based on player position
            self.game.update_trees();
            
            // Play explosion sounds for destroyed enemies
            let enemies_after = self.game.enemies.len();
            if enemies_after < enemies_before {
                self.sound_system.play_explosion();
            }
            
            // Determine music state based on enemy proximity
            let mut music_state = MusicState::Peaceful;
            let mut closest_enemy_dist = f32::MAX;
            
            for enemy in &self.game.enemies {
                let dist = (enemy.pos - self.game.player.pos).length();
                if dist < closest_enemy_dist {
                    closest_enemy_dist = dist;
                }
                
                // Check if enemy is actively attacking
                if matches!(enemy.alert_state, AlertState::Alert) && dist < 1000.0 {
                    music_state = MusicState::Combat;
                    break;
                }
            }
            
            // Set music state based on distance if not in combat
            if music_state != MusicState::Combat && closest_enemy_dist < 2000.0 {
                music_state = MusicState::Suspense;
            }
            
            self.sound_system.update_music(music_state, dt);
            
            // Update thruster sound based on player thrust
            let is_thrusting = self.game.player.thrust.length() > 100.0; // Threshold for "significant" thrust
            self.sound_system.update_thruster(is_thrusting);
            
            // Update camera with smoothing
            self.camera.update(&self.game.player, dt);
        }
    }

    fn draw(&mut self) {
        // FPS calculation
        let current_time = miniquad::date::now();
        let delta_time = current_time - self.last_frame_time;
        self.last_frame_time = current_time;
        
        let frame_start = miniquad::date::now();
        
        self.frame_count += 1;
        self.fps_timer += delta_time;
        
        // Print FPS once per second
        if self.fps_timer >= 1.0 && SHOW_FPS {
            let fps = self.frame_count as f64 / self.fps_timer;
            println!("FPS: {:.1}", fps);
            println!("Enemies: {}", self.game.enemies.len());
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }
        
        // GPU terrain doesn't need position updates
        
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
        
        // Draw skybox first (behind everything)
        {
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            unsafe {
                self.skybox.draw(&mut *ctx_ptr, mvp, self.game.player.pos);
            }
        }
        
        // Draw terrain
        let elapsed_time = (current_time - self.start_time) as f32;
        let terrain_triangles = if let Some(ref terrain) = self.terrain_gpu_complete {
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            unsafe {
                terrain.draw(&mut *ctx_ptr, &self.terrain_simple_pipeline, mvp, [0.0, 1.0, 0.0], self.game.player.pos, elapsed_time)
            }
        } else {
            0
        };
        
        // Log terrain performance periodically
        if self.frame_count % 300 == 0 && terrain_triangles > 0 {
            println!("Terrain triangles: {}", terrain_triangles);
        }
        
        // Draw trees using instancing (after terrain, before other objects)
        {
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            let instance_counts = unsafe {
                self.tree_instance_system.update_instances(&mut *ctx_ptr, &self.game.trees)
            };
            
            // Define colors for each tree type
            let tree_colors = [
                (tree::TreeType::Pine, [0.1, 0.4, 0.1]),       // Dark green
                (tree::TreeType::Oak, [0.2, 0.5, 0.1]),        // Green
                (tree::TreeType::Palm, [0.2, 0.6, 0.2]),       // Bright green
                (tree::TreeType::Crystal, [0.6, 0.3, 1.0]),    // Purple
                (tree::TreeType::Cactus, [0.1, 0.5, 0.1]),     // Green
                (tree::TreeType::Mushroom, [0.6, 0.1, 0.1]),   // Red
                (tree::TreeType::Dead, [0.3, 0.2, 0.1]),       // Dark brown
            ];
            
            // Log tree instance counts periodically
            if self.frame_count % 300 == 0 && !instance_counts.is_empty() {
                println!("Tree instances: {:?}", instance_counts);
            }
            
            // Render each tree type with its instances
            for (tree_type, color) in &tree_colors {
                if let Some(&count) = instance_counts.get(tree_type) {
                    if count > 0 {
                        if let Some(bindings) = self.tree_bindings.get(tree_type) {
                            self.ctx.apply_pipeline(&self.tree_pipeline);
                            self.ctx.apply_bindings(bindings);
                            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, *color)));
                            
                            if let Some((_, _, _, index_count)) = 
                                self.tree_instance_system.get_buffers(*tree_type) {
                                self.ctx.draw(0, index_count, count);
                            }
                        }
                    }
                }
            }
        }
        
        // Draw bases first (in red/orange)
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            for base in &self.game.bases {
                if base.is_active {
                    base.draw(&mut renderer);
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // Red-orange color for bases
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.3, 0.1])));
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw enemies with different colors per type
        vertices.clear();
        indices.clear();
        
        // Group enemies by type for batch rendering with different colors
        let enemy_colors = [
            (EnemyType::Cube, [0.0, 1.0, 0.0]),       // Green - basic
            (EnemyType::Pyramid, [1.0, 0.0, 0.0]),    // Red - aggressive
            (EnemyType::Spinner, [1.0, 0.5, 0.0]),    // Orange - orbital
            (EnemyType::Hunter, [1.0, 0.0, 1.0]),     // Magenta - dangerous
            (EnemyType::Guardian, [0.5, 0.5, 0.5]),   // Gray - defensive
            (EnemyType::Laser, [0.0, 0.5, 1.0]),      // Blue - laser enemy
            (EnemyType::Swarm, [1.0, 1.0, 0.0]),      // Yellow - small/fast
            (EnemyType::Phaser, [0.8, 0.0, 1.0]),     // Purple - teleporter
            (EnemyType::Shield, [0.0, 1.0, 1.0]),     // Cyan - support
            (EnemyType::Bomber, [0.8, 0.2, 0.2]),     // Dark red - explosive
            (EnemyType::Disruptor, [0.2, 1.0, 0.2]),  // Electric green - debuff
            (EnemyType::Carrier, [0.3, 0.3, 0.3]),    // Dark gray - spawner
            (EnemyType::Reflector, [0.9, 0.9, 0.9]),  // Silver - mirror
            (EnemyType::Vortex, [0.5, 0.0, 0.8]),     // Deep purple - gravity
        ];
        
        for (enemy_type, color) in &enemy_colors {
            vertices.clear();
            indices.clear();
            
            {
                let mut renderer = Renderer::new(&mut vertices, &mut indices);
                renderer.set_mode(RenderMode::Triangles);
                
                for enemy in &self.game.enemies {
                    if enemy.enemy_type == *enemy_type {
                        // Modify color based on alert state
                        let _alert_color = match enemy.alert_state {
                            AlertState::Alert => {
                                // Flash red when actively attacking
                                let elapsed = (miniquad::date::now() - self.start_time) as f32;
                                let flash = (elapsed * 5.0).sin() * 0.5 + 0.5;
                                [color[0] + flash * (1.0 - color[0]), 
                                 color[1] * (1.0 - flash * 0.5), 
                                 color[2] * (1.0 - flash * 0.5)]
                            }
                            AlertState::Suspicious => {
                                // Yellow tint when suspicious
                                [color[0] + 0.3, color[1] + 0.3, color[2]]
                            }
                            AlertState::Searching => {
                                // Orange tint when searching
                                [color[0] + 0.5, color[1] + 0.2, color[2]]
                            }
                            _ => *color, // Normal color
                        };
                        
                        // Store current color for this batch
                        if enemy.alert_state != AlertState::Unaware && enemy.alert_state != AlertState::Returning {
                            // We'll need to draw this enemy separately with its alert color
                            // For now, just draw normally - we'll handle alert indicators separately
                        }
                        enemy.draw(&mut renderer);
                    }
                }
            }
            
            if !vertices.is_empty() {
                self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
                self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
                self.ctx.apply_pipeline(&self.triangle_pipeline);
                self.ctx.apply_bindings(&self.bindings);
                self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, *color)));
                self.ctx.draw(0, indices.len() as i32, 1);
            }
        }
        
        // Draw alert indicators for enemies
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            for enemy in &self.game.enemies {
                match enemy.alert_state {
                    AlertState::Alert => {
                        // Draw exclamation mark above enemy
                        let base_pos = enemy.pos + Vec3::new(0.0, 40.0, 0.0);
                        renderer.draw_line(base_pos, base_pos + Vec3::new(0.0, 15.0, 0.0));
                        renderer.draw_line(
                            base_pos + Vec3::new(0.0, 20.0, 0.0),
                            base_pos + Vec3::new(0.0, 22.0, 0.0)
                        );
                    }
                    AlertState::Suspicious => {
                        // Draw question mark above enemy
                        let base_pos = enemy.pos + Vec3::new(0.0, 40.0, 0.0);
                        let segments = 8;
                        for i in 0..segments {
                            let angle1 = i as f32 * PI / segments as f32;
                            let angle2 = (i + 1) as f32 * PI / segments as f32;
                            let p1 = base_pos + Vec3::new(angle1.cos() * 5.0, angle1.sin() * 5.0 + 15.0, 0.0);
                            let p2 = base_pos + Vec3::new(angle2.cos() * 5.0, angle2.sin() * 5.0 + 15.0, 0.0);
                            renderer.draw_line(p1, p2);
                        }
                        renderer.draw_line(
                            base_pos + Vec3::new(0.0, 0.0, 0.0),
                            base_pos + Vec3::new(0.0, 2.0, 0.0)
                        );
                    }
                    AlertState::Searching => {
                        // Draw rotating search lines
                        let base_pos = enemy.pos + Vec3::new(0.0, 30.0, 0.0);
                        let elapsed = (miniquad::date::now() - self.start_time) as f32;
                        let search_angle = elapsed * 2.0;
                        for i in 0..3 {
                            let angle = search_angle + i as f32 * PI * 2.0 / 3.0;
                            let end_pos = base_pos + Vec3::new(angle.cos() * 15.0, 0.0, angle.sin() * 15.0);
                            renderer.draw_line(base_pos, end_pos);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // White color for alert indicators
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 1.0])));
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw player contrails first (so they appear behind the ship)
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            // Draw trails as separate lines for each wing
            let trail_points = &self.game.player.trail_points;
            if trail_points.len() >= 4 {
                // Draw left wing trail (even indices)
                for i in (0..trail_points.len() - 2).step_by(2) {
                    let alpha = trail_points[i].lifetime / 0.6; // Max lifetime is 0.6
                    if alpha > 0.0 && i + 2 < trail_points.len() {
                        renderer.draw_line(trail_points[i].pos, trail_points[i + 2].pos);
                    }
                }
                
                // Draw right wing trail (odd indices)
                for i in (1..trail_points.len() - 2).step_by(2) {
                    let alpha = trail_points[i].lifetime / 0.6;
                    if alpha > 0.0 && i + 2 < trail_points.len() {
                        renderer.draw_line(trail_points[i].pos, trail_points[i + 2].pos);
                    }
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // Pure white color for trails
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 1.0])));
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
        
        // Draw player shield effect
        if self.game.player.shield > 0.0 {
            vertices.clear();
            indices.clear();
            
            {
                let mut renderer = Renderer::new(&mut vertices, &mut indices);
                renderer.set_mode(RenderMode::Lines);
                
                // Draw shield bubble around player
                let shield_radius = 30.0;
                let shield_segments = 16;
                
                // Draw horizontal rings
                for j in 0..3 {
                    let y_offset = (j as f32 - 1.0) * shield_radius * 0.5;
                    let ring_radius = (1.0 - (y_offset / shield_radius).abs()) * shield_radius;
                    
                    for i in 0..shield_segments {
                        let angle1 = i as f32 * PI * 2.0 / shield_segments as f32;
                        let angle2 = (i + 1) as f32 * PI * 2.0 / shield_segments as f32;
                        
                        let p1 = self.game.player.pos + Vec3::new(
                            angle1.cos() * ring_radius,
                            y_offset,
                            angle1.sin() * ring_radius
                        );
                        let p2 = self.game.player.pos + Vec3::new(
                            angle2.cos() * ring_radius,
                            y_offset,
                            angle2.sin() * ring_radius
                        );
                        
                        renderer.draw_line(p1, p2);
                    }
                }
                
                // Draw vertical lines
                for i in 0..8 {
                    let angle = i as f32 * PI * 2.0 / 8.0;
                    let x = angle.cos() * shield_radius;
                    let z = angle.sin() * shield_radius;
                    
                    renderer.draw_line(
                        self.game.player.pos + Vec3::new(x, -shield_radius * 0.5, z),
                        self.game.player.pos + Vec3::new(x, shield_radius * 0.5, z)
                    );
                }
            }
            
            if !vertices.is_empty() {
                self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
                self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
                self.ctx.apply_pipeline(&self.line_pipeline);
                self.ctx.apply_bindings(&self.bindings);
                // Blue-ish shield color with transparency effect based on shield strength
                let shield_color = [0.0, 0.5 * self.game.player.shield, 1.0 * self.game.player.shield];
                self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, shield_color)));
                self.ctx.draw(0, indices.len() as i32, 1);
            }
        }
        
        // Draw all bullets using instancing
        {
            // Update bullet instances
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            let instance_count = unsafe {
                self.bullet_instance_system.update_instances(&mut *ctx_ptr, &self.game.bullets)
            };
            
            if instance_count > 0 {
                // Draw player bullets in yellow
                let player_bullets: Vec<_> = self.game.bullets.iter()
                    .filter(|b| matches!(b.bullet_type, BulletType::Player))
                    .cloned()
                    .collect();
                
                let player_instance_count = unsafe {
                    self.bullet_instance_system.update_instances(&mut *ctx_ptr, &player_bullets)
                };
                
                if player_instance_count > 0 {
                    // First pass: Draw solid bullets
                    self.ctx.apply_pipeline(&self.bullet_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 0.0]))); // Yellow
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), player_instance_count);
                    
                    // Second pass: Draw glow
                    self.ctx.apply_pipeline(&self.bullet_glow_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 0.2]))); // Bright yellow glow
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), player_instance_count);
                }
                
                // Draw enemy bullets in orange
                let enemy_bullets: Vec<_> = self.game.bullets.iter()
                    .filter(|b| matches!(b.bullet_type, BulletType::Enemy))
                    .cloned()
                    .collect();
                
                let enemy_instance_count = unsafe {
                    self.bullet_instance_system.update_instances(&mut *ctx_ptr, &enemy_bullets)
                };
                
                if enemy_instance_count > 0 {
                    // First pass: Draw solid bullets
                    self.ctx.apply_pipeline(&self.bullet_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.5, 0.0]))); // Orange
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), enemy_instance_count);
                    
                    // Second pass: Draw glow
                    self.ctx.apply_pipeline(&self.bullet_glow_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.3, 0.0]))); // Orange glow
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), enemy_instance_count);
                }
                
                // Draw heavy turret bullets in white
                let turret_bullets: Vec<_> = self.game.bullets.iter()
                    .filter(|b| matches!(b.bullet_type, BulletType::HeavyTurret))
                    .cloned()
                    .collect();
                
                let turret_instance_count = unsafe {
                    self.bullet_instance_system.update_instances(&mut *ctx_ptr, &turret_bullets)
                };
                
                if turret_instance_count > 0 {
                    // First pass: Draw solid bullets
                    self.ctx.apply_pipeline(&self.bullet_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 1.0]))); // White
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), turret_instance_count);
                    
                    // Second pass: Draw glow
                    self.ctx.apply_pipeline(&self.bullet_glow_pipeline);
                    self.ctx.apply_bindings(&self.bullet_bindings);
                    self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [0.8, 0.8, 1.0]))); // Blueish-white glow
                    self.ctx.draw(0, self.bullet_instance_system.index_count(), turret_instance_count);
                }
            }
        }
        
        // Draw particles as triangles with different colors
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            // Draw particles
            for particle in &self.game.particles {
                particle.draw(&mut renderer);
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // Use bright colors for particles - they'll fade with alpha in the shader
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.8, 0.3]))); // Orange-yellow
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw debug cross using lines
        if SHOW_DEBUG_CROSS {
            vertices.clear();
            indices.clear();
            
            {
                let mut renderer = Renderer::new(&mut vertices, &mut indices);
                renderer.set_mode(RenderMode::Lines);
                
                renderer.draw_line(Vec3::new(-DEBUG_CROSS_SIZE, 0.0, 0.0), Vec3::new(DEBUG_CROSS_SIZE, 0.0, 0.0));
                renderer.draw_line(Vec3::new(0.0, -DEBUG_CROSS_SIZE, 0.0), Vec3::new(0.0, DEBUG_CROSS_SIZE, 0.0));
                renderer.draw_line(Vec3::new(0.0, 0.0, -DEBUG_CROSS_SIZE), Vec3::new(0.0, 0.0, DEBUG_CROSS_SIZE));
            }
            
            if !vertices.is_empty() {
                self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
                self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
                self.ctx.apply_pipeline(&self.line_pipeline);
                self.ctx.apply_bindings(&self.bindings);
                self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 1.0]))); // White
                self.ctx.draw(0, indices.len() as i32, 1);
            }
        }
        
        
        // Draw volumetric laser beams
        vertices.clear();
        indices.clear();
        
        // Process each laser beam enemy
        for enemy in &self.game.enemies {
            if let Some((laser_start, laser_end)) = enemy.get_laser_info() {
                vertices.clear();
                indices.clear();
                
                {
                    let mut renderer = Renderer::new(&mut vertices, &mut indices);
                    renderer.set_mode(RenderMode::Triangles);
                    
                    // Create a cylindrical volume around the laser beam
                    let laser_dir = (laser_end - laser_start).normalize();
                    let laser_length = (laser_end - laser_start).length();
                    
                    // Find perpendicular vectors
                    let up = if laser_dir.y.abs() > 0.9 {
                        Vec3::new(1.0, 0.0, 0.0)
                    } else {
                        Vec3::new(0.0, 1.0, 0.0)
                    };
                    let right = laser_dir.cross(up).normalize();
                    let up = right.cross(laser_dir).normalize();
                    
                    // Create a cylinder mesh with more segments for smoother ray marching
                    let segments = 24;
                    let radius = 60.0; // Larger radius for volumetric effect
                    
                    // Create vertices for cylinder
                    let mut cylinder_verts = Vec::new();
                    
                    // Start cap
                    for i in 0..segments {
                        let angle = i as f32 * PI * 2.0 / segments as f32;
                        let x = angle.cos() * radius;
                        let y = angle.sin() * radius;
                        let offset = right * x + up * y;
                        cylinder_verts.push(laser_start + offset);
                    }
                    
                    // End cap
                    for i in 0..segments {
                        let angle = i as f32 * PI * 2.0 / segments as f32;
                        let x = angle.cos() * radius;
                        let y = angle.sin() * radius;
                        let offset = right * x + up * y;
                        cylinder_verts.push(laser_end + offset);
                    }
                    
                    // Draw cylinder sides
                    for i in 0..segments {
                        let i1 = i;
                        let i2 = (i + 1) % segments;
                        let i3 = i + segments;
                        let i4 = ((i + 1) % segments) + segments;
                        
                        // First triangle
                        renderer.draw_triangle(
                            cylinder_verts[i1],
                            cylinder_verts[i2],
                            cylinder_verts[i3]
                        );
                        
                        // Second triangle
                        renderer.draw_triangle(
                            cylinder_verts[i2],
                            cylinder_verts[i4],
                            cylinder_verts[i3]
                        );
                    }
                    
                    // Draw caps
                    let center_start = laser_start;
                    let center_end = laser_end;
                    
                    for i in 0..segments {
                        let i1 = i;
                        let i2 = (i + 1) % segments;
                        
                        // Start cap
                        renderer.draw_triangle(
                            center_start,
                            cylinder_verts[i1],
                            cylinder_verts[i2]
                        );
                        
                        // End cap
                        renderer.draw_triangle(
                            center_end,
                            cylinder_verts[i2 + segments],
                            cylinder_verts[i1 + segments]
                        );
                    }
                }
                
                if !vertices.is_empty() {
                    self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
                    self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
                    self.ctx.apply_pipeline(&self.volumetric_laser_pipeline);
                    self.ctx.apply_bindings(&self.bindings);
                    
                    // Create uniforms for volumetric laser shader
                    #[repr(C)]
                    struct VolumetricLaserUniforms {
                        mvp: [[f32; 4]; 4],
                        color: [f32; 3],
                        _padding1: f32,
                        laser_start: [f32; 3],
                        _padding2: f32,
                        laser_end: [f32; 3],
                        _padding3: f32,
                        laser_radius: f32,
                        time: f32,
                        _padding4: [f32; 2],
                        camera_pos: [f32; 3],
                        _padding5: f32,
                    }
                    
                    let elapsed = (miniquad::date::now() - self.start_time) as f32;
                    let time = elapsed;
                    let uniforms = VolumetricLaserUniforms {
                        mvp: mvp.to_cols_array_2d(),
                        color: [1.0, 1.0, 0.0], // Yellow laser
                        _padding1: 0.0,
                        laser_start: [laser_start.x, laser_start.y, laser_start.z],
                        _padding2: 0.0,
                        laser_end: [laser_end.x, laser_end.y, laser_end.z],
                        _padding3: 0.0,
                        laser_radius: 20.0,
                        time,
                        _padding4: [0.0, 0.0],
                        camera_pos: [self.camera.get_position().x, self.camera.get_position().y, self.camera.get_position().z],
                        _padding5: 0.0,
                    };
                    
                    self.ctx.apply_uniforms(UniformsSource::table(&uniforms));
                    self.ctx.draw(0, indices.len() as i32, 1);
                }
            }
        }
        
        // Draw mines in red
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            for mine in &self.game.mines {
                mine.draw(&mut renderer);
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.2, 0.2]))); // Dark red
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw shield effects
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            // Draw shield bubbles
            for enemy in &self.game.enemies {
                if enemy.get_shield_active() {
                    let shield_radius = enemy.get_shield_radius();
                    // Draw shield sphere as lines
                    for i in 0..16 {
                        let angle1 = i as f32 * PI * 2.0 / 16.0;
                        let angle2 = (i + 1) as f32 * PI * 2.0 / 16.0;
                        
                        // Horizontal ring
                        renderer.draw_line(
                            enemy.pos + Vec3::new(angle1.cos() * shield_radius, 0.0, angle1.sin() * shield_radius),
                            enemy.pos + Vec3::new(angle2.cos() * shield_radius, 0.0, angle2.sin() * shield_radius)
                        );
                        
                        // Vertical rings
                        renderer.draw_line(
                            enemy.pos + Vec3::new(angle1.cos() * shield_radius, angle1.sin() * shield_radius, 0.0),
                            enemy.pos + Vec3::new(angle2.cos() * shield_radius, angle2.sin() * shield_radius, 0.0)
                        );
                        
                        renderer.draw_line(
                            enemy.pos + Vec3::new(0.0, angle1.cos() * shield_radius, angle1.sin() * shield_radius),
                            enemy.pos + Vec3::new(0.0, angle2.cos() * shield_radius, angle2.sin() * shield_radius)
                        );
                    }
                }
                
                // Draw disruptor waves
                if enemy.get_wave_active() {
                    let wave_radius = enemy.get_wave_radius();
                    if wave_radius > 0.0 {
                        for i in 0..32 {
                            let angle1 = i as f32 * PI * 2.0 / 32.0;
                            let angle2 = (i + 1) as f32 * PI * 2.0 / 32.0;
                            
                            renderer.draw_line(
                                enemy.pos + Vec3::new(angle1.cos() * wave_radius, 0.0, angle1.sin() * wave_radius),
                                enemy.pos + Vec3::new(angle2.cos() * wave_radius, 0.0, angle2.sin() * wave_radius)
                            );
                        }
                    }
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [0.0, 1.0, 1.0]))); // Cyan for shields
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw HUD (should be last to appear on top)
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            let (width, height) = window::screen_size();
            self.hud.draw(&mut renderer, &self.game, width, height);
            
            // Project crosshair into 3D space in front of ship, then back to screen
            // This makes the crosshair feel like it's attached to a point in space
            let aim_distance = 400.0; // Distance in front of ship
            
            // Get ship's forward direction with mouse influence
            let mouse_yaw_offset = self.input.mouse_target_x * 0.5; // How much mouse affects aim
            let mouse_pitch_offset = -self.input.mouse_target_y * 0.3; // Less vertical influence
            
            let aim_yaw = self.game.player.rotation + mouse_yaw_offset;
            let aim_pitch = self.game.player.pitch + mouse_pitch_offset;
            
            // Calculate 3D position of aim point
            let aim_forward = Vec3::new(
                -aim_yaw.sin() * aim_pitch.cos(),
                aim_pitch.sin(),
                -aim_yaw.cos() * aim_pitch.cos()
            );
            let aim_point_3d = self.game.player.pos + aim_forward * aim_distance;
            
            // Project this 3D point back to screen space
            let view_proj = proj * view;
            let aim_point_clip = view_proj * aim_point_3d.extend(1.0);
            
            // Convert to screen coordinates
            let crosshair_x = if aim_point_clip.w != 0.0 {
                (aim_point_clip.x / aim_point_clip.w) * aspect * 40.0
            } else {
                self.input.mouse_target_x * 30.0
            };
            
            let crosshair_y = if aim_point_clip.w != 0.0 {
                (aim_point_clip.y / aim_point_clip.w) * 40.0
            } else {
                -self.input.mouse_target_y * 30.0
            };
            
            let crosshair_size = 1.0;
            
            // Horizontal line
            renderer.draw_line(
                Vec3::new(crosshair_x - crosshair_size, crosshair_y, -1.0),
                Vec3::new(crosshair_x + crosshair_size, crosshair_y, -1.0)
            );
            // Vertical line
            renderer.draw_line(
                Vec3::new(crosshair_x, crosshair_y - crosshair_size, -1.0),
                Vec3::new(crosshair_x, crosshair_y + crosshair_size, -1.0)
            );
            // Center dot
            renderer.draw_line(
                Vec3::new(crosshair_x - 0.2, crosshair_y, -1.0),
                Vec3::new(crosshair_x + 0.2, crosshair_y, -1.0)
            );
        }
        
        if !vertices.is_empty() {
            // Create orthographic projection for HUD
            let hud_proj = Mat4::orthographic_rh_gl(-aspect * 40.0, aspect * 40.0, -40.0, 40.0, 0.1, 100.0);
            let hud_view = Mat4::IDENTITY;
            let hud_mvp = hud_proj * hud_view;
            
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(hud_mvp, [1.0, 1.0, 1.0]))); // White for HUD
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }

    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Left | KeyCode::A => self.input.left = true,
            KeyCode::Right | KeyCode::D => self.input.right = true,
            KeyCode::Up | KeyCode::W => self.input.forward = true,
            KeyCode::Down | KeyCode::S => self.input.backward = true,
            KeyCode::Space => self.input.up = true,
            KeyCode::C => self.input.down = true,
            KeyCode::LeftShift | KeyCode::RightShift => self.input.boost = true,
            KeyCode::Escape => window::request_quit(),
            KeyCode::P => self.paused = !self.paused,
            KeyCode::M => {
                self.mouse_captured = !self.mouse_captured;
                window::show_mouse(!self.mouse_captured);
            }
            KeyCode::F => {
                self.fullscreen = !self.fullscreen;
                if self.fullscreen {
                    // Enter fullscreen - get screen size and use it
                    let (screen_width, screen_height) = window::screen_size();
                    window::set_window_size(screen_width as u32, screen_height as u32);
                    window::set_fullscreen(true);
                } else {
                    // Exit fullscreen - restore window size
                    window::set_fullscreen(false);
                    window::set_window_size(1200, 900);
                }
            }
            KeyCode::Minus | KeyCode::KpSubtract => self.hud.zoom_out(),
            KeyCode::Equal | KeyCode::KpAdd => self.hud.zoom_in(),
            KeyCode::T => {
                println!("GPUComplete terrain mode is the only available mode");
            },
            _ => {}
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _mods: KeyMods) {
        match keycode {
            KeyCode::Left | KeyCode::A => self.input.left = false,
            KeyCode::Right | KeyCode::D => self.input.right = false,
            KeyCode::Up | KeyCode::W => self.input.forward = false,
            KeyCode::Down | KeyCode::S => self.input.backward = false,
            KeyCode::Space => self.input.up = false,
            KeyCode::C => self.input.down = false,
            KeyCode::LeftShift | KeyCode::RightShift => self.input.boost = false,
            _ => {}
        }
    }
    
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        // Calculate mouse position relative to screen center
        let (width, height) = window::screen_size();
        
        // Convert to -1 to 1 range where center is 0
        self.input.mouse_target_x = (x - width / 2.0) / (width / 2.0);
        self.input.mouse_target_y = (y - height / 2.0) / (height / 2.0);
        
        // Clamp to reasonable range
        self.input.mouse_target_x = self.input.mouse_target_x.clamp(-1.0, 1.0);
        self.input.mouse_target_y = self.input.mouse_target_y.clamp(-1.0, 1.0);
    }
    
    fn mouse_button_down_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        match button {
            MouseButton::Left => self.input.shoot = true,
            MouseButton::Right => self.input.forward = true,
            _ => {}
        }
    }
    
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        match button {
            MouseButton::Left => self.input.shoot = false,
            MouseButton::Right => self.input.forward = false,
            _ => {}
        }
    }
}

fn main() {
    // Set up panic hook for debugging
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("\n=== GAME CRASHED ===");
        if let Some(location) = panic_info.location() {
            eprintln!("Location: {}:{}", location.file(), location.line());
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Error: {}", s);
        }
        eprintln!("==================\n");
    }));
    
    // Note: Command line arguments for terrain loading are temporarily disabled
    // The new CPU-based terrain system generates terrain on-the-fly
    
    // Default window size
    let window_width = WINDOW_WIDTH;
    let window_height = WINDOW_HEIGHT;
    
    miniquad::start(
        conf::Conf {
            window_title: "Vector Shooter 3D".to_string(),
            window_width,
            window_height,
            ..Default::default()
        },
        || Box::new(Stage::new()),
    );
}