use glam::Vec2;

/// Determines which edges of a chunk need LOD blending based on neighboring chunks
#[derive(Debug, Clone, Copy)]
pub struct LodBlendInfo {
    pub blend_north: bool,
    pub blend_south: bool,
    pub blend_east: bool,
    pub blend_west: bool,
}

impl LodBlendInfo {
    pub fn new() -> Self {
        Self {
            blend_north: false,
            blend_south: false,
            blend_east: false,
            blend_west: false,
        }
    }
    
    /// Check if any edge needs blending
    pub fn needs_blending(&self) -> bool {
        self.blend_north || self.blend_south || self.blend_east || self.blend_west
    }
}

/// Blend vertex positions along chunk edges to match lower LOD neighbors
pub fn blend_edge_vertices(
    vertices: &mut Vec<crate::vertex::Vertex>,
    grid_size: usize,
    chunk_size: f32,
    blend_info: &LodBlendInfo,
) {
    let vertex_row_size = grid_size + 1;
    
    // For each edge that needs blending, we need to ensure vertices match
    // the positions that the neighboring chunk would have
    
    // North edge blending (z = 0)
    if blend_info.blend_north {
        // For lower LOD neighbors, we need to match their vertex spacing
        // If this is High (8x8) and neighbor is Medium (4x4), we blend to match
        match grid_size {
            8 => {
                // High LOD chunk - blend to match possible Medium or Low LOD neighbor
                for x in 0..=grid_size {
                    let idx = x;
                    if x % 2 == 1 && x > 0 && x < grid_size {
                        // Odd vertices: average neighbors for smooth transition
                        let left_height = vertices[idx - 1].y;
                        let right_height = vertices[idx + 1].y;
                        vertices[idx].y = (left_height + right_height) * 0.5;
                    }
                }
            }
            4 => {
                // Medium LOD chunk - might need to match Low LOD neighbor
                for x in 0..=grid_size {
                    let idx = x;
                    if x % 2 == 1 && x > 0 && x < grid_size {
                        let left_height = vertices[idx - 1].y;
                        let right_height = vertices[idx + 1].y;
                        vertices[idx].y = (left_height + right_height) * 0.5;
                    }
                }
            }
            _ => {} // Low LOD doesn't need blending
        }
    }
    
    // South edge blending (z = grid_size)
    if blend_info.blend_south {
        match grid_size {
            8 => {
                for x in 0..=grid_size {
                    let idx = grid_size * vertex_row_size + x;
                    if x % 2 == 1 && x > 0 && x < grid_size {
                        let left_height = vertices[idx - 1].y;
                        let right_height = vertices[idx + 1].y;
                        vertices[idx].y = (left_height + right_height) * 0.5;
                    }
                }
            }
            4 => {
                for x in 0..=grid_size {
                    let idx = grid_size * vertex_row_size + x;
                    if x % 2 == 1 && x > 0 && x < grid_size {
                        let left_height = vertices[idx - 1].y;
                        let right_height = vertices[idx + 1].y;
                        vertices[idx].y = (left_height + right_height) * 0.5;
                    }
                }
            }
            _ => {}
        }
    }
    
    // West edge blending (x = 0)
    if blend_info.blend_west {
        match grid_size {
            8 => {
                for z in 0..=grid_size {
                    let idx = z * vertex_row_size;
                    if z % 2 == 1 && z > 0 && z < grid_size {
                        let top_height = vertices[(z - 1) * vertex_row_size].y;
                        let bottom_height = vertices[(z + 1) * vertex_row_size].y;
                        vertices[idx].y = (top_height + bottom_height) * 0.5;
                    }
                }
            }
            4 => {
                for z in 0..=grid_size {
                    let idx = z * vertex_row_size;
                    if z % 2 == 1 && z > 0 && z < grid_size {
                        let top_height = vertices[(z - 1) * vertex_row_size].y;
                        let bottom_height = vertices[(z + 1) * vertex_row_size].y;
                        vertices[idx].y = (top_height + bottom_height) * 0.5;
                    }
                }
            }
            _ => {}
        }
    }
    
    // East edge blending (x = grid_size)
    if blend_info.blend_east {
        match grid_size {
            8 => {
                for z in 0..=grid_size {
                    let idx = z * vertex_row_size + grid_size;
                    if z % 2 == 1 && z > 0 && z < grid_size {
                        let top_height = vertices[(z - 1) * vertex_row_size + grid_size].y;
                        let bottom_height = vertices[(z + 1) * vertex_row_size + grid_size].y;
                        vertices[idx].y = (top_height + bottom_height) * 0.5;
                    }
                }
            }
            4 => {
                for z in 0..=grid_size {
                    let idx = z * vertex_row_size + grid_size;
                    if z % 2 == 1 && z > 0 && z < grid_size {
                        let top_height = vertices[(z - 1) * vertex_row_size + grid_size].y;
                        let bottom_height = vertices[(z + 1) * vertex_row_size + grid_size].y;
                        vertices[idx].y = (top_height + bottom_height) * 0.5;
                    }
                }
            }
            _ => {}
        }
    }
}