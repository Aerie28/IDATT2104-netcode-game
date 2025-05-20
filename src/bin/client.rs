use std::collections::HashMap;
use minifb::Key;
use netcode_game::network::NetworkClient;
use netcode_game::render::Renderer;
use netcode_game::types::{PlayerInput, Direction, Position};
use std::net::SocketAddr;

fn main() {
    let mut renderer = Renderer::new(640, 480);
    let net = NetworkClient::new("127.0.0.1:9000");

    net.send_connect();

    let mut all_players: HashMap<SocketAddr, (Position, u32, bool)> = HashMap::new();

    // Hold styr på forrige status for hver relevant tast
    let keys = [Key::W, Key::A, Key::S, Key::D];
    let mut prev_keys: HashMap<Key, bool> = keys.iter().map(|&k| (k, false)).collect();

    while renderer.window.is_open() {
        for &key in &keys {
            let is_down = renderer.window.is_key_down(key);
            let was_down = *prev_keys.get(&key).unwrap_or(&false);

            // Sjekk overgang fra ikke-trykket til trykket
            if is_down && !was_down {
                let dir = match key {
                    Key::W => Direction::Up,
                    Key::A => Direction::Left,
                    Key::S => Direction::Down,
                    Key::D => Direction::Right,
                    _ => continue,
                };
                net.send_input(PlayerInput { dir });
            }

            prev_keys.insert(key, is_down);
        }

        // Motta ny snapshot med alle spillere og deres farger
        if let Some(snapshot) = net.try_receive_snapshot() {
            all_players.clear();
            for (addr, pos, color, active) in snapshot.players {
                all_players.insert(addr, (pos, color, active));
            
            }
        }

        // Tegn alle spillere med farge
        renderer.clear();
    for (_addr, (pos, color, active)) in &all_players {
        let draw_color = if *active {
            *color
        } else {
            color & 0x7F7F7F // darken the color (strip high bits)
        };
        renderer.draw_rect(pos.x as usize, pos.y as usize, 10, 10, draw_color);
    }
        renderer.update();
    }
    net.send_disconnect();
}
