use crate::types::{ClientMessage, PlayerInput, GameState};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use bincode;
use rand::Rng;
use rand::seq::SliceRandom;
use crate::constants::{DELAY_MS, PACKET_LOSS};
use std::collections::VecDeque;
use uuid::Uuid;
use crate::types::Position;

pub struct NetworkClient {
    pub socket: UdpSocket,
    server_addr: String,
    client_addr: Option<SocketAddr>,
    pub delay_ms: i32,
    pub packet_loss: i32,
    delayed_packets: VecDeque<(Vec<u8>, Instant, u32, i32)>, // (data, send_time, sequence, delay)
}

impl NetworkClient {
    pub fn new(server_addr: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        socket.set_nonblocking(true).expect("Failed to set non-blocking");
        Self {
            socket,
            server_addr: server_addr.to_string(),
            client_addr: None,
            delay_ms: DELAY_MS,
            packet_loss: PACKET_LOSS,
            delayed_packets: VecDeque::new(),
        }
    }
    pub fn send_connect(&self) {
        let msg = ClientMessage::Connect;
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }
    
    

    pub fn send_ping(&self, timestamp: u64) {
        let msg = ClientMessage::Ping(timestamp);
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }

    pub fn send_input(&mut self, input: PlayerInput) {
        if self.simulate_network_conditions() {
            // Drop the packet (simulate loss)
            return;
        }
        let msg = ClientMessage::Input(input);
        let data = bincode::serialize(&msg).unwrap();
        
        // Add artificial delay with jitter
        if self.delay_ms > 0 {
            let jitter = rand::rng().random_range(-5..=5); // ±5ms jitter
            let delay = (self.delay_ms + jitter).max(0);
            self.delayed_packets.push_back((data, Instant::now(), input.sequence, delay));
        } else {
            let _ = self.socket.send_to(&data, &self.server_addr);
        }
    }

    pub fn try_receive_snapshot(&mut self) -> Option<GameState> {
        // Process delayed packets
        self.process_delayed_packets();

        if self.simulate_network_conditions() {
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

    pub fn try_receive_message(&mut self) -> Option<ClientMessage> {
        // Process delayed packets
        self.process_delayed_packets();

        if self.simulate_network_conditions() {
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
    

    fn simulate_network_conditions(&self) -> bool {
        // Simulate packet loss
        rand::rng().random_bool(self.packet_loss as f64 / 100.0)
    }

    fn process_delayed_packets(&mut self) {
        let now = Instant::now();
        let mut ready_packets: Vec<(Vec<u8>, u32)> = Vec::new();

        // Collect all packets that are ready to be sent
        while let Some((data, send_time, sequence, delay)) = self.delayed_packets.front() {
            if now.duration_since(*send_time) >= Duration::from_millis(*delay as u64) {
                ready_packets.push((data.clone(), *sequence));
                self.delayed_packets.pop_front();
            } else {
                break;
            }
        }

        // Shuffle ready packets to simulate out-of-order delivery
        if !ready_packets.is_empty() {
            let mut rng = rand::rng();
            ready_packets.shuffle(&mut rng);

            // Send packets in shuffled order
            for (data, _) in ready_packets {
                let _ = self.socket.send_to(&data, &self.server_addr);
            }
        }
    }
}
