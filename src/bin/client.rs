use std::collections::HashMap;
use minifb::Key;
use netcode_game::network::NetworkClient;
use netcode_game::render::Renderer;
use netcode_game::types::{PlayerInput, Direction, Position};
use std::net::SocketAddr;

fn main() {
    let mut renderer = Renderer::new(640, 480);
    let net = NetworkClient::new("127.0.0.1:9000");

    let mut all_players: HashMap<SocketAddr, (Position, u32)> = HashMap::new();

    while renderer.window.is_open() {
        // Input
        if renderer.window.is_key_down(Key::Up) {
            net.send_input(PlayerInput { dir: Direction::Up });
        }
        if renderer.window.is_key_down(Key::Down) {
            net.send_input(PlayerInput { dir: Direction::Down });
        }
        if renderer.window.is_key_down(Key::Left) {
            net.send_input(PlayerInput { dir: Direction::Left });
        }
        if renderer.window.is_key_down(Key::Right) {
            net.send_input(PlayerInput { dir: Direction::Right });
        }

        // Motta ny snapshot
        if let Some(snapshot) = net.try_receive_snapshot() {
            all_players.clear();
            for (addr, pos, color) in snapshot.players {
                all_players.insert(addr, (pos, color));
            }
        }

        // Tegn alle spillere med farge
        renderer.clear();
        for (_addr, (pos, color)) in &all_players {
            renderer.draw_rect(pos.x as usize, pos.y as usize, 10, 10, *color);
        }
        renderer.update();
    }
}
