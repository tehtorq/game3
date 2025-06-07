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
        
        // Draw a proper spaceship using triangles
        let scale = 20.0;
        
        // Ship body vertices
        let nose = self.pos + rotation.transform_vector3(Vec3::new(0.0, 0.0, -scale * 1.5));
        let left_wing = self.pos + rotation.transform_vector3(Vec3::new(-scale, 0.0, scale * 0.5));
        let right_wing = self.pos + rotation.transform_vector3(Vec3::new(scale, 0.0, scale * 0.5));
        let tail_top = self.pos + rotation.transform_vector3(Vec3::new(0.0, scale * 0.5, scale));
        let tail_bottom = self.pos + rotation.transform_vector3(Vec3::new(0.0, -scale * 0.3, scale));
        let center_top = self.pos + rotation.transform_vector3(Vec3::new(0.0, scale * 0.3, 0.0));
        let center_bottom = self.pos + rotation.transform_vector3(Vec3::new(0.0, -scale * 0.2, 0.0));
        
        // Main body triangles
        // Top surfaces
        renderer.draw_triangle(nose, center_top, left_wing);
        renderer.draw_triangle(nose, right_wing, center_top);
        renderer.draw_triangle(center_top, right_wing, tail_top);
        renderer.draw_triangle(center_top, tail_top, left_wing);
        
        // Bottom surfaces
        renderer.draw_triangle(nose, left_wing, center_bottom);
        renderer.draw_triangle(nose, center_bottom, right_wing);
        renderer.draw_triangle(center_bottom, tail_bottom, right_wing);
        renderer.draw_triangle(center_bottom, left_wing, tail_bottom);
        
        // Side panels
        renderer.draw_triangle(left_wing, tail_top, tail_bottom);
        renderer.draw_triangle(right_wing, tail_bottom, tail_top);
        
        // Wing surfaces
        let left_wing_tip = self.pos + rotation.transform_vector3(Vec3::new(-scale * 1.5, 0.0, 0.0));
        let right_wing_tip = self.pos + rotation.transform_vector3(Vec3::new(scale * 1.5, 0.0, 0.0));
        
        // Left wing
        renderer.draw_triangle(left_wing, left_wing_tip, center_top);
        renderer.draw_triangle(left_wing, center_bottom, left_wing_tip);
        
        // Right wing
        renderer.draw_triangle(right_wing, center_top, right_wing_tip);
        renderer.draw_triangle(right_wing, right_wing_tip, center_bottom);
        
        // Rear panel
        renderer.draw_triangle(tail_top, tail_bottom, self.pos + rotation.transform_vector3(Vec3::new(0.0, 0.0, scale)));
    }
}