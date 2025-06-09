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
attribute vec3 pos;
attribute vec3 barycentric;
attribute vec2 instance_offset;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

uniform mat4 mvp;
uniform float terrain_scale;
uniform float terrain_y_base;

// Biome height generation functions
float canyon_height(vec2 p) {
    float scale = 0.003 * 1.5;
    float canyon_cut = (sin(p.x * scale) + cos(p.y * scale * 0.7)) * 0.5;
    float depth = pow(abs(canyon_cut), 3.0);
    return -300.0 + (1.0 - depth) * 350.0;
}

float plateau_height(vec2 p) {
    float scale = 0.001 * 0.5;
    float plateau_shape = clamp(sin(p.x * scale) * cos(p.y * scale), -1.0, 1.0);
    float flatness = pow(abs(plateau_shape), 0.2);
    return 200.0 + flatness * 100.0;
}

float crystalline_height(vec2 p) {
    float scale1 = 0.01 * 3.0;
    float scale2 = 0.02 * 3.0;
    float scale3 = 0.05 * 3.0;
    
    float spike1 = abs(sin(p.x * scale1) * cos(p.y * scale1)) * 100.0;
    float spike2 = abs(cos(p.x * scale2 + 50.0) * sin(p.y * scale2 - 30.0)) * 70.0;
    float spike3 = abs(sin(p.x * scale3 - 20.0) * cos(p.y * scale3 + 40.0)) * 40.0;
    
    return clamp(spike1 + spike2 + spike3, -50.0, 250.0);
}

float volcanic_height(vec2 p) {
    float scale1 = 0.004 * 2.0;
    float scale2 = 0.008 * 2.0;
    
    float crater = sqrt(pow(sin(p.x * scale1), 2.0) + pow(cos(p.y * scale1), 2.0));
    float rough = sin(p.x * scale2) * cos(p.y * scale2) * 80.0 * 0.7;
    
    return clamp((1.0 - crater) * 80.0 + rough, -50.0, 150.0);
}

float mountain_height(vec2 p) {
    float scale1 = 0.001 * 0.8;
    float scale2 = 0.003 * 0.8;
    
    float ridge = abs(sin(p.x * scale1) - cos(p.y * scale1 * 0.8)) * 200.0;
    float peaks = abs(sin(p.x * scale2 + 100.0) * cos(p.y * scale2 - 50.0)) * 120.0;
    
    return clamp(ridge + peaks, -100.0, 400.0);
}

float plains_height(vec2 p) {
    float scale1 = 0.002;
    float scale2 = 0.007;
    float scale3 = 0.015;
    
    float h1 = sin(p.x * scale1) * cos(p.y * scale1) * 60.0;
    float h2 = sin(p.x * scale2 + 100.0) * sin(p.y * scale2 + 100.0) * 30.0;
    float h3 = cos(p.x * scale3 + 200.0) * cos(p.y * scale3 + 200.0) * 15.0;
    
    return h1 + h2 + h3;
}

float desert_height(vec2 p) {
    float scale1 = 0.005 * 2.5;
    float scale2 = 0.01 * 2.5;
    float scale3 = 0.03 * 2.5;
    
    float dunes = abs(sin(p.x * scale1) * cos(p.y * scale1 * 1.2)) * 50.0;
    float ripples = sin(p.x * scale3) * cos(p.y * scale3) * 5.0;
    float medium = cos(p.x * scale2 + 30.0) * sin(p.y * scale2 - 20.0) * 15.0;
    
    return -20.0 + dunes + ripples + medium;
}

float arctic_height(vec2 p) {
    float scale1 = 0.008 * 1.8;
    float scale2 = 0.02 * 1.8;
    float scale3 = 0.04 * 1.8;
    
    float glacial = (sin(p.x * scale1) + cos(p.y * scale1 * 0.9)) * 60.0;
    float spikes = pow(abs(sin(p.x * scale2) * cos(p.y * scale2)), 2.0) * 120.0;
    float crevasses = abs(sin(p.x * scale3 + p.y * scale3 * 0.7)) * 36.0;
    
    return 50.0 + glacial + spikes - crevasses;
}

float badlands_height(vec2 p) {
    float scale1 = 0.006 * 2.2;
    float scale2 = 0.015 * 2.2;
    float scale3 = 0.04 * 2.2;
    
    float mesas = pow(abs(sin(p.x * scale1) * cos(p.y * scale1)), 0.3) * 90.0;
    float erosion = abs(sin(p.x * scale2 + p.y * scale2 * 0.5) + cos(p.x * scale2 * 0.7 - p.y * scale2)) * 36.0;
    float pillars = pow(abs(sin(p.x * scale3) * cos(p.y * scale3)), 3.0) * 45.0;
    
    return -150.0 + mesas + erosion + pillars;
}

float floating_height(vec2 p) {
    float scale1 = 0.002 * 0.3;
    float scale2 = 0.008 * 0.3;
    
    float islands = sqrt(pow(sin(p.x * scale1), 2.0) + pow(cos(p.y * scale1), 2.0));
    float shape = pow(max(1.0 - islands, 0.0), 2.0) * 60.0;
    float detail = sin(p.x * scale2) * cos(p.y * scale2) * 6.0;
    
    return 150.0 + shape + detail;
}

float caverns_height(vec2 p) {
    float scale1 = 0.01 * 3.5;
    float scale2 = 0.025 * 3.5;
    float scale3 = 0.05 * 3.5;
    
    float base = sin(p.x * scale1) * cos(p.y * scale1) * 50.0;
    float holes1 = pow(abs(sin(p.x * scale2) * cos(p.y * scale2)), 4.0) * 100.0;
    float holes2 = pow(abs(cos(p.x * scale3 + 50.0) * sin(p.y * scale3 - 30.0)), 4.0) * 70.0;
    
    return base - holes1 - holes2;
}

float swamp_height(vec2 p) {
    float scale1 = 0.02 * 4.0;
    float scale2 = 0.04 * 4.0;
    float scale3 = 0.08 * 4.0;
    
    float undulation = sin(p.x * scale1) * cos(p.y * scale1) * 9.0;
    float pools = (sin(p.x * scale2) + cos(p.y * scale2)) * 6.0;
    float bumps = sin(p.x * scale3) * sin(p.y * scale3) * 3.0;
    
    return -50.0 + undulation + pools + bumps;
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
    // Get primary biome height
    float primary_height = get_biome_height(p);
    
    // Add micro-scale detail that works across all biomes
    float micro1 = sin(p.x * 0.1) * cos(p.y * 0.1) * 5.0;
    float micro2 = sin(p.x * 0.2 + p.y * 0.15) * 3.0;
    float micro3 = cos(p.x * 0.05 - p.y * 0.07) * sin(p.x * 0.03 + p.y * 0.04) * 8.0;
    
    // Add terrain features that span biomes
    float ridge = pow(abs(sin(p.x * 0.0002) - cos(p.y * 0.00025)), 3.0) * 50.0;
    float valleys = -pow(abs(sin(p.x * 0.00015 + p.y * 0.0001)), 2.0) * 30.0;
    
    return primary_height + micro1 + micro2 + micro3 + ridge + valleys;
}

void main() {
    v_barycentric = barycentric;
    
    // Calculate world position with instance offset
    vec3 world_pos = vec3(pos.x + instance_offset.x, pos.y + 20.0, pos.z + instance_offset.y);
    
    // Pass world XZ to fragment shader for biome sampling
    v_world_xz = world_pos.xz;
    
    // Calculate biome-based height with blending
    world_pos.y += get_blended_biome_height(world_pos.xz);
    
    // Pass height to fragment shader
    v_height = world_pos.y;
    
    gl_Position = mvp * vec4(world_pos, 1.0);
}"#;

pub const FRAGMENT_TERRAIN: &str = r#"#version 100
precision mediump float;

uniform vec3 color;
uniform sampler2D biome_texture;
uniform float terrain_scale;

varying vec3 v_barycentric;
varying float v_height;
varying vec2 v_world_xz;

void main() {
    // Sample biome color from texture
    vec2 uv = (v_world_xz / terrain_scale) + 0.5;
    vec3 biome_color = texture2D(biome_texture, uv).rgb;
    
    // Calculate height-based brightness (higher = brighter)
    // Terrain can range from about -500 to +500, normalize to this range
    float height_factor = (v_height + 500.0) / 1000.0; // Normalize to 0-1 range
    height_factor = clamp(height_factor, 0.0, 1.0);
    height_factor = 0.3 + height_factor * 0.7; // Map to 0.3-1.0 range for more contrast
    
    // Blend base color with biome color
    vec3 blended_color = mix(color, biome_color, 0.7); // 70% biome color, 30% base color
    vec3 adjusted_color = blended_color * height_factor;
    
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
        images: vec!["biome_texture".to_string()],
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