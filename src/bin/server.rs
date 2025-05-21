use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time;

use bincode;

use netcode_game::game::Game;
use netcode_game::types::{ClientMessage, GameState};
use netcode_game::constants::BROADCAST_INTERVAL;


#[tokio::main]
async fn main() {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:9000").await.unwrap());
    println!("Server running on {}", socket.local_addr().unwrap());

    // Use Game struct wrapped in Arc<Mutex> for shared mutable state
    let game = Arc::new(Mutex::new(Game::new()));

    // Clone handles for broadcast task
    let socket_clone = Arc::clone(&socket);
    let game_clone = Arc::clone(&game);

    // Spawn periodic broadcast task
    tokio::spawn(async move {
        let mut interval = time::interval(BROADCAST_INTERVAL);

        loop {
            interval.tick().await;

            let mut game = game_clone.lock().await;
            game.update_inactive();
            let snapshot = game.build_snapshot();

            // Get only active players' addresses
            let active_players = game.active_player_addrs();
            let num_active = active_players.len();
            println!("Periodic: Sending snapshot to {} active clients", num_active);

            // Send snapshot only to active players
            broadcast_snapshot_to_selected(&socket_clone, &active_players, &snapshot).await;
        }
    });

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, addr)) => {
                let data = &buf[..size];
                if let Ok(msg) = bincode::deserialize::<ClientMessage>(data) {
                    let mut game = game.lock().await;

                    match msg {
                        ClientMessage::Connect => {
                            game.connect_player(addr);
                        }
                        ClientMessage::Input(input) => {
                            game.handle_input(addr, input);
                        }
                        ClientMessage::Disconnect => {
                            game.disconnect_player(&addr);
                            println!("Client {} disconnected", addr);
                        }
                    }

                    game.update_inactive();
                    let snapshot = game.build_snapshot();
                    let num_players = snapshot.players.len();
                    println!("Input: Sending snapshot to {} clients", num_players);

                    broadcast_snapshot(&socket, game.players(), &snapshot).await;
                }
            }
            Err(e) => {
                eprintln!("recv_from error: {:?}", e);
            }
        }
    }
}

/// Broadcast snapshot to all active players
async fn broadcast_snapshot(
    socket: &UdpSocket,
    players: &std::collections::HashMap<SocketAddr, netcode_game::game::PlayerState>,
    snapshot: &GameState,
) {
    let payload = bincode::serialize(snapshot).unwrap();

    for (client_addr, player) in players {
        if player.active {
            let _ = socket.send_to(&payload, client_addr).await;
        }
    }
}

async fn broadcast_snapshot_to_selected(
    socket: &UdpSocket,
    active_players: &[SocketAddr],
    snapshot: &GameState,
) {
    let payload = bincode::serialize(snapshot).unwrap();

    for client_addr in active_players {
        let _ = socket.send_to(&payload, client_addr).await;
    }
}
