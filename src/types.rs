use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents messages sent from the server to the client
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
    PlayerId(Uuid),
    Input(PlayerInput),
    Ping(u64),  // Client sends timestamp
    Pong(u64),  // Server echoes timestamp
}

/// Represents a network condition for simulating latency and packet loss
#[derive(Clone)]
pub struct NetworkCondition {
    pub latency_ms: i32,
    pub packet_loss_percent: i32,
    pub name: String,
}

/// Represents directions for player movement
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Represents player input with direction, sequence number, and timestamp
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct PlayerInput {
    pub dir: Direction,
    pub sequence: u32,
    pub timestamp: u64,
}

/// Represents a player's position in the game world
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Represents a snapshot of a player's position at a specific timestamp
#[derive(Clone)]
pub struct PositionSnapshot {
    pub position: Position,
    pub timestamp: u64,
}

/// Represents a position with an associated timestamp and sequence number for interpolation
#[derive(Debug, Clone)]
pub struct InterpolatedPosition {
    pub position: Position,
    pub timestamp: f32,
    pub sequence: u32,
}

/// Represents the dimensions of the game board
#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub width: usize,
    pub height: usize,
}

/// Represents the state of the game, including players and their positions and sequences
#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub players: Vec<(Uuid, Position, u32)>, // id, pos, color
    pub last_processed: HashMap<Uuid, u32>, // Track inputs
    pub server_timestamp: u64,
}

/// Tests for the types
#[cfg(test)]
mod tests {
    use super::*;
    use bincode;

    #[test]
    fn test_client_message_serialization() {
        // Test each variant of ClientMessage
        let messages = vec![
            ClientMessage::Connect,
            ClientMessage::PlayerId(Uuid::new_v4()),
            ClientMessage::Input(PlayerInput {
                dir: Direction::Up,
                sequence: 42,
                timestamp: 12345,
            }),
            ClientMessage::Ping(54321),
            ClientMessage::Pong(98765),
        ];

        for message in messages {
            // Serialize and deserialize
            let serialized = bincode::serialize(&message).unwrap();
            let deserialized: ClientMessage = bincode::deserialize(&serialized).unwrap();

            // Compare the debug output since ClientMessage doesn't implement PartialEq
            assert_eq!(format!("{:?}", message), format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_network_condition_creation() {
        let condition = NetworkCondition {
            latency_ms: 100,
            packet_loss_percent: 5,
            name: "Test Network".to_string(),
        };

        assert_eq!(condition.latency_ms, 100);
        assert_eq!(condition.packet_loss_percent, 5);
        assert_eq!(condition.name, "Test Network");
    }

    #[test]
    fn test_direction_serialization() {
        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        for dir in directions {
            let serialized = bincode::serialize(&dir).unwrap();
            let deserialized: Direction = bincode::deserialize(&serialized).unwrap();
            assert_eq!(dir as u8, deserialized as u8); // Compare enum variants
        }
    }

    #[test]
    fn test_player_input_serialization() {
        let input = PlayerInput {
            dir: Direction::Right,
            sequence: 123,
            timestamp: 456789,
        };

        let serialized = bincode::serialize(&input).unwrap();
        let deserialized: PlayerInput = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.dir as u8, Direction::Right as u8);
        assert_eq!(deserialized.sequence, 123);
        assert_eq!(deserialized.timestamp, 456789);
    }

    #[test]
    fn test_position_serialization() {
        let pos = Position { x: 10, y: 20 };

        let serialized = bincode::serialize(&pos).unwrap();
        let deserialized: Position = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.x, 10);
        assert_eq!(deserialized.y, 20);
    }

    #[test]
    fn test_position_snapshot() {
        let pos = Position { x: 15, y: 25 };
        let timestamp = 123456789;

        let snapshot = PositionSnapshot {
            position: pos,
            timestamp,
        };

        assert_eq!(snapshot.position.x, 15);
        assert_eq!(snapshot.position.y, 25);
        assert_eq!(snapshot.timestamp, 123456789);
    }

    #[test]
    fn test_interpolated_position() {
        let pos = Position { x: 30, y: 40 };

        let interpol = InterpolatedPosition {
            position: pos,
            timestamp: 12.34,
            sequence: 42,
        };

        assert_eq!(interpol.position.x, 30);
        assert_eq!(interpol.position.y, 40);
        assert_eq!(interpol.timestamp, 12.34);
        assert_eq!(interpol.sequence, 42);
    }

    #[test]
    fn test_board_serialization() {
        let board = Board {
            width: 100,
            height: 200,
        };

        let serialized = bincode::serialize(&board).unwrap();
        let deserialized: Board = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.width, 100);
        assert_eq!(deserialized.height, 200);
    }

    #[test]
    fn test_game_state_serialization() {
        let mut last_processed = HashMap::new();
        let player_id = Uuid::new_v4();
        last_processed.insert(player_id, 42);

        let game_state = GameState {
            players: vec![(player_id, Position { x: 5, y: 10 }, 2)],
            last_processed,
            server_timestamp: 98765,
        };

        let serialized = bincode::serialize(&game_state).unwrap();
        let deserialized: GameState = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.players.len(), 1);
        assert_eq!(deserialized.players[0].0, player_id);
        assert_eq!(deserialized.players[0].1.x, 5);
        assert_eq!(deserialized.players[0].1.y, 10);
        assert_eq!(deserialized.players[0].2, 2);
        assert_eq!(deserialized.last_processed.get(&player_id), Some(&42));
        assert_eq!(deserialized.server_timestamp, 98765);
    }
}