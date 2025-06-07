#[repr(C)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub bx: f32,
    pub by: f32,
    pub bz: f32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, bx: 0.0, by: 0.0, bz: 0.0 }
    }
    
    pub fn with_barycentric(x: f32, y: f32, z: f32, bx: f32, by: f32, bz: f32) -> Self {
        Self { x, y, z, bx, by, bz }
    }
}