use miniquad::*;
use super::state::AppState;
use crate::input::InputHandler;

pub struct EventHandler;

impl EventHandler {
    pub fn handle_key_down(app: &mut AppState, keycode: KeyCode) {
        match keycode {
            KeyCode::Escape => {
                if app.mouse_captured {
                    app.release_mouse();
                } else {
                    std::process::exit(0);
                }
            }
            KeyCode::F => app.toggle_fullscreen(),
            KeyCode::P => app.toggle_pause(),
            _ => {
                InputHandler::handle_key_down(&mut app.input, keycode);
            }
        }
    }
    
    pub fn handle_key_up(app: &mut AppState, keycode: KeyCode) {
        InputHandler::handle_key_up(&mut app.input, keycode);
    }
    
    pub fn handle_mouse_motion(app: &mut AppState, x: f32, y: f32) {
        let (width, height) = window::screen_size();
        InputHandler::handle_mouse_motion(&mut app.input, x, y, width, height, app.mouse_captured);
    }
    
    pub fn handle_mouse_button(app: &mut AppState, button: MouseButton, pressed: bool) {
        if !app.mouse_captured && pressed {
            app.capture_mouse();
        } else {
            InputHandler::handle_mouse_button(&mut app.input, button, pressed);
        }
    }
}