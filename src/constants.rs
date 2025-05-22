use std::time::Duration;

/// Constants for the game state
pub const TIMEOUT: Duration = Duration::from_secs(30);

pub const INTERPOLATION_DELAY: f32 = 0.05;

/// Constants for the board
pub const BOARD_WIDTH: f32 = 640.0;
pub const BOARD_HEIGHT: f32 = 480.0;

/// Constants for the player
pub const PLAYER_SIZE: f32 = 20.0;
pub const PLAYER_SPEED: i32 = 5;

/// Constants for server
pub const BROADCAST_INTERVAL: Duration = Duration::from_millis(100);

/// Constants for network

pub const DELAY_MS: i32 = 0;
pub const PACKET_LOSS: i32 = 0;

/// Constants for inputs from players
pub const INITIAL_DELAY: f32 = 0.15;
pub const REPEAT_START: f32 = 0.1;
pub const REPEAT_MIN: f32 = 0.0;
pub const REPEAT_ACCEL: f32 = 0.3;


