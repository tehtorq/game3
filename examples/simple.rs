use miniquad::*;

struct Stage;

impl EventHandler for Stage {
    fn update(&mut self) {}
    fn draw(&mut self) {
        // This will help us understand the API
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), || Box::new(Stage));
}