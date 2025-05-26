use bincode;

use netcode_game::constants::BROADCAST_INTERVAL;
use netcode_game::game::Game;
use netcode_game::types::{ClientMessage, GameState};

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time;

/// Server main function using Tokio for async I/O
#[tokio::main]
async fn main() {
    // Bind the UDP socket to the specified address and start the server
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
            game.update_server_dropped();
            
            let current_time = Instant::now().elapsed().as_millis() as u64;

            let snapshot = game.build_snapshot();

            // Add server timestamp to the game state
            let game_state = GameState {
                players: snapshot.players,
                last_processed: snapshot.last_processed,
                server_timestamp: current_time,
            };

            // Get only active players' addresses
            let active_players = game.active_player_addrs();

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
                            
                            println!("Player {} connected from {}", id, addr);
                        }
                        ClientMessage::Input(input) => {
                            game.handle_input(addr, input);
                            game.update_server_dropped();
                        }
                        ClientMessage::Ping(timestamp) => {
                            // Echo back the timestamp as a pong
                            let pong_msg = ClientMessage::Pong(timestamp);
                            let pong_payload = bincode::serialize(&pong_msg).unwrap();
                            let _ = socket.send_to(&pong_payload, addr).await;
                            
                            // Update player's last active time
                            if let Some(player) = game.get_players_mut().get_mut(&addr) {
                                player.last_active = Instant::now();
                            }
                        }
                        ClientMessage::Pong(_) => {
                            // Ignore pong messages from clients
                        }
                        ClientMessage::PlayerId(_) => {
                            // Ignore PlayerId messages from clients
                        }
                    }
                }
            }
            Err(_e) => {
                // Handle errors (e.g., log them)
            }
        }
    }
}

/// Broadcasts the game state snapshot to all active players
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

/// Tests for the server functionality
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;
    use netcode_game::types::Position;

    #[tokio::test]
    async fn test_broadcast_snapshot_to_selected() {
        // Create a mock socket using a real UDP socket bound to a temporary port
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = socket.local_addr().unwrap();

        // Create client sockets to receive the broadcast
        let client1 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client1_addr = client1.local_addr().unwrap();
        let client2 = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client2_addr = client2.local_addr().unwrap();

        // Connect clients to the server (for UDP this just sets the default destination)
        client1.connect(server_addr).await.unwrap();
        client2.connect(server_addr).await.unwrap();

        // Create a test game state
        let player_id1 = Uuid::new_v4();
        let player_id2 = Uuid::new_v4();

        // Create player data for the game state
        let mut players = Vec::new();
        let mut last_processed = std::collections::HashMap::new();

        // Add players to the vector (using the expected (Uuid, Position, u32) format)
        players.push((player_id1, Position { x: 100, y: 100 }, 0));
        players.push((player_id2, Position { x: 200, y: 200 }, 0));

        last_processed.insert(player_id1, 5);
        last_processed.insert(player_id2, 10);

        let game_state = GameState {
            players,
            last_processed,
            server_timestamp: 123456,
        };

        // Broadcast to the client addresses
        broadcast_snapshot_to_selected(&socket, &[client1_addr, client2_addr], &game_state).await;

        // Now check that both clients received the broadcast
        let mut buf = [0u8; 1024];

        // Set a timeout for receiving
        tokio::select! {
            res = client1.recv(&mut buf) => {
                let size = res.unwrap();
                let received: GameState = bincode::deserialize(&buf[..size]).unwrap();
                assert_eq!(received.server_timestamp, 123456);
                assert_eq!(received.players.len(), 2);
            }
            _ = sleep(Duration::from_millis(100)) => {
                panic!("Timeout waiting for broadcast to client 1");
            }
        }

        tokio::select! {
            res = client2.recv(&mut buf) => {
                let size = res.unwrap();
                let received: GameState = bincode::deserialize(&buf[..size]).unwrap();
                assert_eq!(received.server_timestamp, 123456);
                assert_eq!(received.players.len(), 2);
            }
            _ = sleep(Duration::from_millis(100)) => {
                panic!("Timeout waiting for broadcast to client 2");
            }
        }
    }

    // The second test can be kept as is
    #[tokio::test]
    async fn test_server_connect_handler() {
        // Start a server on a random port
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let game = Arc::new(Mutex::new(Game::new()));
        let socket_arc = Arc::new(server_socket);

        // Spawn the server handler task
        let socket_clone = Arc::clone(&socket_arc);
        let game_clone = Arc::clone(&game);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            // Just handle one message for the test
            if let Ok((size, addr)) = socket_clone.recv_from(&mut buf).await {
                let data = &buf[..size];
                if let Ok(msg) = bincode::deserialize::<ClientMessage>(data) {
                    let mut game = game_clone.lock().await;

                    if let ClientMessage::Connect = msg {
                        let id = game.connect_player(addr);

                        let id_msg = ClientMessage::PlayerId(id);
                        let id_payload = bincode::serialize(&id_msg).unwrap();
                        let _ = socket_clone.send_to(&id_payload, addr).await;

                        let snapshot = game.build_snapshot();
                        let game_state = GameState {
                            players: snapshot.players,
                            last_processed: snapshot.last_processed,
                            server_timestamp: Instant::now().elapsed().as_millis() as u64,
                        };
                        let state_payload = bincode::serialize(&game_state).unwrap();
                        let _ = socket_clone.send_to(&state_payload, addr).await;
                    }
                }
            }
        });

        // Create a client and connect to the server
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.connect(server_addr).await.unwrap();

        // Send a Connect message
        let connect_msg = ClientMessage::Connect;
        let connect_payload = bincode::serialize(&connect_msg).unwrap();
        client.send(&connect_payload).await.unwrap();

        // Wait for the response - should be a PlayerID message
        let mut buf = [0u8; 1024];

        tokio::select! {
            res = client.recv(&mut buf) => {
                let size = res.unwrap();
                if let Ok(ClientMessage::PlayerId(id)) = bincode::deserialize(&buf[..size]) {
                    assert!(!id.to_string().is_empty());
                } else {
                    panic!("Expected PlayerId message");
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                panic!("Timeout waiting for PlayerID response");
            }
        }

        // Wait for the GameState message
        tokio::select! {
            res = client.recv(&mut buf) => {
                let size = res.unwrap();
                let game_state: GameState = bincode::deserialize(&buf[..size]).unwrap();
                assert_eq!(game_state.players.len(), 1);
            }
            _ = sleep(Duration::from_millis(100)) => {
                panic!("Timeout waiting for GameState");
            }
        }
    }
}