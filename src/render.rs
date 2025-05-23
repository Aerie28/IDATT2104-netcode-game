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

    pub fn draw_tool_bar(&self, delay_ms: i32, packet_loss: i32, is_connected: bool) {
        let bar_height = 40.0;
        let width = screen_width();
        let height = screen_height();
        draw_rectangle(0.0, height - bar_height, width, bar_height, bg_colors::DARK_GRAY);
        let text = if is_connected { "Disconnect [R]" } else { "Connect [R]" };
        
        // Draw network stats
        draw_text(
            &format!("Delay: {} ms  [V/B]   Packet Loss: {}%  [N/M]   Movement [W,A,S,D]   {}", delay_ms, packet_loss, text),
            20.0,
            height - bar_height / 2.0 + 10.0,
            24.0,
            bg_colors::WHITE,
        );
        
    }
}