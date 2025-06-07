# Miniquad Rendering Notes

## Basic Rendering Pipeline

### 1. Buffer Creation
```rust
// Vertex buffer
let vertex_buffer = ctx.new_buffer(
    BufferType::VertexBuffer,
    BufferUsage::Stream,  // or Immutable
    BufferSource::slice(&vertices)  // or BufferSource::empty::<T>(size)
);

// Index buffer
let index_buffer = ctx.new_buffer(
    BufferType::IndexBuffer,
    BufferUsage::Stream,
    BufferSource::slice(&indices)
);
```

### 2. Pipeline Creation
```rust
let pipeline = ctx.new_pipeline(
    &[BufferLayout::default()],  // Buffer layouts
    &[
        VertexAttribute::new("pos", VertexFormat::Float3),
        VertexAttribute::new("color", VertexFormat::Float4),
    ],
    shader,
    PipelineParams {
        depth_test: Comparison::LessOrEqual,
        depth_write: true,
        cull_face: CullFace::Back,
        primitive_type: PrimitiveType::Triangles,  // or Lines
        ..Default::default()
    }
);
```

### 3. Bindings
```rust
let bindings = Bindings {
    vertex_buffers: vec![vertex_buffer],
    index_buffer,
    images: vec![],  // textures go here
};
```

### 4. Draw Call
```rust
ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
ctx.apply_pipeline(&pipeline);
ctx.apply_bindings(&bindings);
ctx.apply_uniforms(UniformsSource::table(&uniforms));
ctx.draw(base_element, num_elements, num_instances);
ctx.end_render_pass();
ctx.commit_frame();
```

## Instanced Rendering

### Key Differences from Regular Rendering:

1. **Multiple Buffer Layouts**: Need at least 2 - one for per-vertex data, one for per-instance data
```rust
&[
    BufferLayout::default(),  // Per-vertex
    BufferLayout {
        step_func: VertexStep::PerInstance,
        ..Default::default()
    }  // Per-instance
]
```

2. **Vertex Attributes with Buffer Index**: Must specify which buffer each attribute comes from
```rust
&[
    VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),      // buffer 0
    VertexAttribute::with_buffer("color", VertexFormat::Float4, 0),    // buffer 0
    VertexAttribute::with_buffer("instance_pos", VertexFormat::Float3, 1), // buffer 1
]
```

3. **Multiple Vertex Buffers in Bindings**:
```rust
let bindings = Bindings {
    vertex_buffers: vec![per_vertex_buffer, per_instance_buffer],
    index_buffer,
    images: vec![],
};
```

4. **Draw Call**: Last parameter is number of instances
```rust
ctx.draw(0, vertices_per_instance, num_instances);
```

## Shader Requirements

### For Instanced Rendering:
- Instance attributes must be declared in vertex shader
- Instance data is combined with vertex data in shader
```glsl
attribute vec3 pos;           // per-vertex
attribute vec3 instance_pos;  // per-instance

void main() {
    vec3 world_pos = pos + instance_pos;
    gl_Position = mvp * vec4(world_pos, 1.0);
}
```

## Common Issues & Solutions

1. **Nothing renders**: 
   - Check vertex winding order (counter-clockwise for front faces)
   - Verify camera/MVP matrix is correct
   - Check if vertices are within view frustum
   - Ensure depth test settings are appropriate

2. **Instancing not working**:
   - Verify buffer layouts match vertex attributes
   - Ensure vertex attributes specify correct buffer indices
   - Check that vertex buffers array in Bindings has all required buffers
   - Make sure instance buffer has correct data format

3. **Texture sampling issues**:
   - ShaderMeta must list image names that match uniform names in shader
   - Textures must be added to bindings.images in same order as in ShaderMeta

## Coordinate System
- Miniquad uses a left-handed coordinate system by default
- Y-up convention (Y points upward)
- Depth buffer: near=0, far=1 (reverse of OpenGL tradition)

## Performance Tips
- Use Immutable buffers when data doesn't change
- Batch draw calls when possible
- Minimize state changes (pipeline, bindings, uniforms)