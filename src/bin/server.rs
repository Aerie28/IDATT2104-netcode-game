use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time;

use bincode;
use uuid::Uuid;

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

            // Check for collisions using lag compensation
            let current_time = Instant::now().elapsed().as_millis() as u64;
            let collisions = game.check_collisions_at_time(current_time);
            
            // Handle collisions (for now, just print them)
            for (id1, id2) in collisions {
                println!("Collision detected between players {} and {}", id1, id2);
            }

            let snapshot = game.build_snapshot();

            // Add server timestamp to the game state
            let game_state = GameState {
                players: snapshot.players,
                last_processed: snapshot.last_processed,
                server_timestamp: current_time,
            };

            // Get only active players' addresses
            let active_players = game.active_player_addrs();
            let num_active = active_players.len();
            println!("Periodic: Sending snapshot to {} active clients", num_active);

            // Send snapshot only to active players
            broadcast_snapshot_to_selected(&socket_clone, &active_players, &game_state).await;
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
                            let id = game.connect_player(addr);
                            
                            let id_msg = ClientMessage::PlayerId(id);
                            let id_payload = bincode::serialize(&id_msg).unwrap();
                            let _ = socket.send_to(&id_payload, addr).await;
                            
                            // Send initial game state to the new player
                            let snapshot = game.build_snapshot();
                            let game_state = GameState {
                                players: snapshot.players,
                                last_processed: snapshot.last_processed,
                                server_timestamp: Instant::now().elapsed().as_millis() as u64,
                            };
                            let state_payload = bincode::serialize(&game_state).unwrap();
                            let _ = socket.send_to(&state_payload, addr).await;
                            
                            println!("Client {} connected with ID {}", addr, id);
                        }
                        ClientMessage::Input(input) => {
                            game.handle_input(addr, input);
                            game.update_inactive();
                        }
                        ClientMessage::Disconnect => {
                            game.disconnect_player(&addr);
                            println!("Client {} disconnected", addr);
                        }
                        ClientMessage::PlayerId(_) => {
                            // Ignore PlayerId messages from clients
                            println!("Received PlayerId message from client {}", addr);
                        }
                    }
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
