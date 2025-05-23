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
use netcode_game::constants::{PREDICTION_ERROR_THRESHOLD, PING_INTERVAL};
use std::time::{Instant};

#[macroquad::main(config_window)]
async fn main() {
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    let initial_position = Position { x: 320, y: 240 };
    let mut prediction = PredictionState::new(initial_position);

    let mut all_players: HashMap<Uuid, (Position, u32)> = HashMap::new();
    let mut interpolated_positions: HashMap<Uuid, InterpolationState> = HashMap::new();
    let mut my_id: Option<Uuid> = None;
    let mut my_pos: Position = initial_position;
    let mut prediction_errors: HashMap<Uuid, f32> = HashMap::new();
    let mut last_ping_time = Instant::now();
    let mut is_connected = true;
    
    // Store previous state for reconnection
    let mut previous_id: Option<Uuid> = None;
    let mut previous_position: Option<Position> = None;
    
    loop {
        // Check if window is being closed
        if is_quit_requested() {
            // Send disconnect message before closing
            if my_id.is_some() {
                net.send_disconnect();
            }
            break;
        }

        let current_time = get_time();
        
        // Handle disconnect/reconnect
        if is_key_pressed(KeyCode::R) {
            if is_connected {
                // Disconnect
                if my_id.is_some() {
                    // Store current state before disconnecting
                    previous_id = my_id;
                    previous_position = Some(my_pos);
                    
                    net.send_disconnect();
                    my_id = None;
                    all_players.clear();
                    interpolated_positions.clear();
                    prediction_errors.clear();
                }
                is_connected = false;
            } else {
                // Reconnect
                if let Some(prev_id) = previous_id {
                    // Send reconnect message with previous ID
                    net.send_reconnect(prev_id, previous_position.unwrap_or(initial_position));
                } else {
                    net.send_connect();
                }
                is_connected = true;
            }
        }
        
        // Send periodic ping if connected
        if is_connected && last_ping_time.elapsed() >= PING_INTERVAL {
            net.send_ping(current_time as u64);
            last_ping_time = Instant::now();
        }
        
        // Handle input and prediction for local player
        if is_connected {
            input_handler.handle_selector_input();
            input_handler.handle_input(&mut my_pos, &mut net, get_frame_time(), &mut prediction);
            net.delay_ms = input_handler.delay_ms;
            net.packet_loss = input_handler.packet_loss;

            // Receive and process game state from server
            if let Some(game_state) = net.try_receive_snapshot() {
                // Update interpolation states for other players
                for (id, pos, _color) in &game_state.players {
                    if Some(*id) != my_id {
                        let interpolation = interpolated_positions.entry(*id).or_insert_with(InterpolationState::new);
                        interpolation.add_position(*pos, current_time as f32, game_state.last_processed.get(id).copied().unwrap_or(0));
                    }
                }

                // Update all players map and check for prediction errors
                for (id, pos, color) in &game_state.players {
                    if Some(*id) == my_id {
                        // Reconcile prediction with server state
                        prediction.reconcile(*pos, game_state.last_processed.get(id).copied().unwrap_or(0), current_time);
                        
                        // Calculate prediction error
                        let error = prediction.get_prediction_error(*pos);
                        prediction_errors.insert(*id, error);
                        
                        // Reapply pending inputs after reconciliation
                        prediction.reapply_pending_inputs(&mut my_pos);
                    }
                    all_players.insert(*id, (*pos, *color));
                }
            }

            // Check for PlayerId message
            if let Some(msg) = net.try_receive_message() {
                if let ClientMessage::PlayerId(id) = msg {
                    my_id = Some(id);
                    println!("Received player ID: {}", id);
                }
            }
        }

        renderer.clear();

        // Draw all players with interpolation
        for (id, (pos, color)) in all_players.iter() {
            if Some(*id) != my_id {
                // Get interpolated position for other players
                if let Some(interpolation) = interpolated_positions.get(id) {
                    if let Some(interpolated_pos) = interpolation.get_interpolated_position(current_time as f32) {
                        renderer.draw_player(
                            interpolated_pos.x as f32,
                            interpolated_pos.y as f32,
                            Color::from_rgba(
                                ((color >> 16) & 0xFF_u32) as u8,
                                ((color >> 8) & 0xFF_u32) as u8,
                                (color & 0xFF_u32) as u8,
                                255,
                            ),
                        );
                    } else {
                        // If we don't have enough positions for interpolation, use the current position
                        renderer.draw_player(
                            pos.x as f32,
                            pos.y as f32,
                            Color::from_rgba(
                                ((color >> 16) & 0xFF_u32) as u8,
                                ((color >> 8) & 0xFF_u32) as u8,
                                (color & 0xFF_u32) as u8,
                                255,
                            )
                        );
                    }
                }
            } else {
                // Draw local player with prediction error visualization
                let error = prediction_errors.get(id).copied().unwrap_or(0.0);
                let error_color = if error > PREDICTION_ERROR_THRESHOLD {
                    Color::from_rgba(255, 0, 0, 128) // Red tint for large errors
                } else {
                    Color::from_rgba(0, 255, 0, 128) // Green tint for small errors
                };

                // Draw prediction error indicator
                if error > 0.0 {
                    draw_circle(
                        my_pos.x as f32,
                        my_pos.y as f32,
                        error * 2.0,
                        error_color,
                    );
                }

                // Draw the player
                renderer.draw_player(
                    my_pos.x as f32,
                    my_pos.y as f32,
                    Color::from_rgba(
                        ((color >> 16) & 0xFF_u32) as u8,
                        ((color >> 8) & 0xFF_u32) as u8,
                        (color & 0xFF_u32) as u8,
                        255,
                    ),
                );
            }
        }

        // Draw network stats
        renderer.draw_tool_bar(input_handler.delay_ms, input_handler.packet_loss, is_connected);

        next_frame().await;
    }
}
