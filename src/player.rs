use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;

#[derive(Clone)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rotation: f32,
    banking: f32,
    pitch: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec3::new(0.0, 50.0, 0.0),
            vel: Vec3::ZERO,
            rotation: 0.0,
            banking: 0.0,
            pitch: 0.0,
        }
    }

    pub fn update(&mut self, left: bool, right: bool, up: bool, down: bool, dt: f32) {
        const TURN_SPEED: f32 = 2.0;
        const FORWARD_SPEED: f32 = 500.0;
        const VERTICAL_SPEED: f32 = 150.0;
        
        // Handle rotation
        if left {
            self.rotation -= TURN_SPEED * dt;
        }
        if right {
            self.rotation += TURN_SPEED * dt;
        }
        
        // Update banking based on turning
        let target_banking = if left {
            -0.5
        } else if right {
            0.5
        } else {
            0.0
        };
        self.banking = self.banking * 0.9 + target_banking * 0.1;
        
        // Update pitch for vertical movement
        let target_pitch = if up {
            0.3
        } else if down {
            -0.3
        } else {
            0.0
        };
        self.pitch = self.pitch * 0.9 + target_pitch * 0.1;
        
        // Calculate forward direction based on rotation
        let forward = Vec3::new(
            -self.rotation.sin() * FORWARD_SPEED,
            0.0,
            -self.rotation.cos() * FORWARD_SPEED
        );
        
        // Set velocity based on rotation and inputs
        self.vel = forward;
        
        // Add vertical movement
        if up {
            self.vel.y = VERTICAL_SPEED;
        }
        if down {
            self.vel.y = -VERTICAL_SPEED;
        }
        
        self.pos += self.vel * dt;
        
        // Constrain player height
        self.pos.y = self.pos.y.clamp(20.0, 300.0);
    }
}

impl Drawable for Player {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.rotation, self.pitch, self.banking);
        
        // Draw a proper spaceship
        let scale = 20.0;
        
        // Ship body vertices
        let nose = self.pos + rotation.transform_vector3(Vec3::new(0.0, 0.0, -scale * 1.5));
        let left_wing = self.pos + rotation.transform_vector3(Vec3::new(-scale, 0.0, scale * 0.5));
        let right_wing = self.pos + rotation.transform_vector3(Vec3::new(scale, 0.0, scale * 0.5));
        let tail_top = self.pos + rotation.transform_vector3(Vec3::new(0.0, scale * 0.5, scale));
        let tail_bottom = self.pos + rotation.transform_vector3(Vec3::new(0.0, -scale * 0.3, scale));
        let center_top = self.pos + rotation.transform_vector3(Vec3::new(0.0, scale * 0.3, 0.0));
        
        // Main body
        renderer.draw_line(nose, left_wing);
        renderer.draw_line(nose, right_wing);
        renderer.draw_line(left_wing, right_wing);
        
        // Tail section
        renderer.draw_line(left_wing, tail_top);
        renderer.draw_line(right_wing, tail_top);
        renderer.draw_line(left_wing, tail_bottom);
        renderer.draw_line(right_wing, tail_bottom);
        renderer.draw_line(tail_top, tail_bottom);
        
        // Cockpit
        renderer.draw_line(nose, center_top);
        renderer.draw_line(center_top, tail_top);
        
        // Wings detail
        let left_wing_tip = self.pos + rotation.transform_vector3(Vec3::new(-scale * 1.5, 0.0, 0.0));
        let right_wing_tip = self.pos + rotation.transform_vector3(Vec3::new(scale * 1.5, 0.0, 0.0));
        renderer.draw_line(left_wing, left_wing_tip);
        renderer.draw_line(right_wing, right_wing_tip);
        
        // Engine exhausts
        let left_engine = self.pos + rotation.transform_vector3(Vec3::new(-scale * 0.5, 0.0, scale));
        let right_engine = self.pos + rotation.transform_vector3(Vec3::new(scale * 0.5, 0.0, scale));
        let left_exhaust = self.pos + rotation.transform_vector3(Vec3::new(-scale * 0.5, 0.0, scale * 1.3));
        let right_exhaust = self.pos + rotation.transform_vector3(Vec3::new(scale * 0.5, 0.0, scale * 1.3));
        
        renderer.draw_line(left_engine, left_exhaust);
        renderer.draw_line(right_engine, right_exhaust);
    }
}