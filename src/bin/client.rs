use std::collections::HashMap;
use minifb::Key;
use netcode_game::network::NetworkClient;
use netcode_game::render::Renderer;
use netcode_game::types::{PlayerInput, Direction, Position};


fn main() {
    let mut renderer = Renderer::new(640, 480);
    let net = NetworkClient::new("127.0.0.1:9000");
    let mut player_pos = Position { x: 320, y: 240 };

    let keys = [Key::Up, Key::Down, Key::Left, Key::Right];
    let mut prev_keys: HashMap<Key, bool> = keys.iter().map(|&k| (k, false)).collect();

    while renderer.window.is_open() {
        for &key in &keys {
            let is_down = renderer.window.is_key_down(key);
            let was_down = *prev_keys.get(&key).unwrap_or(&false);

            if is_down && !was_down {
                let dir = match key {
                    Key::Up => Direction::Up,
                    Key::Down => Direction::Down,
                    Key::Left => Direction::Left,
                    Key::Right => Direction::Right,
                    _ => continue,
                };
                net.send_input(PlayerInput { dir });
            }
            prev_keys.insert(key, is_down);
        }

        if let Some(pos) = net.try_receive_position() {
            player_pos = pos;
        }

        renderer.clear();
        renderer.draw_rect(player_pos.x as usize, player_pos.y as usize, 10, 10, 0x00FF00);
        renderer.update();
    }
}