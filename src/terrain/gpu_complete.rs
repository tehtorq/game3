use miniquad::*;
use glam::{Vec3, Mat4};
use crate::vertex::Vertex;
use crate::shader::UniformsTerrainGPU;
use super::cache::TerrainCache;
use super::mesh::Terrain;

// Complete GPU terrain with texture support
pub struct TerrainGPUComplete {
    grid_size: usize,
    grid_spacing: f32,
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
    height_texture: Option<TextureId>,
    biome_texture: Option<TextureId>,
    terrain_scale: f32,
    terrain_instance: Option<Terrain>,
}

impl TerrainGPUComplete {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        // Create a large grid for GPU terrain
        let grid_size = 1024; // 1024x1024 grid for larger visible area
        let grid_spacing = 80.0; // 80 units between vertices for 81,920 unit total size
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        // Generate grid vertices centered at origin
        let half_size = grid_size as f32 * 0.5;
        for z in 0..=grid_size {
            for x in 0..=grid_size {
                let fx = (x as f32 - half_size) * grid_spacing;
                let fz = (z as f32 - half_size) * grid_spacing;
                
                // Create vertex with barycentric coordinates for wireframe
                // For a grid, we'll use a pattern that creates nice triangulation
                let bary = if (x + z) % 2 == 0 {
                    [1.0, 0.0, 0.0]
                } else {
                    [0.0, 1.0, 0.0]
                };
                
                vertices.push(Vertex::with_barycentric(fx, 0.0, fz, bary[0], bary[1], bary[2]));
            }
        }
        
        // Generate indices for triangles
        let vertex_row_size = grid_size + 1;
        
        for z in 0..grid_size {
            for x in 0..grid_size {
                let idx = (z * vertex_row_size + x) as u32;
                
                // Create two triangles per grid cell
                // First triangle
                indices.push(idx);
                indices.push(idx + 1);
                indices.push(idx + vertex_row_size as u32);
                
                // Second triangle
                indices.push(idx + 1);
                indices.push(idx + vertex_row_size as u32 + 1);
                indices.push(idx + vertex_row_size as u32);
            }
        }
        
        println!("GPU Complete terrain: {} vertices, {} triangles", vertices.len(), indices.len() / 3);
        
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
        
        // Calculate terrain scale to match the grid
        let terrain_scale = grid_size as f32 * grid_spacing;
        
        Self {
            grid_size,
            grid_spacing,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as i32,
            height_texture: None,
            biome_texture: None,
            terrain_scale,
            terrain_instance: None,
        }
    }
    
    pub fn load_or_generate_textures(&mut self, ctx: &mut dyn RenderingBackend, cache_name: Option<&str>) {
        // Try to load from cache first
        if let Some(name) = cache_name {
            match TerrainCache::load_terrain_textures(name) {
                Ok((height_data, biome_data, texture_size)) => {
                    println!("Loaded terrain textures from cache: {}x{}", texture_size, texture_size);
                    
                    // Create textures from cached data
                    let height_texture = ctx.new_texture(
                        TextureAccess::Static,
                        TextureSource::Bytes(&height_data),
                        TextureParams {
                            format: TextureFormat::RGBA8,
                            wrap: TextureWrap::Clamp,
                            min_filter: FilterMode::Linear,
                            mag_filter: FilterMode::Linear,
                            mipmap_filter: MipmapFilterMode::None,
                            width: texture_size,
                            height: texture_size,
                            allocate_mipmaps: false,
                            sample_count: 1,
                            kind: TextureKind::Texture2D,
                        }
                    );
                    
                    let biome_texture = ctx.new_texture(
                        TextureAccess::Static,
                        TextureSource::Bytes(&biome_data),
                        TextureParams {
                            format: TextureFormat::RGBA8,
                            wrap: TextureWrap::Clamp,
                            min_filter: FilterMode::Linear,
                            mag_filter: FilterMode::Linear,
                            mipmap_filter: MipmapFilterMode::None,
                            width: texture_size,
                            height: texture_size,
                            allocate_mipmaps: false,
                            sample_count: 1,
                            kind: TextureKind::Texture2D,
                        }
                    );
                    
                    self.height_texture = Some(height_texture);
                    self.biome_texture = Some(biome_texture);
                    return;
                }
                Err(e) => {
                    println!("Could not load terrain from cache: {}", e);
                }
            }
        }
        
        // Generate new terrain if cache load failed
        println!("Generating new terrain textures...");
        let view_distance = 40; // Matches the chunk-based terrain view distance
        let terrain = Terrain::new_with_seed_and_save(ctx, view_distance, 12345, cache_name);
        
        self.height_texture = Some(terrain.height_texture());
        self.biome_texture = Some(terrain.biome_texture());
        self.terrain_scale = terrain.terrain_scale();
        self.terrain_instance = Some(terrain);
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, pipeline: &Pipeline, mvp: Mat4, base_color: [f32; 3], player_pos: Vec3, elapsed_time: f32) -> i32 {
        ctx.apply_pipeline(pipeline);
        
        // Create bindings with textures if available
        let mut images = vec![];
        if let Some(height_tex) = self.height_texture {
            images.push(height_tex);
        }
        if let Some(biome_tex) = self.biome_texture {
            images.push(biome_tex);
        }
        
        let bindings = Bindings {
            vertex_buffers: vec![self.vertex_buffer],
            index_buffer: self.index_buffer,
            images,
        };
        
        ctx.apply_bindings(&bindings);
        
        // Grid follows player for infinite terrain
        let chunk_offset = [
            (player_pos.x / self.grid_spacing).floor() * self.grid_spacing,
            (player_pos.z / self.grid_spacing).floor() * self.grid_spacing
        ];
        
        // Use elapsed time for water animation
        // Use modulo to keep time values reasonable and prevent precision issues
        let time = elapsed_time % 1000.0;
        
        let uniforms = UniformsTerrainGPU::new(
            mvp,
            base_color,
            0.0, // No morphing for single LOD
            chunk_offset,
            self.grid_spacing,
            player_pos,
            time
        );
        
        ctx.apply_uniforms(UniformsSource::table(&uniforms));
        ctx.draw(0, self.index_count, 1);
        
        self.index_count / 3
    }
    
    pub fn get_height_at(&self, x: f32, z: f32) -> f32 {
        // Use the terrain instance if available for height queries
        if let Some(ref terrain) = self.terrain_instance {
            terrain.get_height_at(x, z)
        } else {
            // Fallback to shader-matched implementation
            Terrain::height_at(x, z)
        }
    }
}