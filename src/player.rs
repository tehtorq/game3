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

    pub fn update(&mut self, left: bool, right: bool, up: bool, down: bool, boost: bool, dt: f32) {
        // Physics constants
        const TURN_ACCELERATION: f32 = 8.0;  // Doubled for snappier turning
        const TURN_DAMPING: f32 = 0.9;      // Less damping for more responsive controls
        const MAX_TURN_SPEED: f32 = 4.0;    // Slightly faster max turn rate
        
        const THRUST_POWER: f32 = 1200.0;   // More than doubled for snappier acceleration
        const VERTICAL_THRUST: f32 = 600.0;  // Doubled for better vertical control
        const AFTERBURNER_MULTIPLIER: f32 = 3.0;  // Even more powerful to escape aggressive enemies
        const AFTERBURNER_DRAIN: f32 = 0.33; // 3 seconds of fuel
        const AFTERBURNER_REGEN: f32 = 0.15; // Faster regen (6.7 seconds to refill)
        
        const AIR_DRAG: f32 = 1.5;          // Much higher drag for tighter control
        const BRAKE_DRAG: f32 = 4.0;        // Stronger brakes
        const GRAVITY: f32 = 80.0;          // Slightly stronger gravity
        const MAX_SPEED: f32 = 800.0;       // Increased to outrun aggressive enemies
        const MAX_VERTICAL_SPEED: f32 = 300.0;  // Reduced to match
        
        // Handle rotation with acceleration
        let turn_input = (right as i32 - left as i32) as f32;
        self.angular_vel += turn_input * TURN_ACCELERATION * dt;
        self.angular_vel = self.angular_vel.clamp(-MAX_TURN_SPEED, MAX_TURN_SPEED);
        self.angular_vel *= TURN_DAMPING; // Damping
        self.rotation += self.angular_vel * dt;
        
        // Update banking based on angular velocity - more responsive
        let target_banking = self.angular_vel / MAX_TURN_SPEED * 0.8;  // More pronounced banking
        self.banking = self.banking * 0.7 + target_banking * 0.3;      // Faster response
        
        // Update pitch based on vertical input - more responsive
        let target_pitch = (down as i32 - up as i32) as f32 * 0.4;    // More pronounced pitch
        self.pitch = self.pitch * 0.8 + target_pitch * 0.2;           // Faster response
        
        // Calculate thrust based on inputs
        let forward_dir = Vec3::new(-self.rotation.sin(), 0.0, -self.rotation.cos());
        let mut thrust_magnitude = THRUST_POWER;
        
        // Afterburner system
        if boost && self.afterburner_fuel > 0.0 {
            thrust_magnitude *= AFTERBURNER_MULTIPLIER;
            self.afterburner_fuel = (self.afterburner_fuel - AFTERBURNER_DRAIN * dt).max(0.0);
        } else {
            self.afterburner_fuel = (self.afterburner_fuel + AFTERBURNER_REGEN * dt).min(1.0);
        }
        
        // Apply forward thrust
        self.thrust = forward_dir * thrust_magnitude;
        
        // Add vertical thrust
        if up {
            self.thrust.y -= VERTICAL_THRUST;
        }
        if down {
            self.thrust.y += VERTICAL_THRUST;
        }
        
        // Apply thrust to velocity
        self.vel += self.thrust * dt;
        
        // Apply drag (more when braking)
        self.braking = self.thrust.length() < 0.1 && (left || right || up || down);
        let drag = if self.braking { BRAKE_DRAG } else { AIR_DRAG };
        self.vel *= 1.0 - (drag * dt);
        
        // Apply gravity
        self.vel.y -= GRAVITY * dt;
        
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
        const MAX_ALTITUDE: f32 = 1000.0;
        if self.pos.y > MAX_ALTITUDE {
            self.pos.y = MAX_ALTITUDE;
            self.vel.y = self.vel.y.min(0.0);
        }
        
        // Update trail
        self.update_trail(dt, boost);
        
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
    
    fn update_trail(&mut self, dt: f32, boost: bool) {
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
            let lifetime = if boost { 0.6 } else { 0.3 };
            
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