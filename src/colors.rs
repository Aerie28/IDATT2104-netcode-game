use macroquad::prelude::*;

/// Background/UI colors
pub mod bg_colors {
    use super::*;
    pub const BLACK: Color = Color::from_hex(0x171717); // Dark background
    pub const DARK_GRAY: Color = Color::from_hex(0x303030); // Dark gray for UI elements
    pub const WHITE: Color = Color::from_hex(0xfcfcfc); // Light gray / off-white for text and highlights
    pub const RED: Color = Color::from_hex(0xff1717); // Bright red for errors or highlights
    pub const GREEN: Color = Color::from_hex(0x17ff17); // Bright green for success or highlights
    pub const ORANGE: Color = Color::from_hex(0xe65c07); // Bright orange for warnings or highlights
}

/// Player colors
pub mod player_colors {
    use super::*;
    pub const RED: Color = Color::from_hex(0xff1717); // Bright red
    pub const GREEN: Color = Color::from_hex(0x17ff17); // Bright green
    pub const BLUE: Color = Color::from_hex(0x1717ff); // Bright blue
    pub const YELLOW: Color = Color::from_hex(0xffff17); // Bright yellow
    pub const ORANGE: Color = Color::from_hex(0xff7f17); // Bright orange
    pub const PURPLE: Color = Color::from_hex(0x7f17ff); // Bright purple
    pub const CYAN: Color = Color::from_hex(0x17ffff); // Bright cyan
    pub const MAGENTA: Color = Color::from_hex(0xff17ff); // Bright magenta
    pub const PINK: Color = Color::from_hex(0xff7f7f); // Bright pink
    
    /// Returns a vector of all player colors as a palette
    pub fn get_palette() -> Vec<Color> {
        vec![
            RED,
            GREEN,
            BLUE,
            YELLOW,
            ORANGE,
            PURPLE,
            CYAN,
            MAGENTA,
            PINK,
        ]
    }
}

/// Tests for the color module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_color_palette() {
        // Test that the palette returns the correct number of colors
        let palette = player_colors::get_palette();
        assert_eq!(palette.len(), 9);

        // Test that the colors in the palette match the constants
        assert_eq!(palette[0], player_colors::RED);
        assert_eq!(palette[1], player_colors::GREEN);
        assert_eq!(palette[8], player_colors::PINK);
    }
}