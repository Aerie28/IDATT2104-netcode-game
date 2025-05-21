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
    pub players: Vec<(SocketAddr, Position, u32, bool)>, // addr, pos, color, active
}
