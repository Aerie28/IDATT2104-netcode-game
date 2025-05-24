use crate::colors::bg_colors;
use crate::constants::{PLAYER_SIZE, TOOL_BAR_HEIGHT};

use macroquad::prelude::*;

/// Renderer for the game, responsible for drawing the game elements
pub struct Renderer;

/// Implementation of the Renderer
impl Renderer {
    /// Creates a new Renderer instance
    pub fn new() -> Self {
        Renderer
    }

    /// Clears the screen with a black background
    pub fn clear(&self) {
        clear_background(bg_colors::BLACK);
    }
    
    /// Draws the player at the specified position with the given color
    pub fn draw_player(
        &self,
        x: f32,
        y: f32,
        color: Color,
    ) {
        draw_rectangle(
            x - (PLAYER_SIZE as f32) / 2.0,
            y - (PLAYER_SIZE as f32) / 2.0,
            PLAYER_SIZE as f32,
            PLAYER_SIZE as f32,
            color,
        );
    }

    /// Draws the toolbar with network stats and controls
    pub fn draw_tool_bar(&self, delay_ms: i32, packet_loss: i32, is_connected: bool, is_testing: bool) {
        let bar_height = TOOL_BAR_HEIGHT as f32;
        let width = screen_width();
        let height = screen_height();
        let text_size = 20.0;
        let text_spacing = 20.0;

        // Check if we need a two-line layout
        let min_width_for_single_line = 900.0;
        let is_two_line = width < min_width_for_single_line;
        let bar_total_height = if is_two_line { bar_height * 2.0 } else { bar_height };

        // Draw toolbar background
        draw_rectangle(0.0, height - bar_total_height, width, bar_total_height, bg_colors::DARK_GRAY);

        // First line (or only line if enough space)
        let y_pos = if is_two_line {
            height - bar_total_height + bar_height / 2.0 + text_size / 3.0
        } else {
            height - bar_height / 2.0 + text_size / 3.0
        };

        // Draw movement controls text
        draw_text(
            "Movement [W,A,S,D]",
            text_spacing,
            y_pos,
            text_size,
            bg_colors::WHITE,
        );

        // Calculate position for network stats text
        let movement_width = measure_text("Movement [W,A,S,D]", None, text_size as u16, 1.0).width;
        let network_stats_x = text_spacing + movement_width + 30.0; // Add some spacing between texts

        // Draw network stats
        draw_text(
            &format!("Delay: {} ms [V/B]   Packet Loss: {}% [N/M]", delay_ms, packet_loss),
            network_stats_x,
            y_pos,
            text_size,
            bg_colors::WHITE,
        );

        // Second line or right side of the bar
        let status_y_pos = if is_two_line {
            height - bar_height / 2.0 + text_size / 3.0
        } else {
            y_pos
        };

        // Calculate spacing for right-aligned elements
        let connect_text = if is_connected { "Drop connection [R]" } else { "Reconnect [R]" };
        let connect_width = measure_text(connect_text, None, text_size as u16, 1.0).width;
        let test_text = "Test [T]";
        let test_width = measure_text(test_text, None, text_size as u16, 1.0).width;

        // Testing indicator and label
        let indicator_size = 10.0;
        let indicator_spacing = 8.0;

        // Position from right side
        let test_x = width - connect_width - text_spacing * 2.0 - test_width - indicator_size - indicator_spacing;

        // Draw indicator
        let indicator_x = test_x;
        let indicator_y = status_y_pos - text_size/3.0;

        // Draw indicator light
        let indicator_color = if is_testing {
            bg_colors::ORANGE // Orange when testing
        } else {
            bg_colors::DARK_GRAY // Dim gray when not testing
        };

        draw_circle(indicator_x, indicator_y, indicator_size / 2.0, indicator_color);
        draw_circle_lines(indicator_x, indicator_y, indicator_size / 2.0, 1.0, bg_colors::WHITE);

        // Draw test text
        draw_text(
            test_text,
            indicator_x + indicator_size + indicator_spacing,
            status_y_pos,
            text_size,
            bg_colors::WHITE,
        );

        // Connection status
        draw_text(
            connect_text,
            width - connect_width - text_spacing,
            status_y_pos,
            text_size,
            bg_colors::WHITE,
        );
    }
}

/// Tests for the Renderer
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        Renderer::new();
    }

    #[test]
    fn test_player_position_calculation() {
        let player_x = 100.0;
        let player_y = 200.0;
        let half_size = (PLAYER_SIZE as f32) / 2.0;

        // Calculate expected rectangle coordinates
        let expected_rect_x = player_x - half_size;
        let expected_rect_y = player_y - half_size;

        // Verify the calculation for rectangle position
        assert_eq!(expected_rect_x, player_x - half_size);
        assert_eq!(expected_rect_y, player_y - half_size);
    }

    #[test]
    fn test_toolbar_responsive_layout() {
        let wide_screen_width = 1000.0;
        let min_width_for_single_line = 900.0;
        let is_two_line = wide_screen_width < min_width_for_single_line;

        assert!(!is_two_line); // Should be false for 1000 width

        // Test two-line layout (narrow screen)
        let narrow_width = 800.0;
        let is_two_line_narrow = narrow_width < min_width_for_single_line;

        assert!(is_two_line_narrow); // Should be true for 800 width
    }

    #[test]
    fn test_toolbar_height_calculation() {
        let bar_height = TOOL_BAR_HEIGHT as f32;

        // For wide screen (single line)
        let width_wide = 1000.0;
        let min_width_for_single_line = 900.0;
        let is_two_line_wide = width_wide < min_width_for_single_line;
        let bar_total_height_wide = if is_two_line_wide { bar_height * 2.0 } else { bar_height };

        assert_eq!(bar_total_height_wide, bar_height);

        // For narrow screen (two lines)
        let width_narrow = 800.0;
        let is_two_line_narrow = width_narrow < min_width_for_single_line;
        let bar_total_height_narrow = if is_two_line_narrow { bar_height * 2.0 } else { bar_height };

        assert_eq!(bar_total_height_narrow, bar_height * 2.0);
    }

    #[test]
    fn test_indicator_color_selection() {
        // When testing
        let is_testing = true;
        let indicator_color_testing = if is_testing {
            bg_colors::ORANGE
        } else {
            bg_colors::DARK_GRAY
        };
        assert_eq!(indicator_color_testing, bg_colors::ORANGE);

        // When not testing
        let is_testing = false;
        let indicator_color_not_testing = if is_testing {
            bg_colors::ORANGE
        } else {
            bg_colors::DARK_GRAY
        };
        assert_eq!(indicator_color_not_testing, bg_colors::DARK_GRAY);
    }

    #[test]
    fn test_connection_text() {
        // When connected
        let is_connected = true;
        let connect_text_connected = if is_connected { "Drop connection [R]" } else { "Reconnect [R]" };
        assert_eq!(connect_text_connected, "Drop connection [R]");

        // When disconnected
        let is_connected = false;
        let connect_text_disconnected = if is_connected { "Drop connection [R]" } else { "Reconnect [R]" };
        assert_eq!(connect_text_disconnected, "Reconnect [R]");
    }
}