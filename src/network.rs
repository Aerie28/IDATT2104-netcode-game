use bincode;

use crate::types::{ClientMessage, PlayerInput, GameState};
use crate::constants::{DELAY_MS, PACKET_LOSS};

use rand::Rng;
use rand::seq::SliceRandom;

use std::collections::VecDeque;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

/// Network client that handles sending and receiving messages with simulated network conditions
pub struct NetworkClient {
    pub socket: UdpSocket,
    server_addr: String,
    pub delay_ms: i32,
    pub packet_loss: i32,
    delayed_packets: VecDeque<(Vec<u8>, Instant, u32, i32)>, // (data, send_time, sequence, delay)
}

/// Implementation of the NetworkClient
impl NetworkClient {
    /// Creates a new NetworkClient bound to the specified server address
    pub fn new(server_addr: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        socket.set_nonblocking(true).expect("Failed to set non-blocking");
        Self {
            socket,
            server_addr: server_addr.to_string(),
            delay_ms: DELAY_MS,
            packet_loss: PACKET_LOSS,
            delayed_packets: VecDeque::new(),
        }
    }
    
    /// Connects to the server by sending a connect message
    pub fn send_connect(&self) {
        let msg = ClientMessage::Connect;
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }
    
    /// Sends a ping message with the current timestamp
    pub fn send_ping(&self, timestamp: u64) {
        let msg = ClientMessage::Ping(timestamp);
        let data = bincode::serialize(&msg).unwrap();
        let _ = self.socket.send_to(&data, &self.server_addr);
    }

    /// Sends a player input message with the specified input
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

    /// Tries to receive a game state snapshot from the server
    pub fn try_receive_snapshot(&mut self) -> Option<GameState> {
        self.receive_data()
    }

    /// Tries to receive a client message from the server
    pub fn try_receive_message(&mut self) -> Option<ClientMessage> {
        self.receive_data()
    }
    
    /// Simulates network conditions like packet loss
    fn simulate_network_conditions(&self) -> bool {
        // Simulate packet loss
        rand::rng().random_bool(self.packet_loss as f64 / 100.0)
    }

    /// Processes delayed packets and sends them when their delay has elapsed
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

    /// Receives data from the server for game state or client messages
    fn receive_data<T: serde::de::DeserializeOwned>(&mut self) -> Option<T> {
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
}

/// Test module for NetworkClient
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = NetworkClient::new("127.0.0.1:8080");
        assert_eq!(client.server_addr, "127.0.0.1:8080");
        assert_eq!(client.delay_ms, DELAY_MS);
        assert_eq!(client.packet_loss, PACKET_LOSS);
        assert!(client.delayed_packets.is_empty());
    }

    #[test]
    fn test_simulate_network_conditions() {
        let mut client = NetworkClient::new("127.0.0.1:8080");

        // With 0% loss, should never drop packets
        client.packet_loss = 0;
        for _ in 0..100 {
            assert!(!client.simulate_network_conditions());
        }

        // With 100% loss, should always drop packets
        client.packet_loss = 100;
        for _ in 0..100 {
            assert!(client.simulate_network_conditions());
        }
    }

    #[test]
    fn test_send_connect() {
        // This is mostly a compilation test since we can't easily
        // check the actual message without a mock socket
        let client = NetworkClient::new("127.0.0.1:8080");
        client.send_connect(); // Should not panic
    }

    #[test]
    fn test_send_ping() {
        // Similar to above, just ensuring it compiles and runs
        let client = NetworkClient::new("127.0.0.1:8080");
        client.send_ping(12345); // Should not panic
    }

    #[test]
    fn test_receive_data_with_packet_loss() {
        let mut client = NetworkClient::new("127.0.0.1:8080");
        client.packet_loss = 100; // Always drop packets

        // Since it will always simulate packet loss, this should be None
        let result: Option<GameState> = client.receive_data();
        assert!(result.is_none());
    }

    // For complete socket testing, you'd need more complex setup with
    // mocked UdpSocket, but that's outside the scope of basic unit tests
}
