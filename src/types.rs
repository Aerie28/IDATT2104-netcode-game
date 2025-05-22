use serde::{Deserialize, Serialize};
use std::net::SocketAddr;


#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Connect,
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
    pub seq: u32,
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
pub struct PlayerSnapshot {
    pub addr: SocketAddr,
    pub pos: Position,
    pub color: u32,
    pub active: bool,
    pub last_input_seq: u32, 
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RemotePlayerState {
    pub current: Position,
    pub previous: Position,
    pub last_update_time: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub players: Vec<PlayerSnapshot>,
}
