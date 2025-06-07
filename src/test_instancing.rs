use miniquad::*;

pub fn test_instancing(ctx: &mut dyn RenderingBackend) {
    // Create a simple triangle as base mesh
    let vertices: [f32; 9] = [
        -10.0, -10.0, 0.0,  // vertex 0
         10.0, -10.0, 0.0,  // vertex 1
          0.0,  10.0, 0.0,  // vertex 2
    ];
    
    let vertex_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices)
    );
    
    // Create instance data - positions for 4 triangles
    let instances: [f32; 12] = [
        -50.0, 0.0, -50.0,  // instance 0
         50.0, 0.0, -50.0,  // instance 1
        -50.0, 0.0,  50.0,  // instance 2
         50.0, 0.0,  50.0,  // instance 3
    ];
    
    let instance_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&instances)
    );
    
    println!("Test instancing buffers created");
    
    // Return the buffer IDs
    (vertex_buffer, instance_buffer)
}