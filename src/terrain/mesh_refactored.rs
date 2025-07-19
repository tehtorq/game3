use miniquad::*;
use glam::Vec2;
use crate::biome::{BiomeMap, Biome};
use crate::constants::*;
use super::mesh_builder::MeshBuilder;
use super::texture_generator::TextureGenerator;

pub struct Terrain {
    base_mesh_vertex_buffer: BufferId,
    base_mesh_index_buffer: BufferId,
    instance_buffer: BufferId,
    height_texture: TextureId,
    biome_texture: TextureId,
    index_count: i32,
    instance_count: i32,
    terrain_scale: f32,
    biome_map: BiomeMap,
    view_distance: i32,
}

impl Terrain {
    pub fn new(ctx: &mut dyn RenderingBackend, view_distance: i32) -> Self {
        Self::new_with_seed(ctx, view_distance, 12345)
    }
    
    pub fn new_from_cache(
        ctx: &mut dyn RenderingBackend, 
        view_distance: i32, 
        height_data: Vec<u8>, 
        biome_data: Vec<u8>,
        texture_size: u32
    ) -> Self {
        // Create base mesh
        let (vertices, indices) = MeshBuilder::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance data
        let instance_data = MeshBuilder::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2;
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        
        // Create textures from cached data
        let (height_texture, biome_texture) = TextureGenerator::create_from_cache(
            ctx, height_data, biome_data, texture_size
        );
        
        // Create a dummy biome map (not needed when loading from cache)
        let biome_map = BiomeMap::new(terrain_scale, 0);
        
        println!("Loaded terrain from cache");
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    pub fn new_with_seed_and_save(
        ctx: &mut dyn RenderingBackend, 
        view_distance: i32, 
        seed: u32,
        save_path: &str
    ) -> Self {
        let texture_size = 2048;
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        
        // Create biome map
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Generate terrain data
        let (height_texture, biome_texture, height_pixels, biome_pixels) = 
            TextureGenerator::create_terrain_textures(ctx, texture_size, terrain_scale, &biome_map);
        
        // Save to files
        if let Err(e) = std::fs::write(format!("{}_height.bin", save_path), &height_pixels) {
            eprintln!("Failed to save height data: {}", e);
        }
        if let Err(e) = std::fs::write(format!("{}_biome.bin", save_path), &biome_pixels) {
            eprintln!("Failed to save biome data: {}", e);
        }
        if let Err(e) = std::fs::write(format!("{}_info.txt", save_path), format!("{}", texture_size)) {
            eprintln!("Failed to save info: {}", e);
        }
        
        println!("Saved terrain data to {}", save_path);
        
        // Create mesh
        let (vertices, indices) = MeshBuilder::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance buffer
        let instance_data = MeshBuilder::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2;
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    pub fn new_with_seed(ctx: &mut dyn RenderingBackend, view_distance: i32, seed: u32) -> Self {
        let texture_size = 2048;
        let terrain_scale = (view_distance as f32 * 2.0 + 1.0) * CHUNK_SIZE;
        
        // Create biome map
        let biome_map = BiomeMap::new(terrain_scale, seed);
        
        // Generate terrain textures
        let (height_texture, biome_texture, _, _) = 
            TextureGenerator::create_terrain_textures(ctx, texture_size, terrain_scale, &biome_map);
        
        // Create mesh buffers
        let (vertices, indices) = MeshBuilder::create_base_mesh();
        
        let base_mesh_vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices)
        );
        
        let base_mesh_index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices)
        );
        
        // Create instance buffer
        let instance_data = MeshBuilder::create_instance_data(view_distance);
        let instance_count = instance_data.len() as i32 / 2;
        
        let instance_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Stream,
            BufferSource::slice(&instance_data)
        );
        
        Self {
            base_mesh_vertex_buffer,
            base_mesh_index_buffer,
            instance_buffer,
            height_texture,
            biome_texture,
            index_count: indices.len() as i32,
            instance_count,
            terrain_scale,
            biome_map,
            view_distance,
        }
    }
    
    pub fn update_for_player_position(
        &mut self, 
        ctx: &mut dyn RenderingBackend, 
        player_x: f32, 
        player_z: f32, 
        player_rotation: f32
    ) -> i32 {
        let (instance_data, visible_count) = MeshBuilder::create_culled_instance_data(
            self.view_distance, player_x, player_z, player_rotation
        );
        
        ctx.buffer_update(self.instance_buffer, BufferSource::slice(&instance_data));
        self.instance_count = visible_count;
        
        visible_count
    }
    
    // Getters
    pub fn base_vertex_buffer(&self) -> BufferId {
        self.base_mesh_vertex_buffer
    }
    
    pub fn instance_buffer(&self) -> BufferId {
        self.instance_buffer
    }
    
    pub fn index_buffer(&self) -> BufferId {
        self.base_mesh_index_buffer
    }
    
    pub fn height_texture(&self) -> TextureId {
        self.height_texture
    }
    
    pub fn biome_texture(&self) -> TextureId {
        self.biome_texture
    }
    
    pub fn index_count(&self) -> i32 {
        self.index_count
    }
    
    pub fn instance_count(&self) -> i32 {
        self.instance_count
    }
    
    pub fn terrain_scale(&self) -> f32 {
        self.terrain_scale
    }
    
    pub fn get_height_at(&self, x: f32, z: f32) -> f32 {
        crate::terrain::height_at(x, z)
    }
    
    pub fn height_at(x: f32, z: f32) -> f32 {
        crate::terrain::height_at(x, z)
    }
    
    pub fn get_biome_at(&self, x: f32, z: f32) -> Biome {
        self.biome_map.get_biome_at(x, z)
    }
    
    pub fn get_texture_data(&self) -> (Vec<u8>, Vec<u8>, u32) {
        // This would need to be implemented to extract data from GPU textures
        // For now, return empty data
        (vec![], vec![], 0)
    }
}