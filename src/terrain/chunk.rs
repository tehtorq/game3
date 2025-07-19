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
        if distance < 400.0 {          // Much smaller high detail area
            LodLevel::High
        } else if distance < 1200.0 {   // Reduced medium area
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
        let grid_size = self.lod.grid_size();
        let cell_size = chunk_size / grid_size as f32;
        
        // Generate vertices
        self.vertices.clear();
        self.indices.clear();
        
        // Create grid vertices - just positions, GPU will calculate heights
        for z in 0..=grid_size {
            for x in 0..=grid_size {
                // Local position within chunk (0 to chunk_size)
                let local_x = x as f32 * cell_size;
                let local_z = z as f32 * cell_size;
                
                // Store local position, GPU will add chunk offset
                self.vertices.push(Vertex::new(
                    local_x,
                    0.0,  // Height will be calculated on GPU
                    local_z,
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
        
        
        // Remove per-chunk logging to reduce console spam
    }
    
    pub fn update_lod(&mut self, new_lod: LodLevel, chunk_size: f32) {
        if self.lod as u8 != new_lod as u8 {
            self.lod = new_lod;
            self.generate_vertices(chunk_size);
        }
    }
}