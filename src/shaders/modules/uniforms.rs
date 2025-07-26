use glam::{Mat4, Vec3};

#[repr(C)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
    pub camera_pos: [f32; 3],
    pub _padding2: f32,
}

impl Uniforms {
    pub fn new(mvp: Mat4, color: [f32; 3]) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
            camera_pos: [0.0, 0.0, 0.0],
            _padding2: 0.0,
        }
    }
    
    pub fn with_camera(mvp: Mat4, color: [f32; 3], camera_pos: Vec3) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
            _padding2: 0.0,
        }
    }
}

#[repr(C)]
pub struct UniformsTerrainGPU {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub morph_factor: f32,
    pub chunk_offset: [f32; 2],
    pub lod_scale: f32,
    pub _padding: f32,
    pub camera_pos: [f32; 3],
    pub time: f32,
}

impl UniformsTerrainGPU {
    pub fn new(mvp: Mat4, color: [f32; 3], morph_factor: f32, chunk_offset: [f32; 2], lod_scale: f32, camera_pos: Vec3, time: f32) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            morph_factor,
            chunk_offset,
            lod_scale,
            _padding: 0.0,
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
            time,
        }
    }
}

#[repr(C)]
pub struct UniformsTerrain {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
    pub terrain_scale: f32,
    pub terrain_y_base: f32,
    pub _padding2: [f32; 2],
}

impl UniformsTerrain {
    pub fn new(mvp: Mat4, color: [f32; 3], terrain_scale: f32, terrain_y_base: f32) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
            terrain_scale,
            terrain_y_base,
            _padding2: [0.0, 0.0],
        }
    }
}