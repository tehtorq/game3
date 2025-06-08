use miniquad::*;
use glam::{Vec3, Mat4};
use std::f32::consts::PI;

mod math;
mod vertex;
mod renderer;
mod terrain;
mod terrain_instanced;
mod player;
mod enemy;
mod bullet;
mod particle;
mod mine;
mod crystal;
mod hud;
mod base;
mod camera;
mod shader;
mod game;

use vertex::Vertex;
use renderer::{Renderer, RenderMode, Drawable};
use bullet::BulletType;
use enemy::{EnemyType, AlertState};
use hud::HUD;
use camera::Camera;
use game::Game;
use terrain_instanced::InstancedTerrain;

// Default screen dimensions are now dynamically calculated at 75% of monitor size

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
    hud: HUD,
    // FPS tracking fields
    frame_count: u32,
    fps_timer: f64,
    last_frame_time: f64,
    // Fullscreen state
    fullscreen: bool,
}

#[derive(Default)]
struct InputState {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    shoot: bool,
    boost: bool,
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
                fragment: shader::FRAGMENT_TERRAIN,
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

        let stage = Self {
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
            hud: HUD::new(),
            frame_count: 0,
            fps_timer: 0.0,
            last_frame_time: miniquad::date::now(),
            fullscreen: true,
        };
        
        // Start in fullscreen
        let (screen_width, screen_height) = window::screen_size();
        window::set_window_size(screen_width as u32, screen_height as u32);
        window::set_fullscreen(true);
        
        stage
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
                self.input.boost,
                dt
            );
        }
    }

    fn draw(&mut self) {
        // FPS calculation
        let current_time = miniquad::date::now();
        let delta_time = current_time - self.last_frame_time;
        self.last_frame_time = current_time;
        
        self.frame_count += 1;
        self.fps_timer += delta_time;
        
        // Print FPS once per second
        if self.fps_timer >= 1.0 {
            let fps = self.frame_count as f64 / self.fps_timer;
            println!("FPS: {:.1}", fps);
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }
        
        // Initialize instanced terrain on first draw
        if self.instanced_terrain.is_none() {
            println!("Initializing instanced terrain...");
            let ctx_ptr = &mut *self.ctx as *mut dyn RenderingBackend;
            unsafe {
                let terrain = InstancedTerrain::new(&mut *ctx_ptr, 160); // 32x original view distance
                
                // Create terrain bindings
                self.terrain_bindings = Bindings {
                    vertex_buffers: vec![terrain.base_vertex_buffer(), terrain.instance_buffer()],
                    index_buffer: terrain.index_buffer(),
                    images: vec![],
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
            // Drawing terrain
            
            self.ctx.apply_pipeline(&self.terrain_pipeline);
            self.ctx.apply_bindings(&self.terrain_bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::UniformsTerrain::new(
                mvp, 
                [0.0, 1.0, 0.0], // Green
                terrain.terrain_scale(),
                0.0 // terrain_y_base - start at Y=0
            )));
            
            // Draw all terrain instances
            self.ctx.draw(0, terrain.index_count(), terrain.instance_count());
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
                                let flash = (self.game.enemy_spawn_timer * 5.0).sin() * 0.5 + 0.5;
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
                        let search_angle = self.game.enemy_spawn_timer * 2.0;
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
        
        // Draw player bullets in yellow
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            // Draw debug cross
            renderer.draw_line(Vec3::new(-100.0, 0.0, 0.0), Vec3::new(100.0, 0.0, 0.0));
            renderer.draw_line(Vec3::new(0.0, -100.0, 0.0), Vec3::new(0.0, 100.0, 0.0));
            renderer.draw_line(Vec3::new(0.0, 0.0, -100.0), Vec3::new(0.0, 0.0, 100.0));
            
            // Draw player bullets
            for bullet in &self.game.bullets {
                if matches!(bullet.bullet_type, BulletType::Player) {
                    bullet.draw(&mut renderer);
                }
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
        
        // Draw enemy bullets in orange/red
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            // Draw enemy bullets
            for bullet in &self.game.bullets {
                if matches!(bullet.bullet_type, BulletType::Enemy) {
                    bullet.draw(&mut renderer);
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.5, 0.0]))); // Orange
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Draw laser beams in two passes - lines for core and triangles for glow
        vertices.clear();
        indices.clear();
        
        // First pass: Draw laser core as bright lines
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Lines);
            
            for enemy in &self.game.enemies {
                if let Some((laser_start, laser_end)) = enemy.get_laser_info() {
                    // Draw bright core line
                    renderer.draw_line(laser_start, laser_end);
                    
                    // Add some extra lines for thickness and 3D effect
                    let laser_dir = (laser_end - laser_start).normalize();
                    
                    // Calculate perpendicular vectors for creating a 3D beam
                    let up = Vec3::new(0.0, 1.0, 0.0);
                    let laser_right = laser_dir.cross(up).normalize();
                    let laser_up = laser_right.cross(laser_dir).normalize();
                    
                    // Draw multiple lines to create a cylindrical beam effect
                    for i in 0..8 {
                        let angle = i as f32 * std::f32::consts::PI * 2.0 / 8.0;
                        let offset_x = angle.cos() * 8.0;
                        let offset_y = angle.sin() * 8.0;
                        let offset = laser_right * offset_x + laser_up * offset_y;
                        
                        renderer.draw_line(
                            laser_start + offset,
                            laser_end + offset
                        );
                    }
                    
                    // Add some intermediate points for a slight curve effect
                    let mid_point = (laser_start + laser_end) * 0.5;
                    renderer.draw_line(laser_start, mid_point);
                    renderer.draw_line(mid_point, laser_end);
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.line_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // Bright yellow for laser core
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 1.0, 0.0])));
            self.ctx.draw(0, indices.len() as i32, 1);
        }
        
        // Second pass: Draw laser glow as triangles
        vertices.clear();
        indices.clear();
        
        {
            let mut renderer = Renderer::new(&mut vertices, &mut indices);
            renderer.set_mode(RenderMode::Triangles);
            
            for enemy in &self.game.enemies {
                if let Some((laser_start, laser_end)) = enemy.get_laser_info() {
                    let laser_dir = (laser_end - laser_start).normalize();
                    let laser_right = Vec3::new(-laser_dir.z, 0.0, laser_dir.x).normalize();
                    
                    // Draw wide glow
                    let width = 40.0;
                    let right_offset = laser_right * width;
                    
                    renderer.draw_triangle(
                        laser_start - right_offset,
                        laser_start + right_offset,
                        laser_end + right_offset
                    );
                    
                    renderer.draw_triangle(
                        laser_start - right_offset,
                        laser_end + right_offset,
                        laser_end - right_offset
                    );
                }
            }
        }
        
        if !vertices.is_empty() {
            self.ctx.buffer_update(self.bindings.vertex_buffers[0], BufferSource::slice(&vertices));
            self.ctx.buffer_update(self.bindings.index_buffer, BufferSource::slice(&indices));
            self.ctx.apply_pipeline(&self.triangle_pipeline);
            self.ctx.apply_bindings(&self.bindings);
            // Translucent yellow-orange for glow
            self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms::new(mvp, [1.0, 0.8, 0.2])));
            self.ctx.draw(0, indices.len() as i32, 1);
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
            KeyCode::Up | KeyCode::W => self.input.up = true,
            KeyCode::Down | KeyCode::S => self.input.down = true,
            KeyCode::Space => self.input.shoot = true,
            KeyCode::LeftShift | KeyCode::RightShift => self.input.boost = true,
            KeyCode::Escape => window::request_quit(),
            KeyCode::P => self.paused = !self.paused,
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
            KeyCode::LeftShift | KeyCode::RightShift => self.input.boost = false,
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
    
    // Default window size - 75% will be calculated after window creation
    // Using 1200x900 as a reasonable default (75% of 1600x1200)
    let window_width = 1200;
    let window_height = 900;
    
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