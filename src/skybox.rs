use miniquad::*;
use glam::{Vec3, Mat4};
use crate::vertex::Vertex;

pub struct Skybox {
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    pipeline: Pipeline,
}

impl Skybox {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
        // Create a large cube for the skybox
        let size = 10000.0;
        
        // Define vertices for a cube (using x,y,z format for Vertex)
        let vertices = vec![
            // Front face
            Vertex::new(-size, -size,  size),
            Vertex::new( size, -size,  size),
            Vertex::new( size,  size,  size),
            Vertex::new(-size,  size,  size),
            
            // Back face
            Vertex::new(-size, -size, -size),
            Vertex::new(-size,  size, -size),
            Vertex::new( size,  size, -size),
            Vertex::new( size, -size, -size),
            
            // Top face
            Vertex::new(-size,  size, -size),
            Vertex::new(-size,  size,  size),
            Vertex::new( size,  size,  size),
            Vertex::new( size,  size, -size),
            
            // Bottom face
            Vertex::new(-size, -size, -size),
            Vertex::new( size, -size, -size),
            Vertex::new( size, -size,  size),
            Vertex::new(-size, -size,  size),
            
            // Right face
            Vertex::new( size, -size, -size),
            Vertex::new( size,  size, -size),
            Vertex::new( size,  size,  size),
            Vertex::new( size, -size,  size),
            
            // Left face
            Vertex::new(-size, -size, -size),
            Vertex::new(-size, -size,  size),
            Vertex::new(-size,  size,  size),
            Vertex::new(-size,  size, -size),
        ];
        
        // Define indices for the cube (render from inside)
        let indices: Vec<u32> = vec![
            // Front face (reversed for inside view)
            0, 2, 1,    0, 3, 2,
            // Back face
            4, 6, 5,    4, 7, 6,
            // Top face
            8, 10, 9,   8, 11, 10,
            // Bottom face
            12, 14, 13, 12, 15, 14,
            // Right face
            16, 18, 17, 16, 19, 18,
            // Left face
            20, 22, 21, 20, 23, 22,
        ];
        
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );
        
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );
        
        // Create skybox shader
        let shader = ctx.new_shader(
            ShaderSource::Glsl {
                vertex: SKYBOX_VERTEX_SHADER,
                fragment: SKYBOX_FRAGMENT_SHADER,
            },
            skybox_shader_meta(),
        ).unwrap();
        
        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float3),
                VertexAttribute::new("barycentric", VertexFormat::Float3),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: false, // Don't write to depth buffer
                cull_face: CullFace::Front, // Cull front faces since we're inside the cube
                ..Default::default()
            },
        );
        
        Self {
            vertex_buffer,
            index_buffer,
            pipeline,
        }
    }
    
    pub fn draw(&self, ctx: &mut dyn RenderingBackend, view_proj: Mat4, camera_pos: Vec3) {
        // Create a view matrix without translation to keep skybox centered on camera
        let view_rotation = view_proj;
        
        let bindings = Bindings {
            vertex_buffers: vec![self.vertex_buffer],
            index_buffer: self.index_buffer,
            images: vec![],
        };
        
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&bindings);
        ctx.apply_uniforms(UniformsSource::table(&SkyboxUniforms {
            mvp: view_rotation,
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
            time: miniquad::date::now() as f32,
        }));
        ctx.draw(0, 36, 1);
    }
}

#[repr(C)]
struct SkyboxUniforms {
    mvp: Mat4,
    camera_pos: [f32; 3],
    time: f32,
}

const SKYBOX_VERTEX_SHADER: &str = r#"#version 100
precision highp float;

attribute vec3 position;
attribute vec3 barycentric;

uniform mat4 mvp;
uniform vec3 camera_pos;

varying vec3 v_pos;
varying vec3 v_world_pos;

void main() {
    // Keep skybox centered on camera
    vec3 pos = position + camera_pos;
    gl_Position = mvp * vec4(pos, 1.0);
    
    // Make sure skybox is always behind everything
    gl_Position.z = gl_Position.w * 0.9999;
    
    v_pos = position;
    v_world_pos = pos;
}
"#;

const SKYBOX_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

uniform float time;
uniform vec3 camera_pos;

varying vec3 v_pos;
varying vec3 v_world_pos;

// Simple gradient skybox with some atmospheric effects
void main() {
    vec3 direction = normalize(v_pos);
    float y = direction.y;
    
    // Sky gradient
    vec3 horizon_color = vec3(0.7, 0.8, 0.9);
    vec3 zenith_color = vec3(0.2, 0.3, 0.6);
    vec3 ground_color = vec3(0.1, 0.1, 0.15);
    
    vec3 sky_color;
    if (y > 0.0) {
        // Sky
        float t = y;
        sky_color = mix(horizon_color, zenith_color, t);
    } else {
        // Ground
        float t = -y;
        sky_color = mix(horizon_color, ground_color, t * 0.5);
    }
    
    // Add some subtle clouds/atmosphere
    float cloud_density = 0.0;
    if (y > -0.1 && y < 0.4) {
        // Simple cloud pattern using sine waves
        float cloud_x = v_world_pos.x * 0.0001 + time * 0.01;
        float cloud_z = v_world_pos.z * 0.0001 + time * 0.005;
        
        cloud_density = sin(cloud_x) * sin(cloud_z * 0.7) * 0.5 + 0.5;
        cloud_density *= (1.0 - abs(y - 0.15) * 3.0);
        cloud_density = clamp(cloud_density * 0.3, 0.0, 0.3);
    }
    
    // Add clouds to sky color
    sky_color = mix(sky_color, vec3(1.0, 1.0, 1.0), cloud_density);
    
    // Add sun/moon glow
    vec3 sun_dir = normalize(vec3(0.5, 0.3, 0.8));
    float sun_dot = dot(direction, sun_dir);
    if (sun_dot > 0.95) {
        float sun_intensity = smoothstep(0.95, 0.99, sun_dot);
        sky_color = mix(sky_color, vec3(1.0, 0.9, 0.7), sun_intensity);
    }
    
    // Fog effect near horizon
    float fog_factor = 1.0 - abs(y);
    fog_factor = pow(fog_factor, 3.0);
    sky_color = mix(sky_color, horizon_color, fog_factor * 0.3);
    
    gl_FragColor = vec4(sky_color, 1.0);
}
"#;

fn skybox_shader_meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4),
                UniformDesc::new("camera_pos", UniformType::Float3),
                UniformDesc::new("time", UniformType::Float1),
            ],
        },
    }
}