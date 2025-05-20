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
        window_title: "Netcode Game".to_string(),
        window_width: 800,
        window_height: 600,
        icon: Some(icon),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut all_players: HashMap<SocketAddr, (Position, u32)> = HashMap::new();
    let mut net = NetworkClient::new("127.0.0.1:9000");
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

    let mut my_addr: Option<SocketAddr> = None;
    let mut my_pos: Position = Position { x: 320, y: 240 };

    loop {
        // Input handling and prediction
        for &key in &[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D] {
            let is_down = is_key_down(key);
            let was_down = *prev_keys.get(&key).unwrap_or(&false);

            if is_down && !was_down {
                let dir = match key {
                    KeyCode::W => Direction::Up,
                    KeyCode::A => Direction::Left,
                    KeyCode::S => Direction::Down,
                    KeyCode::D => Direction::Right,
                    _ => continue,
                };
                net.send_input(PlayerInput { dir }); // dir is Copy, so it's fine

                // Predict movement
                match dir {
                    Direction::Up => my_pos.y = my_pos.y.saturating_sub(5),
                    Direction::Down => my_pos.y = my_pos.y.saturating_add(5),
                    Direction::Left => my_pos.x = my_pos.x.saturating_sub(5),
                    Direction::Right => my_pos.x = my_pos.x.saturating_add(5),
                }
            }
            prev_keys.insert(key, is_down);
        }

        // Receive snapshot and correct position
        if let Some(snapshot) = net.try_receive_snapshot() {
            all_players.clear();
            for (addr, pos, color) in snapshot.players {
                if my_addr.is_none() {
                    my_addr = Some(addr);
                    // Set the client address in NetworkClient
                    // You need net to be mutable for this, so declare it as `let mut net = ...` at the top
                    net.set_client_addr(addr);
                    my_pos = pos;
                } else if Some(addr) == my_addr {
                    my_pos = pos;
                }
                all_players.insert(addr, (pos, color));
            }
        }

        clear_background(BLACK);
        set_camera(&camera);

        // Draw all players, using predicted position for yourself
        for (addr, (pos, color)) in &all_players {
            let (draw_x, draw_y) = if Some(*addr) == my_addr {
                (my_pos.x as f32, my_pos.y as f32)
            } else {
                (pos.x as f32, pos.y as f32)
            };

            let scaled_x = (draw_x as f32) * FIELD_WIDTH / SERVER_WIDTH;
            let scaled_y = (draw_y as f32) * FIELD_HEIGHT / SERVER_HEIGHT;

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
