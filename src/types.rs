
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub dir: Direction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}