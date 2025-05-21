use macroquad::prelude::*;
use macroquad::camera::{Camera3D};
use std::collections::HashMap;
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{Position};
use netcode_game::config::config_window;
use std::net::SocketAddr;

const FIELD_WIDTH: f32 = 64.0;
const FIELD_HEIGHT: f32 = 48.0;

// These represent the server's coordinate system dimensions
const SERVER_WIDTH: f32 = 640.0;
const SERVER_HEIGHT: f32 = 480.0;

#[macroquad::main(config_window)]
async fn main() {
    let mut all_players: HashMap<SocketAddr, (Position, u32, bool)> = HashMap::new();
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    

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
        // Handle key input
        let dt = get_frame_time();
        input_handler.handle_input(&mut my_pos, &mut net, dt);
        

        // Receive snapshot and correct position
        if let Some(snapshot) = net.try_receive_snapshot() {
            all_players.clear();
            for (addr, pos, color, active) in snapshot.players {
                if my_addr.is_none() {
                    my_addr = Some(addr);
                    net.set_client_addr(addr);
                    my_pos = pos;
                } else if Some(addr) == my_addr {
                    my_pos = pos;
                }
                all_players.insert(addr, (pos, color, active));
            }
        }

        renderer.clear();
        renderer.set_camera(&camera);

        // Draw all players, using predicted position for yourself
        for (addr, (pos, color, active)) in &all_players {
            let (draw_x, draw_y) = if Some(*addr) == my_addr {
                (my_pos.x as f32, my_pos.y as f32)
            } else {
                (pos.x as f32, pos.y as f32)
            };

            let scaled_x = (draw_x) * FIELD_WIDTH / SERVER_WIDTH;
            let scaled_y = (draw_y) * FIELD_HEIGHT / SERVER_HEIGHT;

            let draw_color = if *active {
                *color
            } else {
                color & 0x7F7F7F
            };

            let color = Color::from_rgba(
                ((draw_color >> 16) & 0xFF) as u8,
                ((draw_color >> 8) & 0xFF) as u8,
                (draw_color & 0xFF) as u8,
                255,
            );

            renderer.draw_player(scaled_x, scaled_y, color);
        }
        next_frame().await;
    }
}
