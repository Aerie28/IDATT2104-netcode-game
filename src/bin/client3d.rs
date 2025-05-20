use macroquad::prelude::*;
use macroquad::camera::{Camera3D, set_camera};
use macroquad::window::Conf;
use macroquad::miniquad::conf::Icon;
use image::imageops::FilterType;
use std::collections::HashMap;
use netcode_game::network::NetworkClient;
use netcode_game::types::{PlayerInput, Direction, Position};
use std::net::SocketAddr;

const FIELD_WIDTH: f32 = 64.0;
const FIELD_HEIGHT: f32 = 48.0;

// These represent the server's coordinate system dimensions
const SERVER_WIDTH: f32 = 640.0;
const SERVER_HEIGHT: f32 = 480.0;

fn window_conf() -> Conf {
    let icon_bytes = include_bytes!("ntnu-logo.png");
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
        window_title: "Netcoding Game".to_string(),
        window_width: 800,
        window_height: 600,
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut all_players: HashMap<SocketAddr, (Position, u32)> = HashMap::new();
    let net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();

    let mut prev_keys = HashMap::from([
        (KeyCode::W, false),
        (KeyCode::A, false),
        (KeyCode::S, false),
        (KeyCode::D, false),
    ]);

    // Camera centered above the scaled playfield, looking down
    let camera = Camera3D {
        position: vec3(FIELD_WIDTH / 2.0, 40.0, FIELD_HEIGHT / 2.0 + 0.1),
        target: vec3(FIELD_WIDTH / 2.0, 0.0, FIELD_HEIGHT / 2.0),
        up: vec3(0., 1., 0.),
        ..Default::default()
    };

    loop {
        for &key in &[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D] {
            let is_down = is_key_down(key);
            let was_down = *prev_keys.get(&key).unwrap_or(&false);

            if is_down && !was_down {
                let dir = match key {
                    KeyCode::W => Direction::Up,
                    KeyCode::S => Direction::Down,
                    KeyCode::A => Direction::Left,
                    KeyCode::D => Direction::Right,
                    _ => continue,
                };
                net.send_input(PlayerInput { dir });
            }
            prev_keys.insert(key, is_down);
        }

        if let Some(snapshot) = net.try_receive_snapshot() {
            all_players.clear();
            for (addr, pos, color) in snapshot.players {
                all_players.insert(addr, (pos, color));
            }
        }

        clear_background(BLACK);
        set_camera(&camera);

        // Draw ground grid with your scaled FIELD_WIDTH and FIELD_HEIGHT
        draw_grid(FIELD_WIDTH as u32, 1.0, GRAY, DARKGRAY);

        // Draw all players as cubes with position scaled from server coords to FIELD coords
        for (_addr, (pos, color)) in &all_players {
            let scaled_x = (pos.x as f32) * FIELD_WIDTH / SERVER_WIDTH;
            let scaled_y = (pos.y as f32) * FIELD_HEIGHT / SERVER_HEIGHT;

            let color = Color::from_rgba(
                ((*color >> 16) & 0xFF) as u8,
                ((*color >> 8) & 0xFF) as u8,
                (*color & 0xFF) as u8,
                255,
            );
            draw_cube(
                vec3(scaled_x, 0.5, scaled_y),
                vec3(1., 1., 1.),
                None,
                color,
            );
        }

        next_frame().await;
    }
}
