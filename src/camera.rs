use glam::{Vec3, Mat4};
use crate::player::Player;
use crate::constants::*;

pub struct Camera {
    pub distance: f32,
    pub height: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            distance: CAMERA_DISTANCE,
            height: CAMERA_HEIGHT,
        }
    }

    pub fn get_view_matrix(&self, player: &Player) -> Mat4 {
        // Position camera behind and above the player
        let camera_offset = Vec3::new(
            player.rotation.sin() * self.distance,
            self.height,
            player.rotation.cos() * self.distance
        );
        
        let camera_pos = player.pos + camera_offset;
        let look_target = player.pos; // Look directly at player
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        // Use left-handed coordinate system (this worked)
        Mat4::look_at_lh(camera_pos, look_target, up)
    }

    pub fn get_projection_matrix(&self, aspect: f32) -> Mat4 {
        // Use left-handed projection (this worked)
        Mat4::perspective_lh(CAMERA_FOV.to_radians(), aspect, CAMERA_NEAR, CAMERA_FAR)
    }
}