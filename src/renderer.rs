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
}