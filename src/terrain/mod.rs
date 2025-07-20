// Terrain module - consolidates all terrain-related functionality
// This module provides a single source of truth for terrain generation

pub mod biome_heights;
pub mod generation;
pub mod mesh;
pub mod gpu_complete;
pub mod cache;
pub mod height_cache;

// Re-export commonly used items
pub use generation::{height_at, get_biome_at};
pub use gpu_complete::TerrainGPUComplete;

// Constants for terrain generation
pub const CHUNK_SIZE: f32 = 1024.0;
pub const TERRAIN_SCALE: f32 = 10000.0;
pub const MAX_HEIGHT: f32 = 500.0;
pub const WATER_LEVEL: f32 = 0.0;