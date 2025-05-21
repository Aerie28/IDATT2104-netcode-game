use std::time::Duration;

/// Constants for the game state
pub const TIMEOUT: Duration = Duration::from_secs(30);

/// Constants for the board
pub const FIELD_WIDTH: f32 = 64.0;
pub const FIELD_HEIGHT: f32 = 48.0;
pub const SERVER_WIDTH: f32 = 640.0;
pub const SERVER_HEIGHT: f32 = 480.0;

/// Constants for the player
pub const PLAYER_SIZE: f32 = 10.0;
pub const PLAYER_SPEED: u32 = 5;

/// Constants for server
pub const BROADCAST_INTERVAL: Duration = Duration::from_secs(20);

/// Constants for inputs from players
pub const INITIAL_DELAY: f32 = 0.35;
pub const REPEAT_START: f32 = 0.15;
pub const REPEAT_MIN: f32 = 0.05;
pub const REPEAT_ACCEL: f32 = 0.90;


