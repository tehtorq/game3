#[derive(Default)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
    pub shoot: bool,
    pub boost: bool,
    pub up: bool,
    pub down: bool,
    pub mouse_target_x: f32,  // Mouse position relative to center (-1 to 1)
    pub mouse_target_y: f32,  // Mouse position relative to center (-1 to 1)
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    pub fn is_moving(&self) -> bool {
        self.left || self.right || self.forward || self.backward || self.up || self.down
    }
    
    pub fn get_movement_vector(&self) -> (f32, f32, f32) {
        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;
        
        if self.left { dx -= 1.0; }
        if self.right { dx += 1.0; }
        if self.forward { dz += 1.0; }
        if self.backward { dz -= 1.0; }
        if self.up { dy += 1.0; }
        if self.down { dy -= 1.0; }
        
        (dx, dy, dz)
    }
}