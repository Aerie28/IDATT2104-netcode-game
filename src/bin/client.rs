use macroquad::prelude::*;
use uuid::Uuid;
use std::collections::HashMap;
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{Position, ClientMessage};
use netcode_game::config::config_window;
use netcode_game::prediction::PredictionState;
use netcode_game::interpolation::InterpolationState;

#[macroquad::main(config_window)]
async fn main() {
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    let initial_position = Position { x: 320, y: 240 };
    let mut prediction = PredictionState::new(initial_position);

    let mut all_players: HashMap<Uuid, (Position, u32, bool)> = HashMap::new();
    let mut interpolated_positions: HashMap<Uuid, InterpolationState> = HashMap::new();
    let mut my_id: Option<Uuid> = None;
    let mut my_pos: Position = initial_position;
    
    loop {
        // Handle input and prediction for local player
        input_handler.handle_selector_input();
        input_handler.handle_input(&mut my_pos, &mut net, get_frame_time(), &mut prediction);
        net.delay_ms = input_handler.delay_ms;
        net.packet_loss = input_handler.packet_loss;

        // Receive and process game state from server
        if let Some(game_state) = net.try_receive_snapshot() {
            // Update interpolation states for other players
            for (id, pos, _color, _active) in &game_state.players {
                if Some(*id) != my_id {
                    let interpolation = interpolated_positions.entry(*id).or_insert_with(InterpolationState::new);
                    interpolation.add_position(*pos, get_time() as f32);
                }
            }

            // Update all players map without replacing it
            for (id, pos, color, active) in &game_state.players {
                all_players.insert(*id, (*pos, *color, *active));
            }
        }

        // Check for PlayerId message
        if let Some(msg) = net.try_receive_message() {
            if let ClientMessage::PlayerId(id) = msg {
                my_id = Some(id);
                println!("Received player ID: {}", id);
            }
        }

        renderer.clear();

        // Draw all players with interpolation
        for (id, (pos, color, active)) in all_players.iter() {
            if Some(*id) != my_id {
                // Get interpolated position for other players
                if let Some(interpolation) = interpolated_positions.get(id) {
                    if let Some(interpolated_pos) = interpolation.get_interpolated_position(get_time() as f32) {
                        renderer.draw_player(
                            interpolated_pos.x as f32,
                            interpolated_pos.y as f32,
                            Color::from_rgba(
                                ((color >> 16) & 0xFF) as u8,
                                ((color >> 8) & 0xFF) as u8,
                                (color & 0xFF) as u8,
                                255,
                            ),
                        );
                    } else {
                        // If we don't have enough positions for interpolation, use the current position
                        renderer.draw_player(
                            pos.x as f32,
                            pos.y as f32,
                            Color::from_rgba(
                                ((color >> 16) & 0xFF) as u8,
                                ((color >> 8) & 0xFF) as u8,
                                (color & 0xFF) as u8,
                                255,
                            ),
                        );
                    }
                }
            } else {
                // Draw local player without interpolation
                renderer.draw_player(
                    my_pos.x as f32,
                    my_pos.y as f32,
                    Color::from_rgba(
                        ((color >> 16) & 0xFF) as u8,
                        ((color >> 8) & 0xFF) as u8,
                        (color & 0xFF) as u8,
                        255,
                    ),
                );
            }
        }
        renderer.draw_tool_bar(input_handler.delay_ms, input_handler.packet_loss);

        next_frame().await;
    }
}
