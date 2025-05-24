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
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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