// Terrain module - consolidates all terrain-related functionality
// This module provides a single source of truth for terrain generation

pub mod biome_heights;
pub mod generation;
pub mod mesh;
pub mod mesh_builder;
pub mod texture_generator;
pub mod gpu_complete;
pub mod gpu_rings;
pub mod cache;
pub mod chunk;
pub mod batch;
pub mod lod_blend;
pub mod predictive;
pub mod clipmap;
pub mod height_cache;
pub mod shader_gen;

// Re-export commonly used items
pub use generation::height_at;
pub use mesh::Terrain;
pub use gpu_complete::TerrainGPUComplete;
pub use gpu_rings::TerrainGPURings;
pub use cache::TerrainCache;
pub use chunk::TerrainChunk;

// Constants for terrain generation
pub const CHUNK_SIZE: f32 = 1024.0;
pub const TERRAIN_SCALE: f32 = 10000.0;
pub const MAX_HEIGHT: f32 = 500.0;
pub const WATER_LEVEL: f32 = 0.0;