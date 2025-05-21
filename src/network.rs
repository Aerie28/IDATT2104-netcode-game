use crate::types::{ClientMessage, PlayerInput, GameState};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use bincode;
use rand::Rng;

pub struct NetworkClient {
    socket: UdpSocket,
    server_addr: String,
    client_addr: Option<SocketAddr>,
}

impl NetworkClient {
    pub fn new(server_addr: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        socket.set_nonblocking(true).expect("Failed to set non-blocking");
        Self {
            socket,
            server_addr: server_addr.to_string(),
            client_addr: None,
        }
    }
    pub fn send_connect(&self) {
        let msg = ClientMessage::Connect;
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }
    
    pub fn send_disconnect(&self) {
        let msg = ClientMessage::Disconnect;
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }

    pub fn send_input(&self, input: PlayerInput) {
        if Self::simulate_network_conditions() {
            // Drop the packet (simulate loss)
            return;
        }
        let msg = ClientMessage::Input(input);
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }

    pub fn try_receive_snapshot(&self) -> Option<GameState> {
        if Self::simulate_network_conditions() {
            // Drop the packet (simulate loss)
            return None;
        }
        let mut buf = [0u8; 2048];
        if let Ok((size, _)) = self.socket.recv_from(&mut buf) {
            bincode::deserialize(&buf[..size]).ok()
        } else {
            None
        }
    }

    pub fn set_client_addr(&mut self, addr: SocketAddr) {
        self.client_addr = Some(addr);
    }

    pub fn client_addr(&self) -> Option<SocketAddr> {
        self.client_addr
    }

    fn simulate_network_conditions() -> bool {
        // Simulate 100ms ping
        thread::sleep(Duration::from_millis(100));
        // Simulate 10% packet loss
        rand::rng().random_bool(0.1)
    }
}
