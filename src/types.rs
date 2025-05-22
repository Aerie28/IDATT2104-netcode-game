use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
    PlayerId(Uuid),
    Input(PlayerInput),
    Disconnect,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct PlayerInput {
    pub dir: Direction,
    pub sequence: u32,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub players: Vec<(Uuid, Position, u32, bool)>, // id, pos, color, active
    pub last_processed: HashMap<Uuid, u32>, // Track inputs
    pub server_timestamp: u64,
}