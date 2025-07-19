use std::fs;
use std::path::Path;
use image::{ImageBuffer, RgbaImage};

pub struct TerrainCache;

impl TerrainCache {
    pub fn save_terrain_textures(
        name: &str,
        height_data: &[u8],
        biome_data: &[u8],
        size: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create terrain cache directory if it doesn't exist
        let cache_dir = Path::new("terrain_cache");
        if !cache_dir.exists() {
            fs::create_dir(cache_dir)?;
        }
        
        // Save height texture
        let height_path = cache_dir.join(format!("{}_height.png", name));
        let height_image: RgbaImage = ImageBuffer::from_raw(size, size, height_data.to_vec())
            .ok_or("Failed to create height image")?;
        height_image.save(&height_path)?;
        println!("Saved height texture to: {}", height_path.display());
        
        // Save biome texture
        let biome_path = cache_dir.join(format!("{}_biome.png", name));
        let biome_image: RgbaImage = ImageBuffer::from_raw(size, size, biome_data.to_vec())
            .ok_or("Failed to create biome image")?;
        biome_image.save(&biome_path)?;
        println!("Saved biome texture to: {}", biome_path.display());
        
        // Save metadata
        let meta_path = cache_dir.join(format!("{}_meta.txt", name));
        let metadata = format!("size={}\nversion=1.0\n", size);
        fs::write(&meta_path, metadata)?;
        
        Ok(())
    }
    
    pub fn load_terrain_textures(
        name: &str,
    ) -> Result<(Vec<u8>, Vec<u8>, u32), Box<dyn std::error::Error>> {
        let cache_dir = Path::new("terrain_cache");
        
        // Load metadata
        let meta_path = cache_dir.join(format!("{}_meta.txt", name));
        let metadata = fs::read_to_string(&meta_path)?;
        let mut size = 0u32;
        for line in metadata.lines() {
            if let Some(size_str) = line.strip_prefix("size=") {
                size = size_str.parse()?;
            }
        }
        
        // Load height texture
        let height_path = cache_dir.join(format!("{}_height.png", name));
        let height_image = image::open(&height_path)?;
        let height_rgba = height_image.to_rgba8();
        let height_data = height_rgba.into_raw();
        
        // Load biome texture
        let biome_path = cache_dir.join(format!("{}_biome.png", name));
        let biome_image = image::open(&biome_path)?;
        let biome_rgba = biome_image.to_rgba8();
        let biome_data = biome_rgba.into_raw();
        
        println!("Loaded terrain '{}' from cache", name);
        
        Ok((height_data, biome_data, size))
    }
    
    pub fn exists(name: &str) -> bool {
        let cache_dir = Path::new("terrain_cache");
        let height_path = cache_dir.join(format!("{}_height.png", name));
        let biome_path = cache_dir.join(format!("{}_biome.png", name));
        let meta_path = cache_dir.join(format!("{}_meta.txt", name));
        
        height_path.exists() && biome_path.exists() && meta_path.exists()
    }
    
    pub fn list_cached_terrains() -> Vec<String> {
        let cache_dir = Path::new("terrain_cache");
        let mut terrains = Vec::new();
        
        if let Ok(entries) = fs::read_dir(cache_dir) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with("_meta.txt") {
                        let name = filename.trim_end_matches("_meta.txt");
                        terrains.push(name.to_string());
                    }
                }
            }
        }
        
        terrains.sort();
        terrains
    }
}