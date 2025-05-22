use macroquad::prelude::*;
use std::collections::HashMap;
use netcode_game::render::Renderer;
use netcode_game::input::InputHandler;
use netcode_game::network::NetworkClient;
use netcode_game::types::{PlayerSnapshot, Position, RemotePlayerState};
use netcode_game::config::config_window;
use std::net::SocketAddr;
use netcode_game::constants::INTERPOLATION_DELAY;

#[macroquad::main(config_window)]
async fn main() {
    let mut all_players: HashMap<SocketAddr, (RemotePlayerState, u32, bool)> = HashMap::new();
    let mut net = NetworkClient::new("127.0.0.1:9000");
    net.send_connect();

    // Initialize helpers
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();

    let mut my_addr: Option<SocketAddr> = None;
    let mut my_pos: Position = Position { x: 320, y: 240 };
    
    loop {
        if is_key_pressed(KeyCode::Escape) {
            net.send_disconnect(); // <- your custom method to inform the server
            break; // exit the loop
        }

        // Handle key input
        let dt = get_frame_time();
        input_handler.handle_input(&mut my_pos, &mut net, dt);
        input_handler.handle_selector_input();
        net.delay_ms = input_handler.delay_ms;
        net.packet_loss = input_handler.packet_loss;


        // Receive snapshot and correct position
        if let Some(snapshot) = net.try_receive_snapshot() {
            let now = get_time(); // Current time in seconds (macroquad)

            for PlayerSnapshot { addr, pos, color, active, last_input_seq } in snapshot.players {
                if my_addr.is_none() {
                    my_addr = Some(addr);
                    net.set_client_addr(addr);
                    my_pos = pos;
                } else if Some(addr) == my_addr {
                    input_handler.reconcile(pos, last_input_seq, &mut my_pos);
                } else {
                    all_players
                        .entry(addr)
                        .and_modify(|(state, _, _)| {
                            state.previous = state.current;
                            state.current = pos;
                            state.last_update_time = now;
                        })
                        .or_insert((
                            RemotePlayerState {
                                current: pos,
                                previous: pos,
                                last_update_time: now,
                            },
                            color,
                            active,
                        ));
                }
            }
        }

        renderer.clear();

        let now = get_time();
        // Draw all players, using predicted position for yourself
        for (addr, (state, color, active)) in &all_players {
            let (draw_x, draw_y) = if Some(*addr) == my_addr {
                (my_pos.x as f32, my_pos.y as f32)
            } else {
                // Interpolate based on how much time has passed since last update
                let elapsed = (now - state.last_update_time) as f32;
                let alpha = (elapsed / INTERPOLATION_DELAY).clamp(0.0, 1.0); // assuming ~20Hz updates (every 50ms)

                let interp_x = state.previous.x as f32 * (1.0 - alpha) + state.current.x as f32 * alpha;
                let interp_y = state.previous.y as f32 * (1.0 - alpha) + state.current.y as f32 * alpha;

                (interp_x, interp_y)
            };

            let mut color = Color::from_rgba(
                ((color >> 16) & 0xFFu32) as u8,
                ((color >> 8) & 0xFFu32) as u8,
                (color & 0xFFu32) as u8,
                255,
            );

            if !*active {
                color = Color::new(color.r * 0.5, color.g * 0.5, color.b * 0.5, 1.0);
            }

            renderer.draw_player(draw_x, draw_y, color);
        }
    }
}
        
