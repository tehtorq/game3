use glam::{Vec2, Vec3};
use std::collections::{HashSet, VecDeque};

const PREDICTION_DISTANCE: f32 = 5.0; // Predict 5 chunks ahead
const VELOCITY_SMOOTHING: f32 = 0.1;  // How much to smooth velocity
const MIN_VELOCITY: f32 = 10.0;        // Minimum velocity to trigger prediction

pub struct PredictiveLoader {
    // Player movement tracking
    position_history: VecDeque<(Vec3, f64)>, // position, timestamp
    smoothed_velocity: Vec3,
    smoothed_direction: Vec3,
    
    // Chunk prediction
    predicted_chunks: HashSet<(i32, i32)>,
    priority_queue: Vec<((i32, i32), f32)>, // chunk coords, priority
    
    // Settings
    chunk_size: f32,
    max_history: usize,
}

impl PredictiveLoader {
    pub fn new(chunk_size: f32) -> Self {
        Self {
            position_history: VecDeque::with_capacity(60),
            smoothed_velocity: Vec3::ZERO,
            smoothed_direction: Vec3::new(0.0, 0.0, -1.0),
            predicted_chunks: HashSet::new(),
            priority_queue: Vec::new(),
            chunk_size,
            max_history: 30, // Half second of history at 60fps
        }
    }
    
    pub fn update(&mut self, player_pos: Vec3, timestamp: f64) {
        // Add to history
        self.position_history.push_back((player_pos, timestamp));
        
        // Remove old entries
        while self.position_history.len() > self.max_history {
            self.position_history.pop_front();
        }
        
        // Calculate velocity and direction
        if self.position_history.len() >= 2 {
            let (old_pos, old_time) = self.position_history[0];
            let (new_pos, new_time) = self.position_history[self.position_history.len() - 1];
            
            let dt = (new_time - old_time) as f32;
            if dt > 0.0 {
                let instant_velocity = (new_pos - old_pos) / dt;
                
                // Smooth velocity
                self.smoothed_velocity = self.smoothed_velocity.lerp(instant_velocity, VELOCITY_SMOOTHING);
                
                // Update direction if moving fast enough
                let speed = self.smoothed_velocity.length();
                if speed > MIN_VELOCITY {
                    self.smoothed_direction = self.smoothed_velocity.normalize();
                }
            }
        }
        
        // Predict future positions
        self.update_predictions(player_pos);
    }
    
    fn update_predictions(&mut self, player_pos: Vec3) {
        self.predicted_chunks.clear();
        self.priority_queue.clear();
        
        let speed = self.smoothed_velocity.length();
        if speed < MIN_VELOCITY {
            return; // Not moving fast enough to predict
        }
        
        // Current chunk
        let current_chunk_x = (player_pos.x / self.chunk_size).floor() as i32;
        let current_chunk_z = (player_pos.z / self.chunk_size).floor() as i32;
        
        // Predict future positions
        let prediction_time = PREDICTION_DISTANCE * self.chunk_size / speed;
        let predicted_pos = player_pos + self.smoothed_velocity * prediction_time;
        
        // Get chunks along the path
        let steps = 20;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let pos = player_pos.lerp(predicted_pos, t);
            
            let chunk_x = (pos.x / self.chunk_size).floor() as i32;
            let chunk_z = (pos.z / self.chunk_size).floor() as i32;
            
            // Add chunk and neighbors
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let cx = chunk_x + dx;
                    let cz = chunk_z + dz;
                    
                    if self.predicted_chunks.insert((cx, cz)) {
                        // Calculate priority (closer = higher priority)
                        let chunk_center_x = (cx as f32 + 0.5) * self.chunk_size;
                        let chunk_center_z = (cz as f32 + 0.5) * self.chunk_size;
                        let dist = ((chunk_center_x - player_pos.x).powi(2) + 
                                   (chunk_center_z - player_pos.z).powi(2)).sqrt();
                        
                        let priority = 1.0 / (1.0 + dist);
                        self.priority_queue.push(((cx, cz), priority));
                    }
                }
            }
        }
        
        // Sort by priority (highest first)
        self.priority_queue.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }
    
    pub fn get_priority_chunks(&self, max_chunks: usize) -> Vec<(i32, i32)> {
        self.priority_queue
            .iter()
            .take(max_chunks)
            .map(|(chunk, _)| *chunk)
            .collect()
    }
    
    pub fn get_movement_direction(&self) -> Vec3 {
        self.smoothed_direction
    }
    
    pub fn get_movement_speed(&self) -> f32 {
        self.smoothed_velocity.length()
    }
    
    pub fn debug_info(&self) -> String {
        format!(
            "Speed: {:.1}, Direction: ({:.2}, {:.2}, {:.2}), Predicted chunks: {}",
            self.smoothed_velocity.length(),
            self.smoothed_direction.x,
            self.smoothed_direction.y,
            self.smoothed_direction.z,
            self.predicted_chunks.len()
        )
    }
}