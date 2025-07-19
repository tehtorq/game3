use crate::vertex::Vertex;
use crate::constants::*;

pub struct MeshBuilder;

impl MeshBuilder {
    /// Create base mesh (single chunk template)
    pub fn create_base_mesh() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let subdivisions = 32;
        let chunk_size = CHUNK_SIZE;
        let step = chunk_size / subdivisions as f32;
        
        // Generate vertex grid
        for z in 0..=subdivisions {
            for x in 0..=subdivisions {
                let px = x as f32 * step;
                let pz = z as f32 * step;
                
                vertices.push(Vertex::new(px, 0.0, pz));
            }
        }
        
        // Generate indices for triangle strips
        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let tl = z * (subdivisions + 1) + x;
                let tr = tl + 1;
                let bl = tl + subdivisions + 1;
                let br = bl + 1;
                
                // First triangle
                indices.push(tl as u32);
                indices.push(bl as u32);
                indices.push(tr as u32);
                
                // Second triangle
                indices.push(tr as u32);
                indices.push(bl as u32);
                indices.push(br as u32);
            }
        }
        
        (vertices, indices)
    }
    
    /// Create instance data (chunk positions)
    pub fn create_instance_data(view_distance: i32) -> Vec<f32> {
        let mut instance_data = Vec::new();
        let chunk_size = CHUNK_SIZE;
        
        for z in -view_distance..=view_distance {
            for x in -view_distance..=view_distance {
                // Chunk offset
                instance_data.push(x as f32 * chunk_size);
                instance_data.push(z as f32 * chunk_size);
            }
        }
        
        instance_data
    }
    
    /// Create frustum culled instance data based on player position and rotation
    pub fn create_culled_instance_data(view_distance: i32, player_x: f32, player_z: f32, player_rotation: f32) -> (Vec<f32>, i32) {
        let mut instance_data = Vec::new();
        let chunk_size = CHUNK_SIZE;
        let mut visible_count = 0;
        
        // Calculate player's chunk position
        let player_chunk_x = (player_x / chunk_size).floor() as i32;
        let player_chunk_z = (player_z / chunk_size).floor() as i32;
        
        // Calculate view frustum
        let fov = 90.0_f32.to_radians();
        let half_fov = fov / 2.0;
        let forward_angle = player_rotation;
        
        for z in -view_distance..=view_distance {
            for x in -view_distance..=view_distance {
                let chunk_x = player_chunk_x + x;
                let chunk_z = player_chunk_z + z;
                
                let chunk_center_x = chunk_x as f32 * chunk_size + chunk_size * 0.5;
                let chunk_center_z = chunk_z as f32 * chunk_size + chunk_size * 0.5;
                
                // Vector from player to chunk center
                let to_chunk_x = chunk_center_x - player_x;
                let to_chunk_z = chunk_center_z - player_z;
                
                // Distance check (circular view distance)
                let distance_sq = to_chunk_x * to_chunk_x + to_chunk_z * to_chunk_z;
                let max_distance = view_distance as f32 * chunk_size * 1.5;
                
                if distance_sq > max_distance * max_distance {
                    continue;
                }
                
                // Frustum check
                if distance_sq > chunk_size * chunk_size {
                    let angle_to_chunk = to_chunk_z.atan2(to_chunk_x);
                    let angle_diff = (angle_to_chunk - forward_angle).rem_euclid(std::f32::consts::TAU);
                    let angle_diff = if angle_diff > std::f32::consts::PI { 
                        angle_diff - std::f32::consts::TAU 
                    } else { 
                        angle_diff 
                    };
                    
                    if angle_diff.abs() > half_fov + 0.5 {
                        continue;
                    }
                }
                
                // Chunk is visible
                instance_data.push(chunk_x as f32 * chunk_size);
                instance_data.push(chunk_z as f32 * chunk_size);
                visible_count += 1;
            }
        }
        
        (instance_data, visible_count)
    }
}