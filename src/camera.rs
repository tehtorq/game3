use glam::{Vec3, Mat4};
use crate::player::Player;

pub struct Camera {
    pub distance: f32,
    pub height: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            distance: 150.0,
            height: 80.0,
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
        let look_target = player.pos + Vec3::new(0.0, 20.0, 0.0); // Look slightly above player
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        // Use left-handed coordinate system (this worked)
        Mat4::look_at_lh(camera_pos, look_target, up)
    }

    pub fn get_projection_matrix(&self, aspect: f32) -> Mat4 {
        // Use left-handed projection (this worked)
        Mat4::perspective_lh(60.0f32.to_radians(), aspect, 1.0, 10000.0)  // Increased far plane
    }
}