use crate::types::{ClientMessage, PlayerInput, GameState};
use std::net::{ UdpSocket};
use bincode;

pub struct NetworkClient {
    socket: UdpSocket,
    server_addr: String,
}

impl NetworkClient {
    pub fn new(server_addr: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        socket.set_nonblocking(true).expect("Failed to set non-blocking");
        Self {
            socket,
            server_addr: server_addr.to_string(),
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
}
