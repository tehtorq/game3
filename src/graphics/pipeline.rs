use miniquad::*;
use crate::shader;
use crate::shader_volumetric_laser;

pub struct GraphicsPipelines {
    pub line_pipeline: Pipeline,
    pub triangle_pipeline: Pipeline,
    pub bullet_pipeline: Pipeline,
    pub bullet_glow_pipeline: Pipeline,
    pub volumetric_laser_pipeline: Pipeline,
    pub terrain_simple_pipeline: Pipeline,
}

pub fn create_pipelines(ctx: &mut dyn RenderingBackend) -> (GraphicsPipelines, Bindings, Bindings, Bindings) {
    // Main shader
    let shader = ctx.new_shader(
        ShaderSource::Glsl {
            vertex: shader::VERTEX,
            fragment: shader::FRAGMENT,
        },
        shader::meta()
    ).unwrap();
    
    // Volumetric laser shader
    let volumetric_laser_shader = ctx.new_shader(
        ShaderSource::Glsl {
            vertex: shader_volumetric_laser::VERTEX_VOLUMETRIC_LASER,
            fragment: shader_volumetric_laser::FRAGMENT_VOLUMETRIC_LASER,
        },
        shader::meta_volumetric_laser()
    ).unwrap();
    
    // Create pipelines
    let line_pipeline = ctx.new_pipeline(
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("uv", VertexFormat::Float2),
            VertexAttribute::new("color", VertexFormat::Float4),
        ],
        shader.clone(),
        PipelineParams {
            primitive_type: PrimitiveType::Lines,
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            ..Default::default()
        }
    );
    
    let triangle_pipeline = ctx.new_pipeline(
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("uv", VertexFormat::Float2),
            VertexAttribute::new("color", VertexFormat::Float4),
        ],
        shader.clone(),
        PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            ..Default::default()
        }
    );
    
    let bullet_pipeline = ctx.new_pipeline(
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("uv", VertexFormat::Float2),
            VertexAttribute::new("color", VertexFormat::Float4),
        ],
        shader.clone(),
        PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            ..Default::default()
        }
    );
    
    let bullet_glow_pipeline = ctx.new_pipeline(
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("uv", VertexFormat::Float2),
            VertexAttribute::new("color", VertexFormat::Float4),
        ],
        shader.clone(),
        PipelineParams {
            depth_test: Comparison::Always,
            depth_write: false,
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::One
            )),
            ..Default::default()
        }
    );
    
    let volumetric_laser_pipeline = ctx.new_pipeline(
        &[BufferLayout::default()],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("normal", VertexFormat::Float3),
            VertexAttribute::new("color", VertexFormat::Float4),
        ],
        volumetric_laser_shader,
        PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: false,
            cull_face: CullFace::Nothing,
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha)
            )),
            ..Default::default()
        }
    );
    
    // Terrain shader
    let terrain_shader = ctx.new_shader(
        ShaderSource::Glsl {
            vertex: shader::VERTEX_TERRAIN_SIMPLE,
            fragment: shader::FRAGMENT_TERRAIN_SIMPLE,
        },
        shader::meta_terrain_simple()
    ).unwrap();
    
    let terrain_simple_pipeline = ctx.new_pipeline(
        &[
            BufferLayout::default(),
            BufferLayout {
                step_func: VertexStep::PerInstance,
                ..Default::default()
            }
        ],
        &[
            VertexAttribute::new("position", VertexFormat::Float3),
            VertexAttribute::new("uv", VertexFormat::Float2),
            VertexAttribute::new("color", VertexFormat::Float4),
            VertexAttribute::new("chunk_offset", VertexFormat::Float2),
        ],
        terrain_shader,
        PipelineParams {
            depth_test: Comparison::LessOrEqual,
            depth_write: true,
            cull_face: CullFace::Back,
            ..Default::default()
        }
    );
    
    // Create bindings
    let vertex_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Stream,
        BufferSource::empty::<crate::vertex::Vertex>(10000)
    );
    
    let index_buffer = ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Stream,
        BufferSource::empty::<u32>(10000)
    );
    
    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer],
        index_buffer,
        images: vec![],
    };
    
    // Terrain bindings placeholder
    let terrain_bindings = bindings.clone();
    
    // Bullet bindings
    let bullet_vertex_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Stream,
        BufferSource::empty::<crate::vertex::Vertex>(1000)
    );
    
    let bullet_index_buffer = ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Stream,
        BufferSource::empty::<u32>(1000)
    );
    
    let bullet_bindings = Bindings {
        vertex_buffers: vec![bullet_vertex_buffer],
        index_buffer: bullet_index_buffer,
        images: vec![],
    };
    
    let pipelines = GraphicsPipelines {
        line_pipeline,
        triangle_pipeline,
        bullet_pipeline,
        bullet_glow_pipeline,
        volumetric_laser_pipeline,
        terrain_simple_pipeline,
    };
    
    (pipelines, bindings, terrain_bindings, bullet_bindings)
}