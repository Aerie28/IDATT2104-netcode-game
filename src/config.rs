use image::imageops::FilterType;
use miniquad::conf::{Conf, Icon};

pub fn config_window() -> Conf {
    let icon_bytes = include_bytes!("assets/icon.png");
    let image = image::load_from_memory(icon_bytes).unwrap();

    let small = image.resize_exact(16, 16, FilterType::Lanczos3).into_rgba8().into_raw();
    let medium = image.resize_exact(32, 32, FilterType::Lanczos3).into_rgba8().into_raw();
    let big = image.resize_exact(64, 64, FilterType::Lanczos3).into_rgba8().into_raw();

    let icon = Icon {
        small: small.try_into().unwrap(),
        medium: medium.try_into().unwrap(),
        big: big.try_into().unwrap(),
    };

    Conf {
        window_title: "Netcode Game".to_string(),
        window_width: 800,
        window_height: 600,
        icon: Some(icon),
        ..Default::default()
    }
}