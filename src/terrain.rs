use miniquad::*;
use crate::vertex::Vertex;

// Approximates the shader's biome-based terrain generation for CPU-side collision detection
pub fn terrain_height_fallback(x: f32, z: f32) -> f32 {
    // Multi-scale noise for biome distribution (matching shader)
    let noise1 = (x * 0.0003).sin() * (z * 0.0003).cos();
    let noise2 = (x * 0.0007 + 1.3).sin() * (z * 0.0006 - 0.7).sin();
    let noise3 = (x * 0.0013 - 2.1).cos() * (z * 0.0011 + 1.9).sin();
    
    // Combine noises for complex patterns
    let biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Add local variation
    let local_var = (x * 0.01).sin() * (z * 0.01).cos() * 0.1;
    let biome_noise = biome_noise + local_var;
    
    // Calculate base biome height
    let biome_height = if biome_noise < -0.7 {
        // Canyon
        let base = (x * 0.0008).sin() * (z * 0.0008 * 0.8).cos() * 60.0;
        let secondary = (x * 0.0004 + 1.0).sin() * (z * 0.0004 * 1.2).cos() * 40.0;
        let valley = ((x * 0.001).sin().max(0.0)) * 30.0 + ((z * 0.0008).cos().max(0.0)) * 20.0;
        let detail = (x * 0.005).sin() * (z * 0.004).cos() * 10.0;
        base + secondary - valley + detail
    } else if biome_noise < -0.3 {
        // Badlands
        let hills = (x * 0.0004).sin() * (z * 0.0004 * 0.8).cos() * 50.0;
        let erosion = ((x * 0.0008 + 0.7).cos() * (z * 0.0008 * 1.2).sin() * 0.5 + 0.5).max(0.3).min(0.7) * 30.0;
        let mesa = ((x * 0.0003).sin() * (z * 0.00025).cos() * 0.5 + 0.5).max(0.2).min(0.6) * 40.0;
        let texture = (x * 0.002).sin() * (z * 0.0018).cos() * 15.0;
        hills + erosion + mesa + texture
    } else if biome_noise < 0.1 {
        // Plains
        let h1 = (x * 0.0002).sin() * (z * 0.0002 * 0.8).cos() * 25.0;
        let h2 = (x * 0.0004 + 1.0).sin() * (z * 0.0004 * 1.2).cos() * 15.0;
        let detail = (x * 0.001).sin() * (z * 0.0008).sin() * 8.0;
        h1 + h2 + detail
    } else if biome_noise < 0.5 {
        // Crystalline
        let cluster1 = ((x * 0.002).sin() * (z * 0.002).cos() * 0.5 + 0.5).max(0.3).min(0.7) * 50.0;
        let cluster2 = ((x * 0.004 + 1.0).cos() * (z * 0.004 - 0.5).sin() * 0.5 + 0.5).max(0.4).min(0.6) * 35.0;
        let texture = ((x * 0.01).sin() * (z * 0.008).cos()).abs() * 20.0;
        let base = (x * 0.0005).sin() * (z * 0.0004).cos() * 25.0;
        cluster1 + cluster2 + texture + base
    } else {
        // Mountain
        let dist1 = ((x * 0.0002).sin().powi(2) + (z * 0.0002).cos().powi(2)).sqrt();
        let dist2 = ((x * 0.0004 + 1.0).sin().powi(2) + (z * 0.0004 - 0.5).cos().powi(2)).sqrt();
        let h1 = (-dist1 * dist1 * 2.0).exp() * 120.0;
        let h2 = (-dist2 * dist2 * 3.0).exp() * 80.0;
        let foothills = (x * 0.0008).sin() * (z * 0.0007).cos() * 30.0 + 
                       (x * 0.0012 + 0.5).sin() * (z * 0.001).sin() * 20.0;
        let valley = (x * 0.0003 + z * 0.0002).sin() * 15.0;
        h1 + h2 + foothills + valley
    };
    
    // Add rolling hills (matching shader blending)
    let hills = (x * 0.0001).sin() * (z * 0.00012).cos() * 60.0 +
                (x * 0.00018 + 1.5).sin() * (z * 0.00015 - 0.7).cos() * 40.0 +
                (x * 0.00025 - 0.3).sin() * (z * 0.0003 + 1.2).cos() * 25.0;
    
    // Add gentle undulations
    let undulation = (x * 0.0004).sin() * (z * 0.0004).cos() * 15.0 +
                     (x * 0.0008 + 2.1).sin() * (z * 0.0007 - 1.3).sin() * 10.0;
    
    // Continental scale features
    let continental = (x * 0.00005).sin() * (z * 0.00005).cos() * 30.0;
    
    // Blend everything together (matching shader ratios)
    biome_height * 0.7 + hills * 0.2 + undulation * 0.1 + continental
}

pub struct TerrainChunk {
    pub x_offset: f32,
    pub z_offset: f32,
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
}

impl TerrainChunk {
    pub fn new(x_offset: f32, z_offset: f32, ctx: &mut dyn RenderingBackend) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate static mesh data
        let terrain_y_base = -30.0;
        let grid_size = 8;
        let cell_size = 20.0;
        let half_size = 80.0;
        
        // First, generate the grid positions
        let mut grid_positions = Vec::new();
        for i in 0..=grid_size {
            for j in 0..=grid_size {
                let x = x_offset - half_size + (i as f32) * cell_size;
                let z = z_offset - half_size + (j as f32) * cell_size;
                let y = Self::height_at_static(x, z) + terrain_y_base;
                grid_positions.push((x, y, z));
            }
        }
        
        // Generate triangles with proper barycentric coordinates
        for i in 0..grid_size {
            for j in 0..grid_size {
                let idx = i * (grid_size + 1) + j;
                let base_vertex_idx = vertices.len() as u32;
                
                // Get the four corners of this cell
                let p0 = grid_positions[idx];
                let p1 = grid_positions[idx + 1];
                let p2 = grid_positions[idx + grid_size + 1];
                let p3 = grid_positions[idx + grid_size + 2];
                
                // First triangle (p0, p2, p1)
                vertices.push(Vertex::with_barycentric(p0.0, p0.1, p0.2, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(p2.0, p2.1, p2.2, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(p1.0, p1.1, p1.2, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx);
                indices.push(base_vertex_idx + 1);
                indices.push(base_vertex_idx + 2);
                
                // Second triangle (p1, p2, p3)
                vertices.push(Vertex::with_barycentric(p1.0, p1.1, p1.2, 1.0, 0.0, 0.0));
                vertices.push(Vertex::with_barycentric(p2.0, p2.1, p2.2, 0.0, 1.0, 0.0));
                vertices.push(Vertex::with_barycentric(p3.0, p3.1, p3.2, 0.0, 0.0, 1.0));
                
                indices.push(base_vertex_idx + 3);
                indices.push(base_vertex_idx + 4);
                indices.push(base_vertex_idx + 5);
            }
        }
        
        let index_count = indices.len() as i32;
        
        // Create GPU buffers
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        Self {
            x_offset,
            z_offset,
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
    
    fn height_at_static(x: f32, z: f32) -> f32 {
        let scale1 = 0.002;
        let scale2 = 0.007;
        let scale3 = 0.015;
        let height_scale = 60.0;
        
        let h1 = (x * scale1).sin() * (z * scale1).cos() * height_scale;
        let h2 = (x * scale2 + 100.0).sin() * (z * scale2 + 100.0).sin() * height_scale * 0.5;
        let h3 = (x * scale3 + 200.0).cos() * (z * scale3 + 200.0).cos() * height_scale * 0.25;
        
        h1 + h2 + h3
    }
    
    pub fn vertex_buffer(&self) -> BufferId {
        self.vertex_buffer
    }
    
    pub fn index_buffer(&self) -> BufferId {
        self.index_buffer
    }
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
}

