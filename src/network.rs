use crate::types::{ClientMessage, PlayerInput, GameState};
use std::net::{SocketAddr, UdpSocket};
use bincode;

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
        let msg = ClientMessage::Input(input);
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }

    pub fn try_receive_snapshot(&self) -> Option<GameState> {
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
}
