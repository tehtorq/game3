use miniquad::*;

pub trait PostProcessingEffect {
    fn new(ctx: &mut Context, screen_width: u32, screen_height: u32) -> Self where Self: Sized;
    fn is_enabled(&self) -> bool;
    fn toggle(&mut self);
    fn apply_effect(
        &mut self,
        ctx: &mut Context,
        screen_width: f32,
        screen_height: f32,
        input_texture: TextureId,
    );
    fn set_input_texture(&mut self, texture: TextureId);
    fn get_output_texture(&self) -> TextureId;
    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32);
}
