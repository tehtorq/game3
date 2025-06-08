use glam::{Vec3, Mat4};
use crate::vertex::Vertex;

pub trait Drawable {
    fn draw(&self, renderer: &mut Renderer);
}

pub struct Renderer<'a> {
    pub vertices: &'a mut Vec<Vertex>,
    pub indices: &'a mut Vec<u32>,
    pub mode: RenderMode,
}

#[derive(Clone, Copy)]
pub enum RenderMode {
    Lines,
    Triangles,
}

impl<'a> Renderer<'a> {
    pub fn new(vertices: &'a mut Vec<Vertex>, indices: &'a mut Vec<u32>) -> Self {
        Self { vertices, indices, mode: RenderMode::Lines }
    }

    pub fn set_mode(&mut self, mode: RenderMode) {
        self.mode = mode;
    }

    pub fn draw_line(&mut self, from: Vec3, to: Vec3) {
        let base_idx = self.vertices.len() as u32;
        // Use zero barycentric coordinates for lines
        self.vertices.push(Vertex::new(from.x, from.y, from.z));
        self.vertices.push(Vertex::new(to.x, to.y, to.z));
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
    }

    pub fn draw_triangle(&mut self, p1: Vec3, p2: Vec3, p3: Vec3) {
        let base_idx = self.vertices.len() as u32;
        
        // Add vertices with barycentric coordinates
        self.vertices.push(Vertex::with_barycentric(p1.x, p1.y, p1.z, 1.0, 0.0, 0.0));
        self.vertices.push(Vertex::with_barycentric(p2.x, p2.y, p2.z, 0.0, 1.0, 0.0));
        self.vertices.push(Vertex::with_barycentric(p3.x, p3.y, p3.z, 0.0, 0.0, 1.0));
        
        // Add triangle indices
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);
    }

    pub fn draw_cube(&mut self, center: Vec3, size: f32, rotation: Mat4) {
        let half = size / 2.0;
        let corners = [
            Vec3::new(-half, -half, -half),
            Vec3::new( half, -half, -half),
            Vec3::new( half,  half, -half),
            Vec3::new(-half,  half, -half),
            Vec3::new(-half, -half,  half),
            Vec3::new( half, -half,  half),
            Vec3::new( half,  half,  half),
            Vec3::new(-half,  half,  half),
        ];

        let transformed: Vec<Vec3> = corners.iter()
            .map(|&c| center + rotation.transform_vector3(c))
            .collect();

        // Draw cube as triangles for solid rendering
        let faces = [
            // Front face
            [4, 5, 6], [4, 6, 7],
            // Back face
            [1, 0, 3], [1, 3, 2],
            // Top face
            [7, 6, 2], [7, 2, 3],
            // Bottom face
            [0, 1, 5], [0, 5, 4],
            // Right face
            [5, 1, 2], [5, 2, 6],
            // Left face
            [0, 4, 7], [0, 7, 3],
        ];

        for face in &faces {
            self.draw_triangle(
                transformed[face[0]], 
                transformed[face[1]], 
                transformed[face[2]]
            );
        }
    }

    pub fn draw_pyramid(&mut self, center: Vec3, size: f32, rotation: Mat4) {
        let half = size / 2.0;
        let points = [
            Vec3::new(-half, 0.0, -half),
            Vec3::new( half, 0.0, -half),
            Vec3::new( half, 0.0,  half),
            Vec3::new(-half, 0.0,  half),
            Vec3::new(0.0, -size, 0.0),
        ];

        let transformed: Vec<Vec3> = points.iter()
            .map(|&p| center + rotation.transform_vector3(p))
            .collect();

        // Draw pyramid as triangles
        // Base
        self.draw_triangle(transformed[0], transformed[1], transformed[2]);
        self.draw_triangle(transformed[0], transformed[2], transformed[3]);
        
        // Sides
        self.draw_triangle(transformed[0], transformed[1], transformed[4]);
        self.draw_triangle(transformed[1], transformed[2], transformed[4]);
        self.draw_triangle(transformed[2], transformed[3], transformed[4]);
        self.draw_triangle(transformed[3], transformed[0], transformed[4]);
    }
    
    pub fn draw_octahedron(&mut self, center: Vec3, size: f32, rotation: Mat4) {
        let points = [
            Vec3::new(0.0, -size, 0.0),  // Bottom
            Vec3::new(0.0, size, 0.0),   // Top
            Vec3::new(-size, 0.0, 0.0),  // Left
            Vec3::new(size, 0.0, 0.0),   // Right
            Vec3::new(0.0, 0.0, -size),  // Front
            Vec3::new(0.0, 0.0, size),   // Back
        ];
        
        let transformed: Vec<Vec3> = points.iter()
            .map(|&p| center + rotation.transform_vector3(p))
            .collect();
        
        // Draw 8 triangular faces
        // Top half
        self.draw_triangle(transformed[1], transformed[2], transformed[4]);
        self.draw_triangle(transformed[1], transformed[4], transformed[3]);
        self.draw_triangle(transformed[1], transformed[3], transformed[5]);
        self.draw_triangle(transformed[1], transformed[5], transformed[2]);
        
        // Bottom half
        self.draw_triangle(transformed[0], transformed[4], transformed[2]);
        self.draw_triangle(transformed[0], transformed[3], transformed[4]);
        self.draw_triangle(transformed[0], transformed[5], transformed[3]);
        self.draw_triangle(transformed[0], transformed[2], transformed[5]);
    }
    
    pub fn draw_hexagon_prism(&mut self, center: Vec3, size: f32, height: f32, rotation: Mat4) {
        let angle_step = std::f32::consts::PI * 2.0 / 6.0;
        let mut top_points = Vec::new();
        let mut bottom_points = Vec::new();
        
        for i in 0..6 {
            let angle = i as f32 * angle_step;
            let x = angle.cos() * size;
            let z = angle.sin() * size;
            top_points.push(Vec3::new(x, height / 2.0, z));
            bottom_points.push(Vec3::new(x, -height / 2.0, z));
        }
        
        // Transform all points
        let top_transformed: Vec<Vec3> = top_points.iter()
            .map(|&p| center + rotation.transform_vector3(p))
            .collect();
        let bottom_transformed: Vec<Vec3> = bottom_points.iter()
            .map(|&p| center + rotation.transform_vector3(p))
            .collect();
        
        // Draw top and bottom faces
        for i in 0..6 {
            let next = (i + 1) % 6;
            // Top face
            self.draw_triangle(center + rotation.transform_vector3(Vec3::new(0.0, height / 2.0, 0.0)), 
                             top_transformed[i], 
                             top_transformed[next]);
            // Bottom face
            self.draw_triangle(center + rotation.transform_vector3(Vec3::new(0.0, -height / 2.0, 0.0)), 
                             bottom_transformed[next], 
                             bottom_transformed[i]);
            // Side faces
            self.draw_triangle(top_transformed[i], bottom_transformed[i], bottom_transformed[next]);
            self.draw_triangle(top_transformed[i], bottom_transformed[next], top_transformed[next]);
        }
    }
    
    pub fn draw_spike_ball(&mut self, center: Vec3, core_size: f32, spike_length: f32, rotation: Mat4) {
        // Draw central octahedron
        self.draw_octahedron(center, core_size, rotation);
        
        // Add spikes at each vertex
        let spike_positions = [
            Vec3::new(0.0, -1.0, 0.0),  // Bottom
            Vec3::new(0.0, 1.0, 0.0),   // Top
            Vec3::new(-1.0, 0.0, 0.0),  // Left
            Vec3::new(1.0, 0.0, 0.0),   // Right
            Vec3::new(0.0, 0.0, -1.0),  // Front
            Vec3::new(0.0, 0.0, 1.0),   // Back
        ];
        
        for &spike_dir in &spike_positions {
            let spike_base = center + rotation.transform_vector3(spike_dir * core_size);
            let spike_tip = center + rotation.transform_vector3(spike_dir * (core_size + spike_length));
            
            // Create spike as a small pyramid
            let perpendicular1 = if spike_dir.y.abs() > 0.5 {
                Vec3::new(1.0, 0.0, 0.0)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            let perpendicular2 = spike_dir.cross(perpendicular1).normalize();
            let perpendicular1 = spike_dir.cross(perpendicular2).normalize();
            
            let spike_size = core_size * 0.3;
            let base1 = spike_base + rotation.transform_vector3(perpendicular1 * spike_size);
            let base2 = spike_base + rotation.transform_vector3(perpendicular2 * spike_size);
            let base3 = spike_base - rotation.transform_vector3(perpendicular1 * spike_size);
            let base4 = spike_base - rotation.transform_vector3(perpendicular2 * spike_size);
            
            self.draw_triangle(base1, base2, spike_tip);
            self.draw_triangle(base2, base3, spike_tip);
            self.draw_triangle(base3, base4, spike_tip);
            self.draw_triangle(base4, base1, spike_tip);
        }
    }
}