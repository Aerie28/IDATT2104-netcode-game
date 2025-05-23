use macroquad::prelude::*;

// Background/UI colors
pub mod bg_colors {
    use super::*;
    pub const BLACK: Color = Color::from_hex(0x171717);
    pub const DARK_GRAY: Color = Color::from_hex(0x303030);
    pub const WHITE: Color = Color::from_hex(0xfcfcfc);
    pub const RED: Color = Color::from_hex(0xff1717);
    pub const GREEN: Color = Color::from_hex(0x17ff17);

    pub const ORANGE: Color = Color::from_hex(0xe65c07);
}

// Player colors
pub mod player_colors {
    use super::*;
    pub const RED: Color = Color::from_hex(0xff1717);
    pub const GREEN: Color = Color::from_hex(0x17ff17);
    pub const BLUE: Color = Color::from_hex(0x1717ff);
    pub const YELLOW: Color = Color::from_hex(0xffff17);
    pub const ORANGE: Color = Color::from_hex(0xff7f17);
    pub const PURPLE: Color = Color::from_hex(0x7f17ff);
    pub const CYAN: Color = Color::from_hex(0x17ffff);
    pub const MAGENTA: Color = Color::from_hex(0xff17ff);
    pub const PINK: Color = Color::from_hex(0xff7f7f);
    
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