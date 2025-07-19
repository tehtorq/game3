use miniquad::*;
use super::state::InputState;

pub struct InputHandler;

impl InputHandler {
    pub fn handle_key_down(input: &mut InputState, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::W | KeyCode::Up => input.forward = true,
            KeyCode::S | KeyCode::Down => input.backward = true,
            KeyCode::A | KeyCode::Left => input.left = true,
            KeyCode::D | KeyCode::Right => input.right = true,
            KeyCode::Space => input.up = true,
            KeyCode::LeftShift => input.down = true,
            KeyCode::LeftControl => input.shoot = true,
            KeyCode::Tab => input.boost = true,
            _ => return false,
        }
        true
    }
    
    pub fn handle_key_up(input: &mut InputState, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::W | KeyCode::Up => input.forward = false,
            KeyCode::S | KeyCode::Down => input.backward = false,
            KeyCode::A | KeyCode::Left => input.left = false,
            KeyCode::D | KeyCode::Right => input.right = false,
            KeyCode::Space => input.up = false,
            KeyCode::LeftShift => input.down = false,
            KeyCode::LeftControl => input.shoot = false,
            KeyCode::Tab => input.boost = false,
            _ => return false,
        }
        true
    }
    
    pub fn handle_mouse_motion(
        input: &mut InputState, 
        x: f32, 
        y: f32, 
        screen_width: f32, 
        screen_height: f32,
        mouse_captured: bool
    ) {
        if mouse_captured {
            // Convert to normalized coordinates (-1 to 1)
            input.mouse_target_x = (x - screen_width / 2.0) / (screen_width / 2.0);
            input.mouse_target_y = -(y - screen_height / 2.0) / (screen_height / 2.0);
            
            // Apply smoothing and sensitivity
            input.mouse_target_x *= 0.5;
            input.mouse_target_y *= 0.5;
            
            // Clamp values
            input.mouse_target_x = input.mouse_target_x.clamp(-1.0, 1.0);
            input.mouse_target_y = input.mouse_target_y.clamp(-1.0, 1.0);
        }
    }
    
    pub fn handle_mouse_button(
        input: &mut InputState,
        button: MouseButton,
        pressed: bool
    ) -> bool {
        match button {
            MouseButton::Left => {
                input.shoot = pressed;
                true
            }
            MouseButton::Right => {
                input.boost = pressed;
                true
            }
            _ => false,
        }
    }
}