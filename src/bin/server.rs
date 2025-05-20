

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::sync::Arc;
use rand::Rng;


use bincode;

use netcode_game::types::{PlayerInput, Direction, Position, GameState};

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:9000").await.unwrap();
    println!("Server running on {}", socket.local_addr().unwrap());

    let state: Arc<Mutex<HashMap<SocketAddr, (Position, u32)>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, addr)) => {
                let data = &buf[..size];

                if let Ok(input) = bincode::deserialize::<PlayerInput>(data) {
                    let mut state = state.lock().await;

                    // Hent eller legg til ny spiller
                    let entry = state.entry(addr).or_insert_with(|| {
                        let color = rand::rng().random_range(0x100000..0xFFFFFF); // tilfeldig farge
                        (Position { x: 320, y: 240 }, color)
                    });

                    // Oppdater posisjon
                    match input.dir {
                        Direction::Up => entry.0.y -= 5,
                        Direction::Down => entry.0.y += 5,
                        Direction::Left => entry.0.x -= 5,
                        Direction::Right => entry.0.x += 5,
                    }

                    // Bygg opp snapshot og send til alle
                    let players: Vec<(SocketAddr, Position, u32)> = state.iter()
                        .map(|(addr, (pos, color))| (*addr, *pos, *color))
                        .collect();

                    let snapshot = GameState { players };
                    let payload = bincode::serialize(&snapshot).unwrap();

                    for client_addr in state.keys() {
                        let _ = socket.send_to(&payload, client_addr).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("recv_from error: {:?}", e);
            }
        }
    }
}