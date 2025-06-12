use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use crate::terrain::Terrain;

#[derive(Clone)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub rotation: f32,
    pub thrust: Vec3,          // Current thrust vector
    pub angular_vel: f32,      // Rotation speed
    pub banking: f32,
    pub pitch: f32,
    pub afterburner_fuel: f32, // 0.0 to 1.0
    pub braking: bool,         // Air brake active
    pub shield: f32,           // Shield strength 0.0 to 1.0
    pub shield_recharge_timer: f32, // Time until shield starts recharging
    pub trail_points: Vec<TrailPoint>, // Engine trail
    trail_timer: f32,          // Timer for adding trail points
}

#[derive(Clone)]
pub struct TrailPoint {
    pub pos: Vec3,
    pub lifetime: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec3::new(0.0, 50.0, 0.0),
            vel: Vec3::ZERO,
            rotation: 0.0,
            thrust: Vec3::ZERO,
            angular_vel: 0.0,
            banking: 0.0,
            pitch: 0.0,
            afterburner_fuel: 1.0,
            braking: false,
            shield: 1.0,
            shield_recharge_timer: 0.0,
            trail_points: Vec::new(),
            trail_timer: 0.0,
        }
    }

    // Get normalized forward vector (pointing in the direction the ship is facing)
    pub fn v_forward(&self) -> Vec3 {
        let yaw_cos = self.rotation.cos();
        let yaw_sin = self.rotation.sin();
        let pitch_cos = self.pitch.cos();
        let pitch_sin = self.pitch.sin();
        
        Vec3::new(
            -yaw_sin * pitch_cos,
            pitch_sin,  // Positive pitch should give positive Y (up)
            -yaw_cos * pitch_cos
        ).normalize()
    }
    
    // Get normalized up vector (pointing up from the ship)
    pub fn v_up(&self) -> Vec3 {
        let rotation = rotation_matrix(self.rotation, self.pitch, self.banking);
        rotation.transform_vector3(Vec3::new(0.0, 1.0, 0.0)).normalize()
    }
    
    // Get normalized right vector (pointing to the right of the ship)
    pub fn v_right(&self) -> Vec3 {
        let rotation = rotation_matrix(self.rotation, self.pitch, self.banking);
        rotation.transform_vector3(Vec3::new(1.0, 0.0, 0.0)).normalize()
    }
    
    // Apply mouse aim - ship rotates to face mouse target
    pub fn apply_mouse_aim(&mut self, mouse_target_x: f32, mouse_target_y: f32, dt: f32) {
        const MAX_TURN_SPEED: f32 = 2.5; // Maximum rotation speed
        const MAX_PITCH: f32 = 0.8; // Limit pitch to about 45 degrees
        const YAW_SENSITIVITY: f32 = 1.5; // How much the mouse affects yaw
        const PITCH_SENSITIVITY: f32 = 0.4; // Reduced pitch sensitivity
        const DEADZONE: f32 = 0.05; // Small deadzone to prevent oscillation
        const PITCH_DEADZONE: f32 = 0.15; // Larger deadzone for pitch
        
        // Calculate desired turn rate based on mouse offset from center
        let yaw_error = mouse_target_x * YAW_SENSITIVITY;
        
        // Apply deadzone to prevent tiny oscillations
        let yaw_rate = if yaw_error.abs() < DEADZONE {
            0.0
        } else {
            // Use the error directly as turn rate (proportional control)
            yaw_error * MAX_TURN_SPEED
        };
        
        // Apply rotation
        self.rotation += yaw_rate * dt;
        
        // Wrap rotation
        use std::f32::consts::PI;
        while self.rotation > PI {
            self.rotation -= 2.0 * PI;
        }
        while self.rotation < -PI {
            self.rotation += 2.0 * PI;
        }
        
        // Calculate target pitch with deadzone
        let pitch_input = if mouse_target_y.abs() < PITCH_DEADZONE {
            0.0
        } else {
            -mouse_target_y * MAX_PITCH * PITCH_SENSITIVITY
        };
        
        // Smoothly adjust pitch towards target
        let target_pitch = pitch_input;
        let pitch_diff = target_pitch - self.pitch;
        let pitch_change = pitch_diff * 5.0 * dt; // Smooth pitch adjustment
        self.pitch = (self.pitch + pitch_change).clamp(-MAX_PITCH, MAX_PITCH);
    }

    pub fn update(&mut self, left: bool, right: bool, forward: bool, backward: bool, boost: bool, up: bool, dt: f32) {
        // Physics constants
        const TURN_ACCELERATION: f32 = 8.0;  // For mouse turning
        const TURN_DAMPING: f32 = 0.9;      // Less damping for more responsive controls
        const MAX_TURN_SPEED: f32 = 4.0;    // Slightly faster max turn rate
        
        const THRUST_POWER: f32 = 800.0;    // Forward thrust power
        const STRAFE_POWER: f32 = 600.0;    // Lateral thrust power
        const BOOST_MULTIPLIER: f32 = 2.5;  // Speed boost when holding shift
        
        const AIR_DRAG: f32 = 2.5;          // Higher drag for hover behavior
        const HOVER_DRAG: f32 = 6.0;        // Extra drag when not thrusting
        const GRAVITY: f32 = 80.0;          // Slightly stronger gravity
        const MAX_SPEED: f32 = 800.0;       // Increased to outrun aggressive enemies
        const MAX_VERTICAL_SPEED: f32 = 300.0;  // Reduced to match
        
        // Angular velocity is no longer used (direct mouse control)
        self.angular_vel = 0.0;
        
        // Wrap rotation to keep it in [-PI, PI] range
        use std::f32::consts::PI;
        while self.rotation > PI {
            self.rotation -= 2.0 * PI;
        }
        while self.rotation < -PI {
            self.rotation += 2.0 * PI;
        }
        
        // Update banking based on strafing
        let strafe_input = (left as i32 - right as i32) as f32;
        let target_banking = -strafe_input * 0.4;  // Bank when strafing
        self.banking = self.banking * 0.7 + target_banking * 0.3;
        
        // Calculate movement directions including pitch
        let yaw_cos = self.rotation.cos();
        let yaw_sin = self.rotation.sin();
        let pitch_cos = self.pitch.cos();
        let pitch_sin = self.pitch.sin();
        
        // Forward direction includes pitch
        let forward_dir = Vec3::new(
            -yaw_sin * pitch_cos,
            pitch_sin,  // Positive pitch should give positive Y (up)
            -yaw_cos * pitch_cos
        );
        
        // Right direction remains horizontal
        let right_dir = Vec3::new(yaw_cos, 0.0, -yaw_sin);
        
        // Reset thrust
        self.thrust = Vec3::ZERO;
        
        // Apply forward/backward thrust
        let mut forward_thrust = 0.0;
        if forward {
            forward_thrust += THRUST_POWER;
        }
        if backward {
            forward_thrust -= THRUST_POWER * 0.7; // Slightly slower backwards
        }
        
        // Apply boost multiplier
        if boost && forward_thrust > 0.0 {
            forward_thrust *= BOOST_MULTIPLIER;
        }
        
        self.thrust += forward_dir * forward_thrust;
        
        // Apply strafe thrust
        if left {
            self.thrust += right_dir * STRAFE_POWER;
        }
        if right {
            self.thrust -= right_dir * STRAFE_POWER;
        }
        
        // Apply vertical thrust
        if up {
            self.thrust.y += THRUST_POWER; // Go up
        }
        
        // Apply thrust to velocity
        self.vel += self.thrust * dt;
        
        // Apply drag - more when not thrusting (hover behavior)
        let drag = if self.thrust.length() < 0.1 { HOVER_DRAG } else { AIR_DRAG };
        self.vel *= 1.0 - (drag * dt);
        
        // Apply gravity - DISABLED
        // self.vel.y -= GRAVITY * dt;
        
        // Limit speeds
        let horizontal_speed = Vec3::new(self.vel.x, 0.0, self.vel.z).length();
        if horizontal_speed > MAX_SPEED {
            let scale = MAX_SPEED / horizontal_speed;
            self.vel.x *= scale;
            self.vel.z *= scale;
        }
        self.vel.y = self.vel.y.clamp(-MAX_VERTICAL_SPEED, MAX_VERTICAL_SPEED);
        
        // Update position
        self.pos += self.vel * dt;
        
        // Constrain player height based on terrain below
        let terrain_below = Terrain::height_at(self.pos.x, self.pos.z);
        let min_height = terrain_below + 10.0;
        
        if self.pos.y < min_height {
            self.pos.y = min_height;
            self.vel.y = self.vel.y.max(0.0); // Stop downward velocity
            
            // Ground effect - reduce drag and provide lift when close to terrain
            let height_above_terrain = self.pos.y - terrain_below;
            if height_above_terrain < 100.0 {
                // Stronger ground effect that scales with proximity
                let effect_strength = 1.0 - (height_above_terrain / 100.0);
                self.vel *= 1.0 + (0.05 * effect_strength); // Up to 5% speed boost
                self.vel.y += 20.0 * effect_strength * dt;  // Upward cushion effect
            }
        }
        
        // Max altitude (above sea level, not terrain)
        const MAX_ALTITUDE: f32 = 3000.0;
        if self.pos.y > MAX_ALTITUDE {
            self.pos.y = MAX_ALTITUDE;
            self.vel.y = self.vel.y.min(0.0);
        }
        
        // Update trail
        self.update_trail(dt, boost && forward);
        
        // Update shield recharge
        if self.shield_recharge_timer > 0.0 {
            self.shield_recharge_timer -= dt;
        } else if self.shield < 1.0 {
            // Recharge shield when timer expires
            const SHIELD_RECHARGE_RATE: f32 = 0.2; // 20% per second
            self.shield = (self.shield + SHIELD_RECHARGE_RATE * dt).min(1.0);
        }
    }
    
    pub fn get_speed(&self) -> f32 {
        self.vel.length()
    }
    
    pub fn get_altitude(&self) -> f32 {
        self.pos.y
    }
    
    pub fn get_terrain_height(&self) -> f32 {
        Terrain::height_at(self.pos.x, self.pos.z)
    }
    
    pub fn get_height_above_terrain(&self) -> f32 {
        self.pos.y - self.get_terrain_height()
    }
    
    pub fn take_damage(&mut self, amount: f32) -> bool {
        // Returns true if player was damaged (not blocked by shield)
        if self.shield > 0.0 {
            self.shield = (self.shield - amount).max(0.0);
            self.shield_recharge_timer = 3.0; // 3 seconds before shield starts recharging
            false // Shield absorbed the damage
        } else {
            true // No shield, player takes damage
        }
    }
    
    fn update_trail(&mut self, dt: f32, boosting: bool) {
        // Update existing trail points
        self.trail_points.retain_mut(|point| {
            point.lifetime -= dt;
            point.lifetime > 0.0
        });
        
        // Add new trail points
        self.trail_timer -= dt;
        if self.trail_timer <= 0.0 {
            // Calculate wing tip positions using rotation matrix
            let rotation = rotation_matrix(self.rotation, self.pitch, self.banking);
            let scale = 20.0; // Match the ship scale
            
            // Left wing tip
            let left_wing_offset = rotation.transform_vector3(Vec3::new(-scale, 0.0, scale * 0.5));
            let left_pos = self.pos + left_wing_offset;
            
            // Right wing tip  
            let right_wing_offset = rotation.transform_vector3(Vec3::new(scale, 0.0, scale * 0.5));
            let right_pos = self.pos + right_wing_offset;
            
            // Add trail points with longer lifetime when boosting
            let lifetime = if boosting { 0.8 } else { 0.3 };
            
            self.trail_points.push(TrailPoint {
                pos: left_pos,
                lifetime,
            });
            
            self.trail_points.push(TrailPoint {
                pos: right_pos,
                lifetime,
            });
            
            // Reset timer - faster trail spawn for smoother lines
            self.trail_timer = 0.02; // Always fast for smooth trails
        }
        
        // Limit trail length
        const MAX_TRAIL_POINTS: usize = 120;
        if self.trail_points.len() > MAX_TRAIL_POINTS {
            self.trail_points.drain(0..self.trail_points.len() - MAX_TRAIL_POINTS);
        }
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