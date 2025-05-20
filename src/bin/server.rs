use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::sync::Arc;
use rand::Rng;
use tokio::time;

use bincode;
use netcode_game::types::{ClientMessage, PlayerInput, Direction, Position, GameState};

const TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:9000").await.unwrap());
    println!("Server running on {}", socket.local_addr().unwrap());

    let state: Arc<Mutex<HashMap<SocketAddr, (Position, u32, Instant, bool)>>> = Arc::new(Mutex::new(HashMap::new()));
    let state_clone = Arc::clone(&state);
    let socket_clone = Arc::clone(&socket);

    // Periodic broadcast task
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            let mut state = state_clone.lock().await;
            update_inactive(&mut state);
            let snapshot = build_snapshot(&state);
            let num_players = snapshot.players.len();
            println!("Periodic: Sending snapshot to {} clients", num_players);
            broadcast_snapshot(&socket_clone, &state, &snapshot).await;
        }
    });

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, addr)) => {
                let data = &buf[..size];
                if let Ok(msg) = bincode::deserialize::<ClientMessage>(data) {
                    let mut state = state.lock().await;

                    match msg {
                        ClientMessage::Connect => {
                            let mut rng = rand::rng();
                            let x = rng.random_range(0..640); // assuming 640x480 game area
                            let y = rng.random_range(0..480);
                            state.entry(addr).or_insert_with(|| {
                                let color = rand::rng().random_range(0x100000..0xFFFFFF);
                                (Position { x, y }, color, Instant::now(), true)
                            });
                        }
                        ClientMessage::Input(input) => {
                            handle_input(&mut state, addr, input);
                        }
                        ClientMessage::Disconnect => {
                            // Mark player as inactive or remove from state
                            state.remove(&addr);
                            println!("Client {} disconnected", addr);
                        }
                    }

                    update_inactive(&mut state);
                    let snapshot = build_snapshot(&state);
                    let num_players = snapshot.players.len();
                    println!("Input: Sending snapshot to {} clients", num_players);
                    broadcast_snapshot(&socket, &state, &snapshot).await;
                }
            }
            Err(e) => {
                eprintln!("recv_from error: {:?}", e);
            }
        }
    }
}
fn update_inactive(state: &mut HashMap<SocketAddr, (Position, u32, Instant, bool)>) {
    for (_addr, (_pos, _color, last, active)) in state.iter_mut() {
        if last.elapsed() >= TIMEOUT {
            *active = false;
        }
    }
}
fn build_snapshot(state: &HashMap<SocketAddr, (Position, u32, Instant, bool)>) -> GameState {
    let players = state.iter()
        .filter(|(_addr, (_pos, _color, _last, active))| *active)
        .map(|(addr, (pos, color, _last, _active))| (*addr, *pos, *color))
        .collect();
    GameState { players }
}

fn handle_input(
    state: &mut HashMap<SocketAddr, (Position, u32, Instant, bool)>,
    addr: SocketAddr,
    input: PlayerInput,
) {
    let entry = state.entry(addr).or_insert_with(|| {
        let color = rand::rng().random_range(0x100000..0xFFFFFF);
        (Position { x: 320, y: 240 }, color, Instant::now(), true)
    });
    entry.2 = Instant::now();
    entry.3 = true;

    match input.dir {
        Direction::Up => entry.0.y -= 5,
        Direction::Down => entry.0.y += 5,
        Direction::Left => entry.0.x -= 5,
        Direction::Right => entry.0.x += 5,
    }
}

async fn broadcast_snapshot(
    socket: &UdpSocket,
    state: &HashMap<SocketAddr, (Position, u32, Instant, bool)>,
    snapshot: &GameState,
) {
    let payload = bincode::serialize(snapshot).unwrap();
    for client_addr in state.keys() {
        let _ = socket.send_to(&payload, client_addr).await;
    }
}