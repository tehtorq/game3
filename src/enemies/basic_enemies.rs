use glam::Vec3;
use crate::renderer::{Renderer, Drawable};
use crate::math::rotation_matrix;
use super::{Enemy, EnemyType, BaseEnemyData, AlertState, MovementPattern};
use super::ai::AIController;
use super::movement::MovementController;

// Cube Enemy - Basic hovering enemy
pub struct CubeEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl CubeEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = crate::terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Cube, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Hover, 250.0, base.spawn_point);
        
        Self { base, ai, movement }
    }
}

impl Enemy for CubeEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Cube }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        self.movement.update(
            &mut self.base.pos,
            &mut self.base.vel,
            self.base.spawn_point,
            self.base.phase,
            player_pos,
            self.ai.alert_state,
            self.ai.last_known_player_pos,
            self.ai.investigation_point,
            self.ai.aggression,
            self.ai.preferred_combat_distance,
            dt
        );
        
        if self.ai.should_attack(&EnemyType::Cube) {
            let to_player = player_pos - self.base.pos;
            Some(to_player.normalize_or_zero())
        } else {
            None
        }
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for CubeEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Basic cube with decorative elements
        renderer.draw_cube(self.base.pos, 40.0, rotation);
        
        // Add smaller rotating cube inside
        let inner_rotation = rotation_matrix(
            self.base.rotation.y * -2.0, 
            self.base.rotation.x * -2.0, 
            0.0
        );
        renderer.draw_cube(self.base.pos, 20.0, inner_rotation);
    }
}

// Pyramid Enemy - Fast diving enemy
pub struct PyramidEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl PyramidEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = crate::terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Pyramid, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Diving, 400.0, base.spawn_point);
        
        Self { base, ai, movement }
    }
}

impl Enemy for PyramidEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Pyramid }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        self.movement.update(
            &mut self.base.pos,
            &mut self.base.vel,
            self.base.spawn_point,
            self.base.phase,
            player_pos,
            self.ai.alert_state,
            self.ai.last_known_player_pos,
            self.ai.investigation_point,
            self.ai.aggression,
            self.ai.preferred_combat_distance,
            dt
        );
        
        if self.ai.should_attack(&EnemyType::Pyramid) {
            let to_player = player_pos - self.base.pos;
            Some(to_player.normalize_or_zero())
        } else {
            None
        }
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for PyramidEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Double pyramid (octahedron)
        renderer.draw_octahedron(self.base.pos, 35.0, rotation);
        
        // Add spinning blades
        let blade_rotation = rotation_matrix(self.base.phase * 4.0, 0.0, 0.0);
        renderer.draw_pyramid(self.base.pos, 25.0, blade_rotation);
    }
}

// Spinner Enemy - Complex orbital movement
pub struct SpinnerEnemy {
    base: BaseEnemyData,
    ai: AIController,
    movement: MovementController,
}

impl SpinnerEnemy {
    pub fn new(x: f32, z: f32) -> Self {
        let height = crate::terrain::height_at(x, z) + rand::random::<f32>() * 70.0 + 30.0;
        let base = BaseEnemyData::new(x, height, z);
        let ai = AIController::new(&EnemyType::Spinner, base.spawn_point);
        let movement = MovementController::new(MovementPattern::Orbital, 300.0, base.spawn_point);
        
        Self { base, ai, movement }
    }
}

impl Enemy for SpinnerEnemy {
    fn get_type(&self) -> EnemyType { EnemyType::Spinner }
    fn get_position(&self) -> Vec3 { self.base.pos }
    fn set_position(&mut self, pos: Vec3) { self.base.pos = pos; }
    fn get_velocity(&self) -> Vec3 { self.base.vel }
    fn set_velocity(&mut self, vel: Vec3) { self.base.vel = vel; }
    
    fn update(&mut self, player_pos: Vec3, dt: f32) -> Option<Vec3> {
        self.base.update_rotation(dt);
        
        self.ai.update(self.base.pos, self.base.vel, player_pos, dt);
        
        self.movement.update(
            &mut self.base.pos,
            &mut self.base.vel,
            self.base.spawn_point,
            self.base.phase,
            player_pos,
            self.ai.alert_state,
            self.ai.last_known_player_pos,
            self.ai.investigation_point,
            self.ai.aggression,
            self.ai.preferred_combat_distance,
            dt
        );
        
        if self.ai.should_attack(&EnemyType::Spinner) {
            let to_player = player_pos - self.base.pos;
            Some(to_player.normalize_or_zero())
        } else {
            None
        }
    }
    
    fn get_alert_state(&self) -> AlertState { self.ai.alert_state }
    fn get_health(&self) -> f32 { self.base.health }
    fn take_damage(&mut self, damage: f32) -> bool {
        self.base.health -= damage;
        self.base.health <= 0.0
    }
    
    fn set_patrol_route(&mut self, waypoints: Vec<Vec3>) {
        self.movement.set_patrol_route(waypoints);
    }
}

impl Drawable for SpinnerEnemy {
    fn draw(&self, renderer: &mut Renderer) {
        let rotation = rotation_matrix(self.base.rotation.y, self.base.rotation.x, self.base.rotation.z);
        
        // Central octahedron
        renderer.draw_octahedron(self.base.pos, 25.0, rotation);
        
        // Three rotating rings
        let ring1 = rotation_matrix(self.base.rotation.y * 2.0, 0.0, 0.0);
        let ring2 = rotation_matrix(0.0, self.base.rotation.x * 2.0, 0.0);
        let ring3 = rotation_matrix(0.0, 0.0, self.base.rotation.z * 2.0);
        
        renderer.draw_cube(self.base.pos, 35.0, ring1);
        renderer.draw_cube(self.base.pos, 30.0, ring2);
        renderer.draw_cube(self.base.pos, 25.0, ring3);
    }
}