use miniquad::*;
use glam::Mat4;

pub const VERTEX: &str = r#"#version 100
attribute vec3 pos;
attribute vec3 barycentric;

uniform mat4 mvp;

varying vec3 v_barycentric;

void main() {
    v_barycentric = barycentric;
    gl_Position = mvp * vec4(pos, 1.0);
}"#;

pub const VERTEX_INSTANCED_TERRAIN: &str = r#"#version 100
precision mediump float;

attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;
uniform sampler2D height_texture;

// Biome height generation functions
float canyon_height(vec2 p) {
    // Lower frequency for wider features
    float scale1 = 0.0008;
    float scale2 = 0.0004;
    
    // Smooth rolling canyon
    float base = sin(p.x * scale1) * cos(p.y * scale1 * 0.8) * 60.0;
    float secondary = sin(p.x * scale2 + 1.0) * cos(p.y * scale2 * 1.2) * 40.0;
    
    // Add some gentle valleys
    float valley = 0.0;
    valley += smoothstep(0.0, 1.0, sin(p.x * 0.001)) * 30.0;
    valley += smoothstep(0.0, 1.0, cos(p.y * 0.0008)) * 20.0;
    
    // Gentle undulations instead of cliffs
    float detail = sin(p.x * 0.005) * cos(p.y * 0.004) * 10.0;
    
    return base + secondary - valley + detail;
}

float plateau_height(vec2 p) {
    // Very low frequency for large, smooth plateaus
    float scale1 = 0.0002;
    float scale2 = 0.0003;
    
    // Smooth raised areas with tapered edges
    float raise1 = smoothstep(0.2, 0.8, sin(p.x * scale1) * cos(p.y * scale1) * 0.5 + 0.5);
    float raise2 = smoothstep(0.3, 0.7, sin(p.x * scale2 + 1.5) * cos(p.y * scale2 - 0.8) * 0.5 + 0.5);
    
    // Gradual height changes
    float h1 = raise1 * 60.0;
    float h2 = raise2 * 40.0;
    
    // Rolling surface
    float surface = sin(p.x * 0.001) * cos(p.y * 0.0008) * 15.0;
    
    // Gentle blend between heights
    float blend = sin(p.x * 0.0001 + p.y * 0.00015) * 0.5 + 0.5;
    
    return mix(h1, h2, blend) + surface;
}

float crystalline_height(vec2 p) {
    // Lower frequency for larger crystal formations
    float scale1 = 0.002;
    float scale2 = 0.004;
    
    // Smooth crystal clusters instead of sharp spikes
    float cluster1 = smoothstep(0.3, 0.7, sin(p.x * scale1) * cos(p.y * scale1) * 0.5 + 0.5);
    float cluster2 = smoothstep(0.4, 0.6, cos(p.x * scale2 + 1.0) * sin(p.y * scale2 - 0.5) * 0.5 + 0.5);
    
    // Varied heights with smooth transitions
    float h1 = cluster1 * 50.0;
    float h2 = cluster2 * 35.0;
    
    // Gentle crystalline texture
    float texture = abs(sin(p.x * 0.01) * cos(p.y * 0.008)) * 20.0;
    
    // Smooth base elevation
    float base = sin(p.x * 0.0005) * cos(p.y * 0.0004) * 25.0;
    
    return h1 + h2 + texture + base;
}

float volcanic_height(vec2 p) {
    // Low frequency for broad volcanic features
    float scale1 = 0.0002;
    float scale2 = 0.0005;
    
    // Smooth volcanic cone with gentle slopes
    float dist = length(vec2(sin(p.x * scale1), cos(p.y * scale1)));
    float cone = smoothstep(1.0, 0.0, dist) * 80.0;
    
    // Rolling lava fields
    float fields = sin(p.x * scale2) * cos(p.y * scale2 * 0.9) * 25.0;
    
    // Gentle surface texture
    float texture = sin(p.x * 0.002) * cos(p.y * 0.0018) * 10.0;
    
    return cone + fields + texture;
}

float mountain_height(vec2 p) {
    // Much lower frequency for broader mountains
    float scale1 = 0.0002;
    float scale2 = 0.0004;
    
    // Smooth gaussian-like peaks instead of sharp ones
    float dist1 = length(vec2(sin(p.x * scale1), cos(p.y * scale1)));
    float dist2 = length(vec2(sin(p.x * scale2 + 1.0), cos(p.y * scale2 - 0.5)));
    
    // Use gaussian falloff for smooth peaks
    float h1 = exp(-dist1 * dist1 * 2.0) * 120.0;
    float h2 = exp(-dist2 * dist2 * 3.0) * 80.0;
    
    // Rolling foothills
    float foothills = 0.0;
    foothills += sin(p.x * 0.0008) * cos(p.y * 0.0007) * 30.0;
    foothills += sin(p.x * 0.0012 + 0.5) * sin(p.y * 0.001) * 20.0;
    
    // Gentle valleys between peaks
    float valley = sin(p.x * 0.0003 + p.y * 0.0002) * 15.0;
    
    return h1 + h2 + foothills + valley;
}

float plains_height(vec2 p) {
    // Very low frequency for gentle rolling plains
    float scale1 = 0.0002;
    float scale2 = 0.0004;
    
    // Gentle rolling hills
    float h1 = sin(p.x * scale1) * cos(p.y * scale1 * 0.8) * 25.0;
    float h2 = sin(p.x * scale2 + 1.0) * cos(p.y * scale2 * 1.2) * 15.0;
    
    // Subtle undulations
    float detail = sin(p.x * 0.001) * sin(p.y * 0.0008) * 8.0;
    
    return h1 + h2 + detail;
}

float desert_height(vec2 p) {
    // Low frequency for large dune fields
    float scale1 = 0.0003;
    float scale2 = 0.0006;
    
    // Smooth, rolling dunes
    float dunes = smoothstep(0.3, 0.7, sin(p.x * scale1) * cos(p.y * scale1 * 1.2) * 0.5 + 0.5) * 30.0;
    float secondary = sin(p.x * scale2 + 0.5) * cos(p.y * scale2 * 0.8) * 20.0;
    
    // Gentle ripples
    float ripples = sin(p.x * 0.003) * cos(p.y * 0.0025) * 5.0;
    
    return dunes + secondary + ripples;
}

float arctic_height(vec2 p) {
    // Low frequency for smooth, rolling ice sheets
    float scale1 = 0.0003;
    float scale2 = 0.0006;
    
    // Smooth rolling glacial terrain
    float glacial = sin(p.x * scale1) * cos(p.y * scale1 * 0.9) * 40.0;
    float sheets = cos(p.x * scale2 + 0.5) * sin(p.y * scale2 * 1.1) * 30.0;
    
    // Gentle ice dunes
    float dunes = smoothstep(0.2, 0.8, sin(p.x * 0.001) * cos(p.y * 0.0008) * 0.5 + 0.5) * 25.0;
    
    // Subtle surface texture
    float texture = sin(p.x * 0.008) * cos(p.y * 0.007) * 10.0;
    
    // Very gentle crevasses
    float crevasse = smoothstep(0.4, 0.6, sin(p.x * 0.002 + p.y * 0.0015)) * -15.0;
    
    return glacial + sheets + dunes + texture + crevasse;
}

float badlands_height(vec2 p) {
    // Low frequency for wider, smoother features
    float scale1 = 0.0004;
    float scale2 = 0.0008;
    
    // Smooth eroded hills
    float hills = sin(p.x * scale1) * cos(p.y * scale1 * 0.8) * 50.0;
    float erosion = smoothstep(0.3, 0.7, cos(p.x * scale2 + 0.7) * sin(p.y * scale2 * 1.2) * 0.5 + 0.5) * 30.0;
    
    // Gentle mesas with sloped sides
    float mesa = smoothstep(0.2, 0.6, sin(p.x * 0.0003) * cos(p.y * 0.00025) * 0.5 + 0.5) * 40.0;
    
    // Rolling badland texture
    float texture = sin(p.x * 0.002) * cos(p.y * 0.0018) * 15.0;
    
    return hills + erosion + mesa + texture;
}

float floating_height(vec2 p) {
    // Low frequency for large floating islands
    float scale1 = 0.0002;
    float scale2 = 0.0004;
    
    // Smooth floating plateaus
    float island1 = smoothstep(0.3, 0.7, sin(p.x * scale1) * cos(p.y * scale1) * 0.5 + 0.5) * 60.0;
    float island2 = smoothstep(0.4, 0.6, cos(p.x * scale2 + 0.8) * sin(p.y * scale2 * 0.9) * 0.5 + 0.5) * 40.0;
    
    // Gentle surface
    float surface = sin(p.x * 0.001) * cos(p.y * 0.0008) * 10.0;
    
    return 80.0 + island1 + island2 + surface;
}

float caverns_height(vec2 p) {
    // Low frequency for larger cavern systems
    float scale1 = 0.0004;
    float scale2 = 0.0008;
    
    // Rolling base terrain
    float base = sin(p.x * scale1) * cos(p.y * scale1 * 0.8) * 40.0;
    
    // Smooth depressions instead of sharp holes
    float depression1 = smoothstep(0.6, 0.3, sin(p.x * scale2) * cos(p.y * scale2) * 0.5 + 0.5) * -30.0;
    float depression2 = smoothstep(0.5, 0.2, cos(p.x * scale2 * 1.3 + 1.0) * sin(p.y * scale2 * 0.9) * 0.5 + 0.5) * -20.0;
    
    // Gentle undulations
    float detail = sin(p.x * 0.002) * cos(p.y * 0.0015) * 10.0;
    
    return base + depression1 + depression2 + detail;
}

float swamp_height(vec2 p) {
    // Low frequency for gentle swamp terrain
    float scale1 = 0.0005;
    float scale2 = 0.001;
    
    // Very gentle undulations
    float undulation = sin(p.x * scale1) * cos(p.y * scale1 * 0.9) * 15.0;
    float pools = smoothstep(0.4, 0.6, sin(p.x * scale2) * cos(p.y * scale2 * 1.1) * 0.5 + 0.5) * -10.0;
    
    // Subtle surface variation
    float surface = sin(p.x * 0.003) * cos(p.y * 0.0025) * 5.0;
    
    return undulation + pools + surface;
}

// Enhanced biome selection with more variety and smaller regions
float get_biome_height(vec2 p) {
    // Multi-scale noise for more organic biome distribution
    float noise1 = sin(p.x * 0.0003) * cos(p.y * 0.0003);
    float noise2 = sin(p.x * 0.0007 + 1.3) * sin(p.y * 0.0006 - 0.7);
    float noise3 = cos(p.x * 0.0013 - 2.1) * sin(p.y * 0.0011 + 1.9);
    
    // Combine noises for complex patterns
    float biome_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Add local variation for sub-biomes
    float local_var = sin(p.x * 0.01) * cos(p.y * 0.01) * 0.1;
    biome_noise += local_var;
    
    // 12 biome types distributed across the noise range
    if (biome_noise < -0.7) {
        return canyon_height(p);
    } else if (biome_noise < -0.5) {
        return caverns_height(p);
    } else if (biome_noise < -0.3) {
        return badlands_height(p);
    } else if (biome_noise < -0.1) {
        return plateau_height(p);
    } else if (biome_noise < 0.1) {
        return plains_height(p);
    } else if (biome_noise < 0.25) {
        return desert_height(p);
    } else if (biome_noise < 0.4) {
        return swamp_height(p);
    } else if (biome_noise < 0.5) {
        return crystalline_height(p);
    } else if (biome_noise < 0.6) {
        return volcanic_height(p);
    } else if (biome_noise < 0.7) {
        return arctic_height(p);
    } else if (biome_noise < 0.8) {
        return floating_height(p);
    } else {
        return mountain_height(p);
    }
}

// Smooth blending between biomes
float get_blended_biome_height(vec2 p) {
    // Sample multiple nearby points for smoother transitions
    float sample_dist = 50.0;
    float h_center = get_biome_height(p);
    float h_north = get_biome_height(p + vec2(0.0, sample_dist));
    float h_south = get_biome_height(p + vec2(0.0, -sample_dist));
    float h_east = get_biome_height(p + vec2(sample_dist, 0.0));
    float h_west = get_biome_height(p + vec2(-sample_dist, 0.0));
    
    // Average nearby samples for smoother terrain
    float primary_height = (h_center * 2.0 + h_north + h_south + h_east + h_west) / 6.0;
    
    // Add rolling hills with lower frequency
    float hills = 0.0;
    hills += sin(p.x * 0.0001) * cos(p.y * 0.00012) * 60.0;
    hills += sin(p.x * 0.00018 + 1.5) * cos(p.y * 0.00015 - 0.7) * 40.0;
    hills += sin(p.x * 0.00025 - 0.3) * cos(p.y * 0.0003 + 1.2) * 25.0;
    
    // Add gentle undulations
    float undulation = 0.0;
    undulation += sin(p.x * 0.0004) * cos(p.y * 0.0004) * 15.0;
    undulation += sin(p.x * 0.0008 + 2.1) * sin(p.y * 0.0007 - 1.3) * 10.0;
    
    // Very gentle large scale features
    float continent_scale = 0.00005;
    float continental = sin(p.x * continent_scale) * cos(p.y * continent_scale) * 30.0;
    
    // Smooth everything together
    float height = primary_height * 0.7 + hills * 0.2 + undulation * 0.1;
    
    return height + continental;
}

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y, pos.z + instance_offset.y);
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Sample height from texture instead of procedural generation
    vec2 uv = (world_pos.xz / terrain_scale) + 0.5;
    
    // Debug: Check if UV coordinates are varying
    // If terrain_scale is 3360 and world_pos.x ranges from about -1680 to +1680
    // then uv should range from 0 to 1
    
    vec4 height_sample = texture2D(height_texture, uv);
    
    // Decode 24-bit height value from RGB channels
    float height_normalized = (height_sample.r * 255.0 * 65536.0 + 
                              height_sample.g * 255.0 * 256.0 + 
                              height_sample.b * 255.0) / 16777215.0;
    
    // Convert from normalized [0,1] to actual height range [-500, 500]
    float height = (height_normalized * 1000.0) - 500.0;
    
    // Since vertex texture fetch isn't working properly, use procedural generation
    // This will use the same improved terrain generation functions
    world_pos.y = get_blended_biome_height(world_pos.xz);
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    // Debug: visualize UV coordinates as height to check if they're correct
    // world_pos.y = uv.x * 100.0 - 50.0;  // Uncomment to debug UV.x
    // world_pos.y = uv.y * 100.0 - 50.0;  // Uncomment to debug UV.y
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

pub const FRAGMENT_TERRAIN: &str = r#"#version 100
precision mediump float;

uniform vec3 color;
uniform sampler2D height_texture;
uniform sampler2D biome_texture;
uniform float terrain_scale;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

void main() {
    // Sample biome color from texture
    vec2 uv = (v_world_xz / terrain_scale) + 0.5;
    vec3 biome_color = texture2D(biome_texture, uv).rgb;
    
    // DEBUG: Sample height in fragment shader to verify texture is working
    vec4 height_sample = texture2D(height_texture, uv);
    float debug_height = height_sample.r;
    
    // Calculate height-based brightness (higher = brighter)
    // Terrain can range from about -500 to +500, normalize to this range
    float height_factor = (v_height + 500.0) / 1000.0; // Normalize to 0-1 range
    height_factor = clamp(height_factor, 0.0, 1.0);
    height_factor = 0.3 + height_factor * 0.7; // Map to 0.3-1.0 range for more contrast
    
    // Blend base color with biome color
    vec3 blended_color = mix(color, biome_color, 0.7); // 70% biome color, 30% base color
    vec3 adjusted_color = blended_color * height_factor;
    
    // DEBUG: Visualize the height texture in red channel
    // adjusted_color.r = debug_height;
    
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(adjusted_color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(adjusted_color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(adjusted_color, 1.0); // Bright version for edges
        }
    }
}"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

uniform vec3 color;

varying vec3 v_barycentric;

void main() {
    // If barycentric coordinates are all zero, this is a line vertex
    if (v_barycentric.x == 0.0 && v_barycentric.y == 0.0 && v_barycentric.z == 0.0) {
        gl_FragColor = vec4(color, 1.0);
    } else {
        // Show filled triangles with darker color
        gl_FragColor = vec4(color * 0.5, 1.0); // Darker version for filled areas
        
        // Calculate distance to nearest edge for triangles
        float minBary = min(min(v_barycentric.x, v_barycentric.y), v_barycentric.z);
        
        // Draw edges brighter
        if (minBary < 0.05) {
            gl_FragColor = vec4(color, 1.0); // Bright version for edges
        }
    }
}"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
            ],
        },
    }
}

pub fn meta_terrain() -> ShaderMeta {
    ShaderMeta {
        images: vec!["height_texture".to_string(), "biome_texture".to_string()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("color", UniformType::Float3),
                UniformDesc::new("terrain_scale", UniformType::Float1),
                UniformDesc::new("terrain_y_base", UniformType::Float1),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub color: [f32; 3],
    pub _padding: f32,
}

impl Uniforms {
    pub fn new(mvp: Mat4, color: [f32; 3]) -> Self {
        Self {
            mvp: mvp.to_cols_array_2d(),
            color,
            _padding: 0.0,
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