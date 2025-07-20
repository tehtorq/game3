use glam::{Vec3, Mat4};
use crate::player::Player;
use crate::constants::*;

pub struct Camera {
    pub distance: f32,
    pub height: f32,
    // Smooth camera following
    pub smooth_pos: Vec3,
    pub smooth_rotation: f32,
    pub initialized: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            distance: CAMERA_DISTANCE,
            height: CAMERA_HEIGHT,
            smooth_pos: Vec3::ZERO,
            smooth_rotation: 0.0,
            initialized: false,
        }
    }

    pub fn update(&mut self, player: &Player, dt: f32) {
        // Initialize on first frame
        if !self.initialized {
            self.smooth_pos = player.pos;
            self.smooth_rotation = player.rotation;
            self.initialized = true;
        }
        
        // Dynamic position smoothing based on player speed
        let player_speed = player.vel.length();
        let base_smoothing = 8.0;
        let boost_smoothing = 20.0; // Much more responsive when moving fast
        
        // Interpolate smoothing based on speed (assuming boost speeds are > 2000)
        let speed_factor = (player_speed / 2000.0).clamp(0.0, 1.0);
        let position_smoothing = base_smoothing + (boost_smoothing - base_smoothing) * speed_factor;
        
        self.smooth_pos = self.smooth_pos.lerp(player.pos, 1.0 - (-position_smoothing * dt).exp());
        
        // Smooth rotation following with angle wrapping
        const ROTATION_SMOOTHING: f32 = 10.0; // Increased for more responsive turning
        let mut angle_diff = player.rotation - self.smooth_rotation;
        
        // Wrap angle difference to [-PI, PI]
        use std::f32::consts::PI;
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }
        
        self.smooth_rotation += angle_diff * (1.0 - (-ROTATION_SMOOTHING * dt).exp());
    }
    
    pub fn get_view_matrix(&self, _player: &Player) -> Mat4 {
        // Position camera behind and above the player using smoothed values
        let camera_offset = Vec3::new(
            self.smooth_rotation.sin() * self.distance,
            self.height,
            self.smooth_rotation.cos() * self.distance
        );
        
        let camera_pos = self.smooth_pos + camera_offset;
        
        // Look slightly ahead of the player
        let look_ahead = Vec3::new(
            -self.smooth_rotation.sin() * 50.0,
            0.0,
            -self.smooth_rotation.cos() * 50.0
        );
        let look_target = self.smooth_pos + look_ahead;
        
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        // Use left-handed coordinate system (this worked)
        Mat4::look_at_lh(camera_pos, look_target, up)
    }

    pub fn get_projection_matrix(&self, aspect: f32) -> Mat4 {
        // Use left-handed projection (this worked)
        Mat4::perspective_lh(CAMERA_FOV.to_radians(), aspect, CAMERA_NEAR, CAMERA_FAR)
    }
    
    pub fn get_position(&self) -> Vec3 {
        // Calculate camera position using smoothed values
        let camera_offset = Vec3::new(
            self.smooth_rotation.sin() * self.distance,
            self.height,
            self.smooth_rotation.cos() * self.distance
        );
        
        self.smooth_pos + camera_offset
    }
}