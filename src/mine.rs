use glam::Vec3;
use crate::renderer::{Renderer, Drawable};

#[derive(Clone)]
pub struct Mine {
    pub pos: Vec3,
    pub lifetime: f32,
    pub armed: bool,
    pub blink_timer: f32,
}

impl Mine {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            lifetime: 10.0,  // Mines last 10 seconds
            armed: false,    // Takes 1 second to arm
            blink_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        self.blink_timer += dt;
        
        // Arm after 1 second
        if !self.armed && self.lifetime < 9.0 {
            self.armed = true;
        }
        
        self.lifetime > 0.0
    }
    
    pub fn get_explosion_radius(&self) -> f32 {
        50.0
    }
}

impl Drawable for Mine {
    fn draw(&self, renderer: &mut Renderer) {
        // Blinking red indicator
        let blink_rate = if self.armed { 2.0 } else { 0.5 };
        let visible = (self.blink_timer * blink_rate * 2.0 * std::f32::consts::PI).sin() > 0.0;
        
        if visible {
            // Draw mine as a spiked sphere
            renderer.draw_octahedron(self.pos, 10.0, glam::Mat4::IDENTITY);
            
            // Add danger spikes
            let spike_positions = [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, -1.0),
            ];
            
            for &spike_dir in &spike_positions {
                let spike_pos = self.pos + spike_dir * 15.0;
                renderer.draw_pyramid(spike_pos, 5.0, glam::Mat4::IDENTITY);
            }
        }
    }
}