use macroquad::prelude::*;
use crate::colors::bg_colors;
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn clear(&self) {
        clear_background(bg_colors::BLACK);
    }
    
    pub fn draw_player(
        &self,
        x: f32,
        y: f32,
        color: Color,
    ) {
        // Draw black border
        draw_rectangle(x, y, 32.0, 32.0, bg_colors::WHITE);
        // Draw colored rectangle (slightly larger for border effect)
        draw_rectangle(x - 2.0, y - 2.0, 36.0, 36.0, color);
    }
}