use macroquad::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{Position};
use netcode_game::config::config_window;
use netcode_game::types::{ClientMessage, GameState};

#[macroquad::main(config_window)]
async fn main() {
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();

    let mut all_players: HashMap<Uuid, (Position, u32, bool)> = HashMap::new();
    let mut my_id: Option<Uuid> = None;
    let mut my_pos: Position = Position { x: 320, y: 240 };
    
    loop {
        // Handle key input
        let dt = get_frame_time();
        input_handler.handle_input(&mut my_pos, &mut net, dt);
        input_handler.handle_selector_input();
        net.delay_ms = input_handler.delay_ms;
        net.packet_loss = input_handler.packet_loss;

        let mut buf = [0u8; 2048];
        if let Ok((size, _)) = net.socket.recv_from(&mut buf) {
            if let Ok(ClientMessage::PlayerId(id)) = bincode::deserialize::<ClientMessage>(&buf[..size]) {
                my_id = Some(id);
            } else if let Ok(snapshot) = bincode::deserialize::<GameState>(&buf[..size]) {
                all_players.clear();
                for (id, pos, color, active) in snapshot.players {
                    if Some(id) == my_id {
                        my_pos = pos;
                    }
                    all_players.insert(id, (pos, color, active));
                }
            }
        }

        renderer.clear();

        // Draw all players, using predicted position for yourself
        for (id, (pos, color, active)) in &all_players {
            let (draw_x, draw_y) = if Some(*id) == my_id {
                (my_pos.x as f32, my_pos.y as f32)
            } else {
                (pos.x as f32, pos.y as f32)
            };

            let mut color = Color::from_rgba(
                ((color >> 16) & 0xFFu32) as u8,
                ((color >> 8) & 0xFFu32) as u8,
                (color & 0xFFu32) as u8,
                255,
            );

            // Dim color if inactive
            if !*active {
                color = Color::new(color.r * 0.5, color.g * 0.5, color.b * 0.5, 1.0);
            }

            renderer.draw_player(draw_x, draw_y, color);
        }
        renderer.draw_tool_bar(input_handler.delay_ms, input_handler.packet_loss);

        next_frame().await;
    }
}
