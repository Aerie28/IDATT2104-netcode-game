use macroquad::prelude::*;
use crate::colors::bg_colors;
use crate::constants::{PLAYER_SIZE};
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
        draw_rectangle(
            x - PLAYER_SIZE / 2.0,
            y - PLAYER_SIZE / 2.0,
            PLAYER_SIZE,
            PLAYER_SIZE,
            color,
        );
    }
}