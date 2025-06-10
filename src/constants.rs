// Game-wide constants for easy configuration

// === TERRAIN CONSTANTS ===

/// View distance in chunks. Each chunk is 160x160 units.
/// This determines how far the player can see terrain.
/// Total chunks rendered = (VIEW_DISTANCE * 2 + 1)^2
pub const VIEW_DISTANCE: i32 = 100;

/// Size of each terrain chunk in world units
pub const CHUNK_SIZE: f32 = 160.0;

/// Number of grid cells per chunk (resolution of terrain mesh)
/// Higher values give more detailed terrain but use more vertices
pub const CHUNK_GRID_SIZE: usize = 8;

/// Size of the terrain texture (width and height)
/// Higher values give smoother terrain but take longer to generate
pub const TERRAIN_TEXTURE_SIZE: u32 = 512;

// === CAMERA CONSTANTS ===

/// Field of view for the camera in degrees
pub const CAMERA_FOV: f32 = 60.0;

/// Near clipping plane distance
pub const CAMERA_NEAR: f32 = 0.1;

/// Far clipping plane distance
pub const CAMERA_FAR: f32 = 10000.0;

/// Default camera distance from player
pub const CAMERA_DISTANCE: f32 = 150.0;

/// Camera height offset from player
pub const CAMERA_HEIGHT: f32 = 80.0;

/// Camera look-ahead distance
pub const CAMERA_LOOK_AHEAD: f32 = 200.0;

// === PLAYER CONSTANTS ===

/// Player movement speed
pub const PLAYER_SPEED: f32 = 200.0;

/// Player boost speed multiplier
pub const PLAYER_BOOST_MULTIPLIER: f32 = 2.0;

/// Player rotation speed (radians per second)
pub const PLAYER_ROTATION_SPEED: f32 = 3.0;

/// Player starting health
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

/// Player starting shield
pub const PLAYER_MAX_SHIELD: f32 = 100.0;

// === ENEMY CONSTANTS ===

/// Maximum number of enemies at once
pub const MAX_ENEMIES: usize = 1000;

/// Enemy spawn distance from player
pub const ENEMY_SPAWN_DISTANCE: f32 = 2000.0;

/// Enemy detection range
pub const ENEMY_DETECTION_RANGE: f32 = 1500.0;

/// Enemy attack range
pub const ENEMY_ATTACK_RANGE: f32 = 800.0;

// === GAME WORLD CONSTANTS ===

/// Total size of the game world (square)
pub const WORLD_SIZE: f32 = 100000.0;

/// Height range for terrain generation
pub const TERRAIN_MIN_HEIGHT: f32 = -500.0;
pub const TERRAIN_MAX_HEIGHT: f32 = 500.0;

// === RENDERING CONSTANTS ===

/// Maximum vertices in dynamic vertex buffer
pub const MAX_VERTICES: usize = 2000000;

/// Maximum indices in dynamic index buffer
pub const MAX_INDICES: usize = 4000000;

/// Default window width (before fullscreen)
pub const WINDOW_WIDTH: i32 = 1200;

/// Default window height (before fullscreen)
pub const WINDOW_HEIGHT: i32 = 900;

// === WAVE CONSTANTS ===

/// Time between enemy waves in seconds
pub const WAVE_INTERVAL: f32 = 60.0;

/// Number of enemies in first wave
pub const WAVE_BASE_ENEMIES: i32 = 5;

/// Enemy increase per wave
pub const WAVE_ENEMY_INCREMENT: i32 = 2;

// === DEBUG CONSTANTS ===

/// Show FPS counter
pub const SHOW_FPS: bool = true;

/// Show debug cross at origin
pub const SHOW_DEBUG_CROSS: bool = true;

/// Debug cross size
pub const DEBUG_CROSS_SIZE: f32 = 100.0;