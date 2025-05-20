use macroquad::prelude::*;
use macroquad::camera::{Camera3D, set_camera};
use macroquad::window::Conf;
use macroquad::miniquad::conf::Icon;
use image::{GenericImageView, imageops::FilterType};

fn window_conf() -> macroquad::window::Conf {
    let icon_bytes = include_bytes!("ntnu-logo.png");
    let image = image::load_from_memory(icon_bytes).unwrap();

    // Resize to required icon sizes
    let small = image.resize_exact(16, 16, FilterType::Lanczos3).into_rgba8().into_raw();
    let medium = image.resize_exact(32, 32, FilterType::Lanczos3).into_rgba8().into_raw();
    let big = image.resize_exact(64, 64, FilterType::Lanczos3).into_rgba8().into_raw();

    let icon = Icon {
        small: small.try_into().unwrap(),
        medium: medium.try_into().unwrap(),
        big: big.try_into().unwrap(),
    };

    macroquad::window::Conf {
        window_title: "My Macroquad App".to_string(),
        window_width: 800,
        window_height: 600,
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initier 3D contekst
    loop {
        clear_background(BLACK);
        set_camera(&Camera3D {
            position: vec3(2., 2., 2.),
            target: vec3(0., 0., 0.),
            up: vec3(0., 1., 0.),
            ..Default::default()
        });

        draw_cube(vec3(0., 0., 0.), vec3(1., 1., 1.), None, RED);
        draw_grid(10, 1.0, GRAY, DARKGRAY);

        next_frame().await;
    }
}
