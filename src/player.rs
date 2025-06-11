use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;

// Smoothstep function (matches GLSL smoothstep)
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}


// Biome-specific height functions (exact match to shader)
fn get_plains_height(x: f32, z: f32) -> f32 {
    // Very low frequency for gentle rolling plains
    let scale1 = 0.0002;
    let scale2 = 0.0004;
    
    // Gentle rolling hills
    let h1 = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 25.0;
    let h2 = (x * scale2 + 1.0).sin() * (z * scale2 * 1.2).cos() * 15.0;
    
    // Subtle undulations
    let detail = (x * 0.001).sin() * (z * 0.0008).sin() * 8.0;
    
    h1 + h2 + detail
}

fn get_canyon_height(x: f32, z: f32) -> f32 {
    // Lower frequency for wider features
    let scale1 = 0.0008;
    let scale2 = 0.0004;
    
    // Smooth rolling canyon
    let base = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 60.0;
    let secondary = (x * scale2 + 1.0).sin() * (z * scale2 * 1.2).cos() * 40.0;
    
    // Add some gentle valleys
    let mut valley = 0.0;
    valley += smoothstep(0.0, 1.0, (x * 0.001).sin()) * 30.0;
    valley += smoothstep(0.0, 1.0, (z * 0.0008).cos()) * 20.0;
    
    // Gentle undulations instead of cliffs
    let detail = (x * 0.005).sin() * (z * 0.004).cos() * 10.0;
    
    base + secondary - valley + detail
}

fn get_plateau_height(x: f32, z: f32) -> f32 {
    // Very low frequency for large, smooth plateaus
    let scale1 = 0.0002;
    let scale2 = 0.0003;
    
    // Smooth raised areas with tapered edges
    let raise1 = smoothstep(0.2, 0.8, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5);
    let raise2 = smoothstep(0.3, 0.7, (x * scale2 + 1.5).sin() * (z * scale2 - 0.8).cos() * 0.5 + 0.5);
    
    // Gradual height changes
    let h1 = raise1 * 60.0;
    let h2 = raise2 * 40.0;
    
    // Rolling surface
    let surface = (x * 0.001).sin() * (z * 0.0008).cos() * 15.0;
    
    // Gentle blend between heights
    let blend = (x * 0.0001 + z * 0.00015).sin() * 0.5 + 0.5;
    
    h1 * (1.0 - blend) + h2 * blend + surface
}

fn get_crystalline_height(x: f32, z: f32) -> f32 {
    // Lower frequency for larger crystal formations
    let scale1 = 0.002;
    let scale2 = 0.004;
    
    // Smooth crystal clusters instead of sharp spikes
    let cluster1 = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5);
    let cluster2 = smoothstep(0.4, 0.6, (x * scale2 + 1.0).cos() * (z * scale2 - 0.5).sin() * 0.5 + 0.5);
    
    // Varied heights with smooth transitions
    let h1 = cluster1 * 50.0;
    let h2 = cluster2 * 35.0;
    
    // Gentle crystalline texture
    let texture = ((x * 0.01).sin() * (z * 0.008).cos()).abs() * 20.0;
    
    // Smooth base elevation
    let base = (x * 0.0005).sin() * (z * 0.0004).cos() * 25.0;
    
    h1 + h2 + texture + base
}

fn get_volcanic_height(x: f32, z: f32) -> f32 {
    // Low frequency for broad volcanic features
    let scale1 = 0.0002;
    let scale2 = 0.0005;
    
    // Smooth volcanic cone with gentle slopes
    let dist = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
    let cone = smoothstep(1.0, 0.0, dist) * 80.0;
    
    // Rolling lava fields
    let fields = (x * scale2).sin() * (z * scale2 * 0.9).cos() * 25.0;
    
    // Gentle surface texture
    let texture = (x * 0.002).sin() * (z * 0.0018).cos() * 10.0;
    
    cone + fields + texture
}

fn get_mountains_height(x: f32, z: f32) -> f32 {
    // Much lower frequency for broader mountains
    let scale1 = 0.0002;
    let scale2 = 0.0004;
    
    // Smooth gaussian-like peaks instead of sharp ones
    let dist1 = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
    let dist2 = ((x * scale2 + 1.0).sin().powi(2) + (z * scale2 - 0.5).cos().powi(2)).sqrt();
    
    // Use gaussian falloff for smooth peaks
    let h1 = (-dist1 * dist1 * 2.0).exp() * 120.0;
    let h2 = (-dist2 * dist2 * 3.0).exp() * 80.0;
    
    // Rolling foothills
    let mut foothills = 0.0;
    foothills += (x * 0.0008).sin() * (z * 0.0007).cos() * 30.0;
    foothills += (x * 0.0012 + 0.5).sin() * (z * 0.001).sin() * 20.0;
    
    // Gentle valleys between peaks
    let valley = (x * 0.0003 + z * 0.0002).sin() * 15.0;
    
    h1 + h2 + foothills + valley
}

fn get_desert_height(x: f32, z: f32) -> f32 {
    // Low frequency for large dune fields
    let scale1 = 0.0003;
    let scale2 = 0.0006;
    
    // Smooth, rolling dunes
    let dunes = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1 * 1.2).cos() * 0.5 + 0.5) * 30.0;
    let secondary = (x * scale2 + 0.5).sin() * (z * scale2 * 0.8).cos() * 20.0;
    
    // Gentle ripples
    let ripples = (x * 0.003).sin() * (z * 0.0025).cos() * 5.0;
    
    dunes + secondary + ripples
}

fn get_arctic_height(x: f32, z: f32) -> f32 {
    // Low frequency for smooth, rolling ice sheets
    let scale1 = 0.0003;
    let scale2 = 0.0006;
    
    // Smooth rolling glacial terrain
    let glacial = (x * scale1).sin() * (z * scale1 * 0.9).cos() * 40.0;
    let sheets = (x * scale2 + 0.5).cos() * (z * scale2 * 1.1).sin() * 30.0;
    
    // Gentle ice dunes
    let dunes = smoothstep(0.2, 0.8, (x * 0.001).sin() * (z * 0.0008).cos() * 0.5 + 0.5) * 25.0;
    
    // Subtle surface texture
    let texture = (x * 0.008).sin() * (z * 0.007).cos() * 10.0;
    
    // Very gentle crevasses
    let crevasse = smoothstep(0.4, 0.6, (x * 0.002 + z * 0.0015).sin()) * -15.0;
    
    glacial + sheets + dunes + texture + crevasse
}

fn get_badlands_height(x: f32, z: f32) -> f32 {
    // Low frequency for wider, smoother features
    let scale1 = 0.0004;
    let scale2 = 0.0008;
    
    // Smooth eroded hills
    let hills = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 50.0;
    let erosion = smoothstep(0.3, 0.7, (x * scale2 + 0.7).cos() * (z * scale2 * 1.2).sin() * 0.5 + 0.5) * 30.0;
    
    // Gentle mesas with sloped sides
    let mesa = smoothstep(0.2, 0.6, (x * 0.0003).sin() * (z * 0.00025).cos() * 0.5 + 0.5) * 40.0;
    
    // Rolling badland texture
    let texture = (x * 0.002).sin() * (z * 0.0018).cos() * 15.0;
    
    hills + erosion + mesa + texture
}

fn get_floating_height(x: f32, z: f32) -> f32 {
    // Low frequency for large floating islands
    let scale1 = 0.0002;
    let scale2 = 0.0004;
    
    // Smooth floating plateaus
    let island1 = smoothstep(0.3, 0.7, (x * scale1).sin() * (z * scale1).cos() * 0.5 + 0.5) * 60.0;
    let island2 = smoothstep(0.4, 0.6, (x * scale2 + 0.8).cos() * (z * scale2 * 0.9).sin() * 0.5 + 0.5) * 40.0;
    
    // Gentle surface
    let surface = (x * 0.001).sin() * (z * 0.0008).cos() * 10.0;
    
    80.0 + island1 + island2 + surface
}

fn get_caverns_height(x: f32, z: f32) -> f32 {
    // Low frequency for larger cavern systems
    let scale1 = 0.0004;
    let scale2 = 0.0008;
    
    // Rolling base terrain
    let base = (x * scale1).sin() * (z * scale1 * 0.8).cos() * 40.0;
    
    // Smooth depressions instead of sharp holes
    let depression1 = smoothstep(0.6, 0.3, (x * scale2).sin() * (z * scale2).cos() * 0.5 + 0.5) * -30.0;
    let depression2 = smoothstep(0.5, 0.2, (x * scale2 * 1.3 + 1.0).cos() * (z * scale2 * 0.9).sin() * 0.5 + 0.5) * -20.0;
    
    // Gentle undulations
    let detail = (x * 0.002).sin() * (z * 0.0015).cos() * 10.0;
    
    base + depression1 + depression2 + detail
}

fn get_swamp_height(x: f32, z: f32) -> f32 {
    // Low frequency for gentle swamp terrain
    let scale1 = 0.0005;
    let scale2 = 0.001;
    
    // Very gentle undulations
    let undulation = (x * scale1).sin() * (z * scale1 * 0.9).cos() * 15.0;
    let pools = smoothstep(0.4, 0.6, (x * scale2).sin() * (z * scale2 * 1.1).cos() * 0.5 + 0.5) * -10.0;
    
    // Subtle surface variation
    let surface = (x * 0.003).sin() * (z * 0.0025).cos() * 5.0;
    
    undulation + pools + surface
}


// Enhanced biome selection with more variety and smaller regions (matches shader exactly)
fn get_biome_height_shader(x: f32, z: f32) -> f32 {
    // Multi-scale noise for more organic biome distribution
    let noise1 = (x * 0.0003).sin() * (z * 0.0003).cos();
    let noise2 = (x * 0.0007 + 1.3).sin() * (z * 0.0006 - 0.7).sin();
    let noise3 = (x * 0.0013 - 2.1).cos() * (z * 0.0011 + 1.9).sin();
    
    // Combine noises for complex patterns
    let mut biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Add local variation for sub-biomes
    let local_var = (x * 0.01).sin() * (z * 0.01).cos() * 0.1;
    biome_noise += local_var;
    
    // 12 biome types distributed across the noise range
    if biome_noise < -0.7 {
        get_canyon_height(x, z)
    } else if biome_noise < -0.5 {
        get_caverns_height(x, z)
    } else if biome_noise < -0.3 {
        get_badlands_height(x, z)
    } else if biome_noise < -0.1 {
        get_plateau_height(x, z)
    } else if biome_noise < 0.1 {
        get_plains_height(x, z)
    } else if biome_noise < 0.25 {
        get_desert_height(x, z)
    } else if biome_noise < 0.4 {
        get_swamp_height(x, z)
    } else if biome_noise < 0.5 {
        get_crystalline_height(x, z)
    } else if biome_noise < 0.6 {
        get_volcanic_height(x, z)
    } else if biome_noise < 0.7 {
        get_arctic_height(x, z)
    } else if biome_noise < 0.8 {
        get_floating_height(x, z)
    } else {
        get_mountains_height(x, z)
    }
}

// Smooth blending between biomes (matches shader exactly)
fn get_blended_biome_height_shader(x: f32, z: f32) -> f32 {
    // Sample multiple nearby points for smoother transitions
    let sample_dist = 50.0;
    let h_center = get_biome_height_shader(x, z);
    let h_north = get_biome_height_shader(x, z + sample_dist);
    let h_south = get_biome_height_shader(x, z - sample_dist);
    let h_east = get_biome_height_shader(x + sample_dist, z);
    let h_west = get_biome_height_shader(x - sample_dist, z);
    
    // Average nearby samples for smoother terrain
    let primary_height = (h_center * 2.0 + h_north + h_south + h_east + h_west) / 6.0;
    
    // Add rolling hills with lower frequency
    let mut hills = 0.0;
    hills += (x * 0.0001).sin() * (z * 0.00012).cos() * 60.0;
    hills += (x * 0.00018 + 1.5).sin() * (z * 0.00015 - 0.7).cos() * 40.0;
    hills += (x * 0.00025 - 0.3).sin() * (z * 0.0003 + 1.2).cos() * 25.0;
    
    // Add gentle undulations
    let mut undulation = 0.0;
    undulation += (x * 0.0004).sin() * (z * 0.0004).cos() * 15.0;
    undulation += (x * 0.0008 + 2.1).sin() * (z * 0.0007 - 1.3).sin() * 10.0;
    
    // Very gentle large scale features
    let continent_scale = 0.00005;
    let continental = (x * continent_scale).sin() * (z * continent_scale).cos() * 30.0;
    
    // Smooth everything together
    let height = primary_height * 0.7 + hills * 0.2 + undulation * 0.1;
    
    height + continental
}

// Calculate terrain height at a given position (matches shader calculation)
fn terrain_height_at(x: f32, z: f32) -> f32 {
    get_blended_biome_height_shader(x, z)
}

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
        let terrain_below = terrain_height_at(self.pos.x, self.pos.z);
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
        terrain_height_at(self.pos.x, self.pos.z)
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