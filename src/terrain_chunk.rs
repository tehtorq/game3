use crate::vertex::Vertex;

#[derive(Clone, Copy, Debug)]
pub enum LodLevel {
    High,   // 8x8 grid per chunk (64 cells)
    Medium, // 4x4 grid per chunk (16 cells)
    Low,    // 2x2 grid per chunk (4 cells)
}

impl LodLevel {
    pub fn grid_size(&self) -> usize {
        match self {
            LodLevel::High => 8,
            LodLevel::Medium => 4,
            LodLevel::Low => 2,
        }
    }
    
    pub fn from_distance(distance: f32) -> Self {
        if distance < 800.0 {
            LodLevel::High
        } else if distance < 1600.0 {
            LodLevel::Medium
        } else {
            LodLevel::Low
        }
    }
}

pub struct TerrainChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub lod: LodLevel,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl TerrainChunk {
    pub fn new(chunk_x: i32, chunk_z: i32, lod: LodLevel, chunk_size: f32) -> Self {
        let mut chunk = Self {
            chunk_x,
            chunk_z,
            lod,
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        chunk.generate_vertices(chunk_size);
        chunk
    }
    
    pub fn generate_vertices(&mut self, chunk_size: f32) {
        let start_time = miniquad::date::now();
        
        let grid_size = self.lod.grid_size();
        let cell_size = chunk_size / grid_size as f32;
        
        let base_x = self.chunk_x as f32 * chunk_size;
        let base_z = self.chunk_z as f32 * chunk_size;
        
        // Generate vertices
        self.vertices.clear();
        self.indices.clear();
        
        // Create main grid vertices
        for z in 0..=grid_size {
            for x in 0..=grid_size {
                // Calculate exact world position with proper edge alignment
                let world_x = if x == grid_size {
                    // Right edge - align exactly with next chunk's left edge
                    (self.chunk_x + 1) as f32 * chunk_size
                } else if x == 0 {
                    // Left edge - align exactly with chunk boundary
                    self.chunk_x as f32 * chunk_size
                } else {
                    // Interior vertices
                    base_x + (x as f32 * cell_size)
                };
                
                let world_z = if z == grid_size {
                    // Far edge - align exactly with next chunk's near edge
                    (self.chunk_z + 1) as f32 * chunk_size
                } else if z == 0 {
                    // Near edge - align exactly with chunk boundary
                    self.chunk_z as f32 * chunk_size
                } else {
                    // Interior vertices
                    base_z + (z as f32 * cell_size)
                };
                
                // Calculate height using our terrain generation function
                let height = crate::terrain_generation::height_at(world_x, world_z);
                
                self.vertices.push(Vertex::new(
                    world_x,
                    height,
                    world_z,
                ));
            }
        }
        
        // Generate indices for main grid
        let vertex_row_size = grid_size + 1;
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let idx = (z * vertex_row_size + x) as u32;
                
                // First triangle
                self.indices.push(idx);
                self.indices.push(idx + 1);
                self.indices.push(idx + vertex_row_size as u32);
                
                // Second triangle
                self.indices.push(idx + 1);
                self.indices.push(idx + vertex_row_size as u32 + 1);
                self.indices.push(idx + vertex_row_size as u32);
            }
        }
        
        let gen_time = miniquad::date::now() - start_time;
        if gen_time > 0.01 { // Only log very slow chunk generations (> 10ms)
            println!("WARNING: Slow chunk ({},{}) generation: {:.1}ms", 
                self.chunk_x, self.chunk_z, gen_time * 1000.0);
        }
    }
    
    pub fn update_lod(&mut self, new_lod: LodLevel, chunk_size: f32) {
        if self.lod as u8 != new_lod as u8 {
            self.lod = new_lod;
            self.generate_vertices(chunk_size);
        }
    }
}