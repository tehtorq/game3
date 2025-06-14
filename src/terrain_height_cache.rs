use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

// Global terrain height cache to avoid recalculating heights
lazy_static! {
    static ref HEIGHT_CACHE: Mutex<TerrainHeightCache> = Mutex::new(TerrainHeightCache::new());
}

pub struct TerrainHeightCache {
    cache: HashMap<(i32, i32), f32>,
    hits: u64,
    misses: u64,
    frame_count: u64,
}

impl TerrainHeightCache {
    fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(10000),
            hits: 0,
            misses: 0,
            frame_count: 0,
        }
    }
    
    pub fn get_or_calculate<F>(&mut self, x: f32, z: f32, calculate: F) -> f32 
    where F: FnOnce(f32, f32) -> f32
    {
        // Quantize to 1-unit grid for caching
        let key = ((x.floor() as i32), (z.floor() as i32));
        
        if let Some(&height) = self.cache.get(&key) {
            self.hits += 1;
            height
        } else {
            self.misses += 1;
            let height = calculate(x, z);
            self.cache.insert(key, height);
            
            // Clean cache periodically
            if self.cache.len() > 50000 {
                self.cache.clear();
                println!("Terrain height cache cleared (was {} entries)", self.cache.len());
            }
            
            height
        }
    }
    
    pub fn frame_update(&mut self) {
        self.frame_count += 1;
        if self.frame_count % 300 == 0 {  // Every 5 seconds at 60fps
            let total = self.hits + self.misses;
            if total > 0 {
                let hit_rate = (self.hits as f32 / total as f32) * 100.0;
                println!("Terrain height cache: {:.1}% hit rate ({} hits, {} misses, {} entries)", 
                    hit_rate, self.hits, self.misses, self.cache.len());
            }
            self.hits = 0;
            self.misses = 0;
        }
    }
}

// Public interface
pub fn cached_height_at(x: f32, z: f32) -> f32 {
    HEIGHT_CACHE.lock().unwrap().get_or_calculate(x, z, |x, z| {
        crate::terrain_generation::height_at(x, z)
    })
}

pub fn update_cache_stats() {
    HEIGHT_CACHE.lock().unwrap().frame_update();
}