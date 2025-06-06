use crate::constants::{WINDOW_HEIGHT, WINDOW_RESIZABLE, WINDOW_TITLE, WINDOW_WIDTH};

use image::imageops::FilterType;
use miniquad::conf::{Conf, Icon};

/// Configuration for the game window
pub fn config_window() -> Conf {
    let icon_bytes = include_bytes!("assets/icon.png");
    let image = image::load_from_memory(icon_bytes).unwrap();

    // Resize the icon to different sizes
    let small = image.resize_exact(16, 16, FilterType::Lanczos3).into_rgba8().into_raw();
    let medium = image.resize_exact(32, 32, FilterType::Lanczos3).into_rgba8().into_raw();
    let big = image.resize_exact(64, 64, FilterType::Lanczos3).into_rgba8().into_raw();

    let icon = Icon {
        small: small.try_into().unwrap(),
        medium: medium.try_into().unwrap(),
        big: big.try_into().unwrap(),
    };

    // Create the configuration for the game window
    Conf {
        window_title: WINDOW_TITLE.to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: WINDOW_RESIZABLE,
        icon: Some(icon),
        ..Default::default()
    }
}

/// Unit test to ensure the configuration is correct
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_window() {
        // Verify the function doesn't panic
        let conf = config_window();

        // Verify configuration values match constants
        assert_eq!(conf.window_title, WINDOW_TITLE);
        assert_eq!(conf.window_width, WINDOW_WIDTH);
        assert_eq!(conf.window_height, WINDOW_HEIGHT);
        assert_eq!(conf.window_resizable, WINDOW_RESIZABLE);

        // Verify icon was created
        assert!(conf.icon.is_some());
    }
}