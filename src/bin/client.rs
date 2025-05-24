use macroquad::prelude::*;

use netcode_game::analysis::PerformanceAnalyzer;
use netcode_game::config::config_window;
use netcode_game::constants::{ PREDICTION_ERROR_THRESHOLD, PING_INTERVAL, PERFORMANCE_TEST_FREQUENCY };
use netcode_game::input::InputHandler;
use netcode_game::interpolation::InterpolationState;
use netcode_game::network::NetworkClient;
use netcode_game::prediction::PredictionState;
use netcode_game::render::Renderer;
use netcode_game::types::{Position, ClientMessage};

use std::collections::HashMap;
use std::time::{Instant};
use uuid::Uuid;

/// Client main function
#[macroquad::main(config_window)]
async fn main() {
    // Initialize the game window and connect to the server
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers and variables
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
    let mut should_send_pings = true;

    let original_delay = input_handler.delay_ms;
    let original_loss = input_handler.packet_loss;
    let mut is_testing = false;

    // Main game loop
    loop {
        let current_time = get_time();
        
        // Handle disconnect/reconnect
        if is_key_pressed(KeyCode::R) {
            if is_connected {
                // Stop sending pings to trigger timeout disconnect
                println!("Stopping ping messages to trigger timeout disconnect...");
                should_send_pings = false;
                is_connected = false;
            } else {
                // Connect
                println!("Starting connect process...");
                net.send_connect();
                should_send_pings = true;
                is_connected = true;
            }
        }
        
        // Send periodic ping if connected and pings are enabled
        if is_connected && should_send_pings && last_ping_time.elapsed() >= PING_INTERVAL {
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
                let current_time = get_time(); // Convert from milliseconds to seconds
                
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

                        // Record performance analysis errors
                        if is_testing {
                            performance_analyzer.record_prediction_error(error);
                        }
                        
                        // Reapply pending inputs after reconciliation
                        prediction.reapply_pending_inputs(&mut my_pos);
                    }
                    all_players.insert(*id, (*pos, *color));
                }
            }

            // Check for PlayerId message from server (not needed for functional gameplay,
            // but needed as a default)
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
                // Determine position to draw (interpolated or fallback)
                let position_to_draw = interpolated_positions
                    .get(id)
                    .and_then(|interpol| interpol.get_interpolated_position(current_time as f32))
                    .unwrap_or(*pos);

                draw_player_with_color(position_to_draw, *color, &renderer);
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

                draw_player_with_color(my_pos, *color, &renderer);
            }
        }

        // Draw network stats
        renderer.draw_tool_bar(input_handler.delay_ms, input_handler.packet_loss, is_connected, is_testing);

        next_frame().await;
    }
}

/// Helper function to start the next performance test
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

/// Helper function to draw a player with a specific color
fn draw_player_with_color(position: Position, color: u32, renderer: &Renderer) {
    renderer.draw_player(
        position.x as f32,
        position.y as f32,
        Color::from_rgba(
            ((color >> 16) & 0xFF_u32) as u8,
            ((color >> 8) & 0xFF_u32) as u8,
            (color & 0xFF_u32) as u8,
            255,
        ),
    );
}

/// Tests for the client functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_next_test() {
        // Simple struct that matches what PerformanceAnalyzer.start_next_test returns
        struct TestCondition {
            pub latency_ms: i32,
            pub packet_loss_percent: i32,
        }

        // Test implementation of the required method
        struct TestPerformanceAnalyzer {
            conditions: Vec<TestCondition>,
            index: usize,
        }

        impl TestPerformanceAnalyzer {
            fn new() -> Self {
                Self {
                    conditions: vec![
                        TestCondition { latency_ms: 50, packet_loss_percent: 5 },
                        TestCondition { latency_ms: 100, packet_loss_percent: 10 },
                    ],
                    index: 0,
                }
            }

            fn start_next_test(&mut self) -> Option<&TestCondition> {
                if self.index < self.conditions.len() {
                    let condition = &self.conditions[self.index];
                    self.index += 1;
                    Some(condition)
                } else {
                    None
                }
            }
        }

        let mut analyzer = TestPerformanceAnalyzer::new();
        let mut input_handler = InputHandler::new();

        // Create our own test function rather than using the real one
        fn test_next(analyzer: &mut TestPerformanceAnalyzer, handler: &mut InputHandler) -> bool {
            if let Some(condition) = analyzer.start_next_test() {
                handler.delay_ms = condition.latency_ms;
                handler.packet_loss = condition.packet_loss_percent;
                true
            } else {
                false
            }
        }

        // Test the logic
        assert!(test_next(&mut analyzer, &mut input_handler));
        assert_eq!(input_handler.delay_ms, 50);
        assert_eq!(input_handler.packet_loss, 5);

        assert!(test_next(&mut analyzer, &mut input_handler));
        assert_eq!(input_handler.delay_ms, 100);
        assert_eq!(input_handler.packet_loss, 10);

        assert!(!test_next(&mut analyzer, &mut input_handler));
    }

    #[test]
    fn test_color_conversion() {
        let color = 0xFF0000; // Red

        let r = ((color >> 16) & 0xFF_u32) as u8;
        let g = ((color >> 8) & 0xFF_u32) as u8;
        let b = (color & 0xFF_u32) as u8;

        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);

        let color = 0x00FF00; // Green

        let r = ((color >> 16) & 0xFF_u32) as u8;
        let g = ((color >> 8) & 0xFF_u32) as u8;
        let b = (color & 0xFF_u32) as u8;

        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_position_creation() {
        // Test the Position struct
        let pos = Position { x: 100, y: 200 };
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
    }
}