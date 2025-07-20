use miniquad::*;
use glam::Vec3;
use crate::vertex::Vertex;
use crate::tree::{Tree, TreeType};
use std::collections::HashMap;

pub struct TreeInstancingSystem {
    // Separate buffers for each tree type since they have different geometries
    tree_buffers: HashMap<TreeType, TreeTypeBuffers>,
    instance_buffers: HashMap<TreeType, BufferId>,
    max_trees_per_type: usize,
}

struct TreeTypeBuffers {
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    index_count: i32,
}

impl TreeInstancingSystem {
    pub fn new(ctx: &mut dyn RenderingBackend, max_trees_per_type: usize) -> Self {
        let mut tree_buffers = HashMap::new();
        let mut instance_buffers = HashMap::new();
        
        // Create geometry for each tree type
        let tree_types = [
            TreeType::Pine,
            TreeType::Oak,
            TreeType::Palm,
            TreeType::Crystal,
            TreeType::Cactus,
            TreeType::Mushroom,
            TreeType::Dead,
        ];
        
        for tree_type in &tree_types {
            let (vertices, indices) = Self::create_tree_geometry(*tree_type);
            
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices)
            );
            
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices)
            );
            
            let instance_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<TreeInstanceData>(max_trees_per_type)
            );
            
            tree_buffers.insert(*tree_type, TreeTypeBuffers {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as i32,
            });
            
            instance_buffers.insert(*tree_type, instance_buffer);
        }
        
        Self {
            tree_buffers,
            instance_buffers,
            max_trees_per_type,
        }
    }
    
    fn create_tree_geometry(tree_type: TreeType) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        match tree_type {
            TreeType::Pine => Self::create_pine_geometry(&mut vertices, &mut indices),
            TreeType::Oak => Self::create_oak_geometry(&mut vertices, &mut indices),
            TreeType::Palm => Self::create_palm_geometry(&mut vertices, &mut indices),
            TreeType::Crystal => Self::create_crystal_geometry(&mut vertices, &mut indices),
            TreeType::Cactus => Self::create_cactus_geometry(&mut vertices, &mut indices),
            TreeType::Mushroom => Self::create_mushroom_geometry(&mut vertices, &mut indices),
            TreeType::Dead => Self::create_dead_tree_geometry(&mut vertices, &mut indices),
        }
        
        (vertices, indices)
    }
    
    fn create_pine_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Pine tree: 3 stacked cones + trunk
        // Normalized to unit height and radius
        
        // Trunk (cylinder)
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.15, 0.3);
        
        // Three cone levels
        for i in 0..3 {
            let y_offset = 0.3 + i as f32 * 0.2;
            let level_height = 0.3 * (1.0 - i as f32 * 0.2);
            let level_radius = 0.7 * (1.0 - i as f32 * 0.3);
            
            Self::add_cone(vertices, indices, 
                Vec3::new(0.0, y_offset, 0.0), 
                level_radius, 
                level_height
            );
        }
    }
    
    fn create_oak_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Oak tree: sphere canopy + trunk
        // Trunk
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.2, 0.6);
        
        // Canopy (sphere)
        Self::add_sphere(vertices, indices, Vec3::new(0.0, 0.75, 0.0), 0.5, 2);
    }
    
    fn create_palm_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Simplified palm: just trunk for now (fronds would need special handling)
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.1, 1.0);
    }
    
    fn create_crystal_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Crystal: multiple octahedrons
        for i in 0..3 {
            let offset_angle = i as f32 * std::f32::consts::PI * 2.0 / 3.0;
            let offset = Vec3::new(
                offset_angle.cos() * 0.2,
                0.0,
                offset_angle.sin() * 0.2
            );
            let height = 0.7 + i as f32 * 0.1;
            
            Self::add_octahedron(vertices, indices, 
                offset + Vec3::new(0.0, height * 0.5, 0.0),
                height * 0.5
            );
        }
    }
    
    fn create_cactus_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Main trunk
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.2, 1.0);
        
        // Arms (simplified)
        Self::add_cylinder(vertices, indices, 
            Vec3::new(0.25, 0.6, 0.0), 0.14, 0.3);
        Self::add_cylinder(vertices, indices, 
            Vec3::new(-0.25, 0.5, 0.0), 0.14, 0.4);
    }
    
    fn create_mushroom_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Stem
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.15, 0.8);
        
        // Cap (hemisphere)
        Self::add_hemisphere(vertices, indices, Vec3::new(0.0, 0.8, 0.0), 0.5, 2);
    }
    
    fn create_dead_tree_geometry(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>) {
        // Trunk only (branches would need special handling)
        Self::add_cylinder(vertices, indices, Vec3::ZERO, 0.15, 0.8);
    }
    
    // Geometry helper functions
    fn add_cylinder(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, 
                    base_center: Vec3, radius: f32, height: f32) {
        let segments = 8;
        
        // Create triangles directly with proper barycentric coordinates
        for i in 0..segments {
            let angle1 = i as f32 * std::f32::consts::PI * 2.0 / segments as f32;
            let angle2 = (i + 1) as f32 * std::f32::consts::PI * 2.0 / segments as f32;
            
            let x1 = angle1.cos() * radius;
            let z1 = angle1.sin() * radius;
            let x2 = angle2.cos() * radius;
            let z2 = angle2.sin() * radius;
            
            // Bottom positions
            let bottom1 = Vec3::new(base_center.x + x1, base_center.y, base_center.z + z1);
            let bottom2 = Vec3::new(base_center.x + x2, base_center.y, base_center.z + z2);
            // Top positions
            let top1 = Vec3::new(base_center.x + x1, base_center.y + height, base_center.z + z1);
            let top2 = Vec3::new(base_center.x + x2, base_center.y + height, base_center.z + z2);
            
            let base_idx = vertices.len() as u32;
            
            // First triangle of side
            vertices.push(Vertex::with_barycentric(bottom1.x, bottom1.y, bottom1.z, 1.0, 0.0, 0.0));
            vertices.push(Vertex::with_barycentric(bottom2.x, bottom2.y, bottom2.z, 0.0, 1.0, 0.0));
            vertices.push(Vertex::with_barycentric(top1.x, top1.y, top1.z, 0.0, 0.0, 1.0));
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
            
            // Second triangle of side
            let base_idx = vertices.len() as u32;
            vertices.push(Vertex::with_barycentric(bottom2.x, bottom2.y, bottom2.z, 1.0, 0.0, 0.0));
            vertices.push(Vertex::with_barycentric(top2.x, top2.y, top2.z, 0.0, 1.0, 0.0));
            vertices.push(Vertex::with_barycentric(top1.x, top1.y, top1.z, 0.0, 0.0, 1.0));
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        }
    }
    
    fn add_cone(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
                base_center: Vec3, radius: f32, height: f32) {
        let segments = 8;
        let tip = Vec3::new(base_center.x, base_center.y + height, base_center.z);
        
        // Create triangles directly
        for i in 0..segments {
            let angle1 = i as f32 * std::f32::consts::PI * 2.0 / segments as f32;
            let angle2 = (i + 1) as f32 * std::f32::consts::PI * 2.0 / segments as f32;
            
            let x1 = angle1.cos() * radius;
            let z1 = angle1.sin() * radius;
            let x2 = angle2.cos() * radius;
            let z2 = angle2.sin() * radius;
            
            let base1 = Vec3::new(base_center.x + x1, base_center.y, base_center.z + z1);
            let base2 = Vec3::new(base_center.x + x2, base_center.y, base_center.z + z2);
            
            let base_idx = vertices.len() as u32;
            
            // Triangle from tip to base edge
            vertices.push(Vertex::with_barycentric(tip.x, tip.y, tip.z, 1.0, 0.0, 0.0));
            vertices.push(Vertex::with_barycentric(base1.x, base1.y, base1.z, 0.0, 1.0, 0.0));
            vertices.push(Vertex::with_barycentric(base2.x, base2.y, base2.z, 0.0, 0.0, 1.0));
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        }
    }
    
    fn add_sphere(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
                  center: Vec3, radius: f32, detail: i32) {
        let rings = 4 + detail * 2;
        let sectors = 6 + detail * 3;
        let base_idx = vertices.len() as u32;
        
        // Generate vertices
        for r in 0..=rings {
            let lat = std::f32::consts::PI * r as f32 / rings as f32;
            let y = radius * lat.cos();
            let ring_radius = radius * lat.sin();
            
            for s in 0..=sectors {
                let lon = 2.0 * std::f32::consts::PI * s as f32 / sectors as f32;
                let x = ring_radius * lon.cos();
                let z = ring_radius * lon.sin();
                
                vertices.push(Vertex::new(
                    center.x + x,
                    center.y + y,
                    center.z + z
                ));
            }
        }
        
        // Generate indices
        for r in 0..rings {
            for s in 0..sectors {
                let current = base_idx + r as u32 * (sectors as u32 + 1) + s as u32;
                let next = current + sectors as u32 + 1;
                
                indices.extend_from_slice(&[current, next, current + 1]);
                indices.extend_from_slice(&[current + 1, next, next + 1]);
            }
        }
    }
    
    fn add_hemisphere(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
                      center: Vec3, radius: f32, detail: i32) {
        let rings = 3 + detail * 2;
        let sectors = 6 + detail * 3;
        let base_idx = vertices.len() as u32;
        
        // Generate vertices (only top half)
        for r in 0..=rings {
            let lat = std::f32::consts::PI * 0.5 * r as f32 / rings as f32;
            let y = radius * lat.cos();
            let ring_radius = radius * lat.sin();
            
            for s in 0..=sectors {
                let lon = 2.0 * std::f32::consts::PI * s as f32 / sectors as f32;
                let x = ring_radius * lon.cos();
                let z = ring_radius * lon.sin();
                
                vertices.push(Vertex::new(
                    center.x + x,
                    center.y + y,
                    center.z + z
                ));
            }
        }
        
        // Generate indices
        for r in 0..rings {
            for s in 0..sectors {
                let current = base_idx + r as u32 * (sectors as u32 + 1) + s as u32;
                let next = current + sectors as u32 + 1;
                
                indices.extend_from_slice(&[current, next, current + 1]);
                indices.extend_from_slice(&[current + 1, next, next + 1]);
            }
        }
    }
    
    fn add_octahedron(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>,
                      center: Vec3, size: f32) {
        let base_idx = vertices.len() as u32;
        
        // 6 vertices of octahedron
        vertices.push(Vertex::new(center.x, center.y - size, center.z)); // 0: bottom
        vertices.push(Vertex::new(center.x, center.y + size, center.z)); // 1: top
        vertices.push(Vertex::new(center.x - size, center.y, center.z)); // 2: left
        vertices.push(Vertex::new(center.x + size, center.y, center.z)); // 3: right
        vertices.push(Vertex::new(center.x, center.y, center.z - size)); // 4: front
        vertices.push(Vertex::new(center.x, center.y, center.z + size)); // 5: back
        
        // 8 triangular faces
        let faces = [
            // Top half
            [1, 2, 4], [1, 4, 3], [1, 3, 5], [1, 5, 2],
            // Bottom half
            [0, 4, 2], [0, 3, 4], [0, 5, 3], [0, 2, 5],
        ];
        
        for face in &faces {
            indices.extend_from_slice(&[
                base_idx + face[0],
                base_idx + face[1],
                base_idx + face[2],
            ]);
        }
    }
    
    pub fn update_instances(&mut self, ctx: &mut dyn RenderingBackend, trees: &[Tree]) -> HashMap<TreeType, i32> {
        let mut trees_by_type: HashMap<TreeType, Vec<&Tree>> = HashMap::new();
        
        // Group trees by type
        for tree in trees {
            trees_by_type.entry(tree.tree_type)
                .or_insert_with(Vec::new)
                .push(tree);
        }
        
        let mut instance_counts = HashMap::new();
        
        // Update instance data for each tree type
        for (tree_type, trees) in trees_by_type {
            let mut instance_data = Vec::new();
            
            for tree in trees.iter().take(self.max_trees_per_type) {
                instance_data.push(TreeInstanceData {
                    position: [tree.pos.x, tree.pos.y, tree.pos.z],
                    scale_height: [tree.radius, tree.height, tree.radius],
                    rotation: tree.rotation,
                });
            }
            
            let count = instance_data.len() as i32;
            if count > 0 {
                if let Some(buffer) = self.instance_buffers.get(&tree_type) {
                    ctx.buffer_update(*buffer, BufferSource::slice(&instance_data));
                }
            }
            
            instance_counts.insert(tree_type, count);
        }
        
        instance_counts
    }
    
    pub fn get_buffers(&self, tree_type: TreeType) -> Option<(BufferId, BufferId, BufferId, i32)> {
        self.tree_buffers.get(&tree_type).and_then(|buffers| {
            self.instance_buffers.get(&tree_type).map(|instance_buffer| {
                (buffers.vertex_buffer, *instance_buffer, buffers.index_buffer, buffers.index_count)
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TreeInstanceData {
    position: [f32; 3],
    scale_height: [f32; 3], // width, height, depth scaling
    rotation: f32,
}