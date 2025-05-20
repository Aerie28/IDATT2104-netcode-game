

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::sync::Arc;


use bincode;

use netcode_game::types::{PlayerInput, Direction, Position};


#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:9000").await.unwrap();
    println!("Server running on {}", socket.local_addr().unwrap());

    let positions: Arc<Mutex<HashMap<SocketAddr, Position>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, addr)) => {
                let data = &buf[..size];
                if let Ok(input) = bincode::deserialize::<PlayerInput>(data) {
                    let mut positions = positions.lock().await;
                    let pos = positions.entry(addr).or_insert(Position { x: 320, y: 240 });

                    match input.dir {
                        Direction::Up => pos.y -= 15,
                        Direction::Down => pos.y += 15,
                        Direction::Left => pos.x -= 15,
                        Direction::Right => pos.x += 15,
                    }

                    println!("Received from {}: {:?} => pos: {:?}", addr, input, pos);

                    // Send ny posisjon tilbake til klient
                    let response = bincode::serialize(&pos).unwrap();
                    let _ = socket.send_to(&response, addr).await;
                }
            }
            Err(e) => {
                eprintln!("recv_from error: {:?}", e);
            }
        }
    }
}