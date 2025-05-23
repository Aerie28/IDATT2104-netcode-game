use macroquad::prelude::*;
use miniquad::*;
use uuid::Uuid;
use std::collections::HashMap;
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{Position, ClientMessage};
use netcode_game::config::config_window;
use netcode_game::prediction::PredictionState;
use netcode_game::interpolation::InterpolationState;
use netcode_game::analysis::PerformanceAnalyzer;
use netcode_game::constants::{PREDICTION_ERROR_THRESHOLD, PING_INTERVAL, PERFORMANCE_TEST_FREQUENCY};
use std::time::{Instant};

#[macroquad::main(config_window)]
async fn main() {
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    let mut performance_analyzer = PerformanceAnalyzer::new(PERFORMANCE_TEST_FREQUENCY);
    let initial_position = Position { x: 320, y: 240 };
    let mut prediction = PredictionState::new(initial_position);

    let mut all_players: HashMap<Uuid, (Position, u32)> = HashMap::new();
    let mut interpolated_positions: HashMap<Uuid, InterpolationState> = HashMap::new();
    let mut my_id: Option<Uuid> = None;
    let mut my_pos: Position = initial_position;
    let mut prediction_errors: HashMap<Uuid, f32> = HashMap::new();
    let mut last_ping_time = Instant::now();
    let mut is_connected = true;

    let original_delay = input_handler.delay_ms;
    let original_loss = input_handler.packet_loss;
    let mut is_testing = false;
    
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
                    
                    // Send disconnect message and wait a bit to ensure it's received
                    net.send_disconnect();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
                    my_id = None;
                    all_players.clear();
                    interpolated_positions.clear();
                    prediction_errors.clear();
                    prediction.reset(); // Reset prediction state
                }
                is_connected = false;
            } else {
                // Reconnect
                if let Some(prev_id) = previous_id {
                    // Send reconnect message with previous ID and position
                    let reconnect_pos = previous_position.unwrap_or(initial_position);
                    net.send_reconnect(prev_id, reconnect_pos);
                    // Reset local position to match server
                    my_pos = reconnect_pos;
                    prediction.reset_with_position(reconnect_pos); // Reset prediction with server position
                } else {
                    net.send_connect();
                }
                is_connected = true;
            }
        }
        
        // Send periodic ping if connected
        if is_connected && last_ping_time.elapsed() >= PING_INTERVAL {
            let current_time = get_time();
            net.send_ping((current_time * 1000.0) as u64); // Convert to milliseconds
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
                let current_time = get_time();
                let server_time = game_state.server_timestamp as f64 / 1000.0; // Convert from milliseconds to seconds
                let time_diff = current_time - server_time;
                
                // Create a set of current player IDs from the server
                let current_player_ids: std::collections::HashSet<Uuid> = game_state.players.iter()
                    .map(|(id, _, _)| *id)
                    .collect();

                // Remove players that are no longer in the game state
                all_players.retain(|id, _| current_player_ids.contains(id));
                interpolated_positions.retain(|id, _| current_player_ids.contains(id));
                prediction_errors.retain(|id, _| current_player_ids.contains(id));

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
                match msg {
                    ClientMessage::PlayerId(id) => {
                        // Only update ID if we don't already have one
                        if my_id.is_none() {
                            my_id = Some(id);
                            println!("Received player ID: {}", id);
                        }
                    }
                    _ => {
                        // Ignore other messages
                    }
                }
            }
        }

        // Test performance analysis
        if is_key_pressed(KeyCode::T) {
            if is_testing {
            } else {
                // Reset analyzer before starting new tests
                performance_analyzer.reset();
                is_testing = start_next_test(&mut performance_analyzer, &mut input_handler);
            }
        }
        if is_testing && performance_analyzer.is_test_complete() {
            performance_analyzer.complete_current_test();
            is_testing = start_next_test(&mut performance_analyzer, &mut input_handler);

            if !is_testing {
                // Testing complete, restore original settings
                input_handler.delay_ms = original_delay;
                input_handler.packet_loss = original_loss;
                println!("{}", performance_analyzer.generate_report());
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
        renderer.draw_tool_bar(input_handler.delay_ms, input_handler.packet_loss, is_connected, is_testing);

        next_frame().await;
    }
}
fn start_next_test(
    performance_analyzer: &mut PerformanceAnalyzer,
    input_handler: &mut InputHandler,
) -> bool {
    if let Some(condition) = performance_analyzer.start_next_test() {
        input_handler.delay_ms = condition.latency_ms;
        input_handler.packet_loss = condition.packet_loss_percent;
        println!("Testing condition: {}", condition.name);
        true
    } else {
        false
    }
}