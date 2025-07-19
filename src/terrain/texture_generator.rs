use miniquad::*;
use glam::Vec2;
use crate::biome::{BiomeMap, Biome};
use crate::terrain::height_at;

pub struct TextureGenerator;

impl TextureGenerator {
    /// Generate height and biome textures for terrain
    pub fn create_terrain_textures(
        ctx: &mut dyn RenderingBackend, 
        size: u32, 
        terrain_scale: f32, 
        biome_map: &BiomeMap
    ) -> (TextureId, TextureId, Vec<u8>, Vec<u8>) {
        let mut height_pixels = vec![0u8; (size * size * 4) as usize];
        let mut biome_pixels = vec![0u8; (size * size * 4) as usize];
        
        let scale = terrain_scale / size as f32;
        let offset = terrain_scale / 2.0;
        
        // Generate texture data
        for y in 0..size {
            for x in 0..size {
                let world_x = x as f32 * scale - offset;
                let world_z = y as f32 * scale - offset;
                let idx = ((y * size + x) * 4) as usize;
                
                // Get biome weights
                let biome_weights = biome_map.get_biome_weights(world_x, world_z);
                
                // Calculate height
                let height = height_at(world_x, world_z);
                
                // Encode height as 16-bit value across RG channels
                let height_normalized = ((height + 200.0) / 400.0).clamp(0.0, 1.0);
                let height_u16 = (height_normalized * 65535.0) as u16;
                height_pixels[idx] = (height_u16 >> 8) as u8;
                height_pixels[idx + 1] = (height_u16 & 0xFF) as u8;
                height_pixels[idx + 2] = 0;
                height_pixels[idx + 3] = 255;
                
                // Store biome weights in RGBA channels
                let weights = Self::pack_biome_weights(&biome_weights);
                biome_pixels[idx] = weights.0;
                biome_pixels[idx + 1] = weights.1;
                biome_pixels[idx + 2] = weights.2;
                biome_pixels[idx + 3] = weights.3;
            }
        }
        
        // Create textures
        let height_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&height_pixels),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: size,
                height: size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        let biome_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&biome_pixels),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                width: size,
                height: size,
                allocate_mipmaps: false,
                sample_count: 1,
                kind: TextureKind::Texture2D,
            }
        );
        
        (height_texture, biome_texture, height_pixels, biome_pixels)
    }
    
    /// Pack biome weights into RGBA channels
    fn pack_biome_weights(biome_weights: &[(Biome, f32)]) -> (u8, u8, u8, u8) {
        // Get the two strongest biomes
        let primary = biome_weights.get(0).unwrap_or(&(Biome::Plains, 1.0));
        let secondary = biome_weights.get(1).unwrap_or(&(Biome::Plains, 0.0));
        
        // Encode biomes and weights
        let biome1_id = Self::biome_to_id(&primary.0);
        let biome2_id = Self::biome_to_id(&secondary.0);
        let weight1 = (primary.1 * 255.0) as u8;
        let weight2 = (secondary.1 * 255.0) as u8;
        
        (biome1_id, weight1, biome2_id, weight2)
    }
    
    /// Convert biome to ID for texture encoding
    fn biome_to_id(biome: &Biome) -> u8 {
        match biome {
            Biome::Plains => 0,
            Biome::Desert => 1,
            Biome::Arctic => 2,
            Biome::Volcanic => 3,
            Biome::Crystalline => 4,
            Biome::Floating => 5,
            Biome::Swamp => 6,
            Biome::Canyon => 7,
            Biome::Plateau => 8,
            Biome::Mountains => 9,
            Biome::Badlands => 10,
            Biome::Caverns => 11,
        }
    }
    
    /// Create textures from cached data
    pub fn create_from_cache(
        ctx: &mut dyn RenderingBackend,
        height_data: Vec<u8>,
        biome_data: Vec<u8>,
        texture_size: u32
    ) -> (TextureId, TextureId) {
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
        
        (height_texture, biome_texture)
    }
}