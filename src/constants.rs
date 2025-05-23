use std::time::Duration;

/// Constants for the game state
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Constants for the game state
pub const INTERPOLATION_DELAY: f32 = 0.016;

/// Constants for window size
pub const WINDOW_TITLE: &str = "Netcode Game";
pub const HIGH_DPI: bool = true;
pub const WINDOW_RESIZABLE: bool = false;
pub const WINDOW_WIDTH: i32 = 1024;
pub const WINDOW_HEIGHT: i32 = 768;

/// Constants for the board
pub const BOARD_WIDTH: i32 = 1024;
pub const BOARD_HEIGHT: i32 = 768;

/// Constants for the player
pub const PLAYER_SIZE: i32 = 20;
pub const PLAYER_SPEED: i32 = 5;

/// Constants for server
pub const BROADCAST_INTERVAL: Duration = Duration::from_millis(16); // 60fps game state updates

/// Constants for performance testing
pub const TEST_DURATION: Duration = Duration::from_millis(1000);
pub const PERFORMANCE_TEST_FREQUENCY: Duration = Duration::from_secs(10); // 10 seconds

/// Constants for network
pub const DELAY_MS: i32 = 0;
pub const PACKET_LOSS: i32 = 0;
pub const PING_INTERVAL: Duration = Duration::from_secs(1); // 1 second between pings

/// Constants for inputs from players
pub const INITIAL_DELAY: f32 = 0.15;
pub const REPEAT_START: f32 = 0.1;
pub const REPEAT_MIN: f32 = 0.0;
pub const REPEAT_ACCEL: f32 = 0.3;

/// New constants for improved interpolation
pub const MAX_POSITION_HISTORY: usize = 30;
pub const PREDICTION_ERROR_THRESHOLD: f32 = 5.0;
pub const MAX_INTERPOLATION_TIME: f32 = 0.1;

/// Constants for player management
pub const ID_GRACE_PERIOD: Duration = Duration::from_secs(10); // 30 seconds grace period for reconnection


