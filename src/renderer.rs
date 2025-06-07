use glam::{Vec3, Mat4};
use crate::vertex::Vertex;

pub trait Drawable {
    fn draw(&self, renderer: &mut Renderer);
}

pub struct Renderer<'a> {
    pub vertices: &'a mut Vec<Vertex>,
    pub indices: &'a mut Vec<u16>,
}

impl<'a> Renderer<'a> {
    pub fn new(vertices: &'a mut Vec<Vertex>, indices: &'a mut Vec<u16>) -> Self {
        Self { vertices, indices }
    }

    pub fn draw_line(&mut self, from: Vec3, to: Vec3) {
        let base_idx = self.vertices.len() as u16;
        self.vertices.push(Vertex::new(from.x, from.y, from.z));
        self.vertices.push(Vertex::new(to.x, to.y, to.z));
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
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

        // Draw cube edges
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // front face
            (4, 5), (5, 6), (6, 7), (7, 4), // back face
            (0, 4), (1, 5), (2, 6), (3, 7), // connecting edges
        ];

        for &(i, j) in &edges {
            self.draw_line(transformed[i], transformed[j]);
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

        // Draw base
        for i in 0..4 {
            self.draw_line(transformed[i], transformed[(i + 1) % 4]);
        }

        // Draw edges to apex
        for i in 0..4 {
            self.draw_line(transformed[i], transformed[4]);
        }
    }
}