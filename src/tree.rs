use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::terrain;
use crate::biome::Biome;
use rand::prelude::*;

pub struct Tree {
    pub pos: Vec3,
    pub height: f32,
    pub radius: f32,
    pub tree_type: TreeType,
    pub rotation: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TreeType {
    Pine,      // Coniferous trees for mountains/arctic
    Oak,       // Deciduous trees for plains
    Palm,      // Desert/coastal trees
    Crystal,   // Crystalline formations
    Cactus,    // Desert vegetation
    Mushroom,  // Swamp vegetation
    Dead,      // Dead trees for badlands
}

impl Tree {
    pub fn new(x: f32, z: f32, tree_type: TreeType) -> Self {
        let mut rng = thread_rng();
        let height = terrain::height_at(x, z);
        
        // Vary tree size based on type
        let (tree_height, tree_radius) = match tree_type {
            TreeType::Pine => (30.0 + rng.gen::<f32>() * 20.0, 8.0 + rng.gen::<f32>() * 4.0),
            TreeType::Oak => (25.0 + rng.gen::<f32>() * 15.0, 15.0 + rng.gen::<f32>() * 5.0),
            TreeType::Palm => (20.0 + rng.gen::<f32>() * 15.0, 3.0 + rng.gen::<f32>() * 2.0),
            TreeType::Crystal => (15.0 + rng.gen::<f32>() * 25.0, 5.0 + rng.gen::<f32>() * 3.0),
            TreeType::Cactus => (10.0 + rng.gen::<f32>() * 10.0, 4.0 + rng.gen::<f32>() * 2.0),
            TreeType::Mushroom => (8.0 + rng.gen::<f32>() * 12.0, 10.0 + rng.gen::<f32>() * 5.0),
            TreeType::Dead => (15.0 + rng.gen::<f32>() * 10.0, 6.0 + rng.gen::<f32>() * 3.0),
        };
        
        Self {
            pos: Vec3::new(x, height, z),
            height: tree_height,
            radius: tree_radius,
            tree_type,
            rotation: rng.gen::<f32>() * std::f32::consts::PI * 2.0,
        }
    }
    
    pub fn should_spawn_at(x: f32, z: f32, biome: Biome) -> Option<TreeType> {
        let height = terrain::height_at(x, z);
        
        // Don't spawn trees underwater or on very steep slopes
        if height < -20.0 {
            return None;
        }
        
        // Calculate slope
        let delta = 5.0;
        let h_n = terrain::height_at(x, z + delta);
        let h_s = terrain::height_at(x, z - delta);
        let h_e = terrain::height_at(x + delta, z);
        let h_w = terrain::height_at(x - delta, z);
        let slope = ((h_n - h_s).abs() + (h_e - h_w).abs()) / (2.0 * delta);
        
        // Don't spawn on steep slopes
        if slope > 0.5 {
            return None;
        }
        
        // Select tree type based on biome and height
        match biome {
            Biome::Plains => {
                if height > 0.0 && height < 150.0 {
                    Some(TreeType::Oak)
                } else {
                    None
                }
            },
            Biome::Mountains => {
                if height > 100.0 && height < 350.0 {
                    Some(TreeType::Pine)
                } else {
                    None
                }
            },
            Biome::Desert => {
                if height > -20.0 && height < 100.0 {
                    Some(TreeType::Cactus)
                } else {
                    None
                }
            },
            Biome::Arctic => {
                if height > 50.0 && height < 250.0 {
                    Some(TreeType::Pine)
                } else {
                    None
                }
            },
            Biome::Swamp => {
                if height > -30.0 && height < 50.0 {
                    Some(TreeType::Mushroom)
                } else {
                    None
                }
            },
            Biome::Crystalline => {
                if height > 0.0 {
                    Some(TreeType::Crystal)
                } else {
                    None
                }
            },
            Biome::Badlands => {
                if height > 0.0 && height < 200.0 && rand::random::<f32>() < 0.3 {
                    Some(TreeType::Dead)
                } else {
                    None
                }
            },
            _ => None, // No trees in other biomes
        }
    }
}

impl Drawable for Tree {
    fn draw(&self, renderer: &mut Renderer) {
        match self.tree_type {
            TreeType::Pine => {
                // Draw coniferous tree - cone shape
                let levels = 3;
                for i in 0..levels {
                    let level_height = self.height * (1.0 - i as f32 * 0.3) / levels as f32;
                    let level_radius = self.radius * (1.0 - i as f32 * 0.3);
                    let y_offset = i as f32 * self.height * 0.3;
                    
                    renderer.draw_cone(
                        self.pos + Vec3::new(0.0, y_offset, 0.0),
                        level_radius,
                        level_height,
                        Vec3::new(0.1, 0.4, 0.1), // Dark green
                        8
                    );
                }
                // Trunk
                renderer.draw_cylinder(
                    self.pos,
                    self.radius * 0.3,
                    self.height * 0.3,
                    Vec3::new(0.4, 0.2, 0.1), // Brown
                    6
                );
            },
            TreeType::Oak => {
                // Draw deciduous tree - sphere canopy
                renderer.draw_sphere(
                    self.pos + Vec3::new(0.0, self.height * 0.6, 0.0),
                    self.radius,
                    Vec3::new(0.2, 0.5, 0.1), // Green
                    2
                );
                // Trunk
                renderer.draw_cylinder(
                    self.pos,
                    self.radius * 0.2,
                    self.height * 0.7,
                    Vec3::new(0.5, 0.3, 0.1), // Brown
                    8
                );
            },
            TreeType::Palm => {
                // Draw palm tree - thin trunk with fronds
                // Trunk
                renderer.draw_cylinder(
                    self.pos,
                    self.radius,
                    self.height,
                    Vec3::new(0.6, 0.4, 0.2), // Light brown
                    6
                );
                // Fronds
                let frond_count = 6;
                for i in 0..frond_count {
                    let angle = i as f32 * std::f32::consts::PI * 2.0 / frond_count as f32 + self.rotation;
                    let frond_dir = Vec3::new(angle.cos(), 0.3, angle.sin());
                    
                    renderer.draw_line(
                        self.pos + Vec3::new(0.0, self.height, 0.0),
                        self.pos + Vec3::new(0.0, self.height, 0.0) + frond_dir * self.radius * 3.0
                    );
                }
            },
            TreeType::Crystal => {
                // Draw crystal formation
                let crystal_count = 3 + (self.rotation * 2.0) as i32;
                for i in 0..crystal_count {
                    let offset_angle = i as f32 * std::f32::consts::PI * 2.0 / crystal_count as f32;
                    let offset = Vec3::new(
                        offset_angle.cos() * self.radius * 0.5,
                        0.0,
                        offset_angle.sin() * self.radius * 0.5
                    );
                    let crystal_height = self.height * (0.7 + i as f32 * 0.1);
                    
                    renderer.draw_octahedron(
                        self.pos + offset + Vec3::new(0.0, crystal_height * 0.5, 0.0),
                        crystal_height * 0.5,
                        crate::math::rotation_matrix(self.rotation + i as f32, 0.0, 0.0)
                    );
                }
            },
            TreeType::Cactus => {
                // Draw cactus - vertical cylinders
                renderer.draw_cylinder(
                    self.pos,
                    self.radius,
                    self.height,
                    Vec3::new(0.1, 0.5, 0.1), // Green
                    6
                );
                // Arms
                if self.height > 15.0 {
                    renderer.draw_cylinder(
                        self.pos + Vec3::new(self.radius, self.height * 0.6, 0.0),
                        self.radius * 0.7,
                        self.height * 0.3,
                        Vec3::new(0.1, 0.5, 0.1),
                        5
                    );
                    renderer.draw_cylinder(
                        self.pos + Vec3::new(-self.radius, self.height * 0.5, 0.0),
                        self.radius * 0.7,
                        self.height * 0.4,
                        Vec3::new(0.1, 0.5, 0.1),
                        5
                    );
                }
            },
            TreeType::Mushroom => {
                // Draw mushroom - dome cap on cylinder
                renderer.draw_hemisphere(
                    self.pos + Vec3::new(0.0, self.height, 0.0),
                    self.radius,
                    Vec3::new(0.6, 0.1, 0.1), // Red cap
                    2
                );
                // Stem
                renderer.draw_cylinder(
                    self.pos,
                    self.radius * 0.3,
                    self.height,
                    Vec3::new(0.8, 0.8, 0.7), // Light stem
                    8
                );
                // Spots
                let spot_count = 5 + (self.rotation * 3.0) as i32;
                for i in 0..spot_count {
                    let angle = i as f32 * 2.3 + self.rotation; // Irregular spacing
                    let spot_pos = self.pos + Vec3::new(
                        angle.cos() * self.radius * 0.7,
                        self.height + self.radius * 0.3,
                        angle.sin() * self.radius * 0.7
                    );
                    renderer.draw_sphere(spot_pos, self.radius * 0.1, Vec3::new(0.9, 0.9, 0.9), 1);
                }
            },
            TreeType::Dead => {
                // Draw dead tree - bare branches
                renderer.draw_cylinder(
                    self.pos,
                    self.radius * 0.3,
                    self.height * 0.8,
                    Vec3::new(0.3, 0.2, 0.1), // Dark brown
                    6
                );
                // Branches
                let branch_count = 4;
                for i in 0..branch_count {
                    let branch_height = self.height * (0.4 + i as f32 * 0.15);
                    let angle = self.rotation + i as f32 * 1.5;
                    let branch_end = self.pos + Vec3::new(
                        angle.cos() * self.radius * 1.5,
                        branch_height + self.radius,
                        angle.sin() * self.radius * 1.5
                    );
                    renderer.draw_line(
                        self.pos + Vec3::new(0.0, branch_height, 0.0),
                        branch_end
                    );
                }
            },
        }
    }
}