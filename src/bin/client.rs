use macroquad::prelude::*;
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{Direction, PlayerInput, Position, PredictionState };
use netcode_game::config::config_window;
use netcode_game::types::{ClientMessage, GameState};

#[macroquad::main(config_window)]
async fn main() {
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();
    
    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    let mut prediction = PredictionState {
        next_sequence: 0,
        pending_inputs: VecDeque::new(),
        last_server_pos: Position { x: 320, y: 240 },
    };

    let mut all_players: HashMap<Uuid, (Position, u32, bool)> = HashMap::new();
    let mut my_id: Option<Uuid> = None;
    let mut my_pos: Position = Position { x: 320, y: 240 };
    
    loop {
        // Handle key input
        let dt = get_frame_time();
        input_handler.handle_input(&mut my_pos, &mut net, dt, &mut prediction);
        input_handler.handle_selector_input();
        net.delay_ms = input_handler.delay_ms;
        net.packet_loss = input_handler.packet_loss;

        let mut buf = [0u8; 2048];
        if let Ok((size, _)) = net.socket.recv_from(&mut buf) {
            if let Ok(snapshot) = bincode::deserialize::<GameState>(&buf[..size]) {
                all_players.clear();
                for (id, pos, color, active) in snapshot.players {
                    if Some(id) == my_id {
                        // Store the authoritative position
                        prediction.last_server_pos = pos;
                        my_pos = pos; // Start with server position

                        // Get last processed input for this player
                        if let Some(&last_processed) = snapshot.last_processed.get(&id) {
                            // Remove acknowledged inputs
                            while let Some((seq, _)) = prediction.pending_inputs.front() {
                                if *seq <= last_processed {
                                    prediction.pending_inputs.pop_front();
                                } else {
                                    break;
                                }
                            }
                        }

                        // Reapply pending inputs
                        for (_, input) in &prediction.pending_inputs {
                            apply_input_to_position(&mut my_pos, *input, dt);
                        }
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

fn apply_input_to_position(pos: &mut Position, input: PlayerInput, _dt: f32) {
    use netcode_game::constants::PLAYER_SPEED;
    match input.dir {
        Direction::Up => pos.y = pos.y.saturating_sub(PLAYER_SPEED),
        Direction::Down => pos.y = pos.y.saturating_add(PLAYER_SPEED),
        Direction::Left => pos.x = pos.x.saturating_sub(PLAYER_SPEED),
        Direction::Right => pos.x = pos.x.saturating_add(PLAYER_SPEED),
    }
}
