use std::time::Duration;

/// Constants for the game state
pub const TIMEOUT: Duration = Duration::from_secs(5); // Timeout for player inactivity

/// Constants for the game state
pub const INTERPOLATION_DELAY: f32 = 0.016; // 16ms for 60fps interpolation

/// Constants for window size
pub const WINDOW_TITLE: &str = "Netcode Game"; // Title of the game window
pub const WINDOW_RESIZABLE: bool = false; // Whether the window is resizable
pub const WINDOW_WIDTH: i32 = 1024; // Width of the game window
pub const WINDOW_HEIGHT: i32 = 768; // Height of the game window
pub const TOOL_BAR_HEIGHT: i32 = 40; // Height of the toolbar at the bottom of the window

/// Constants for the board
pub const BOARD_WIDTH: i32 = 1024; // Width of the game board (usually the same as window width)
pub const BOARD_HEIGHT: i32 = 768; // Height of the game board (usually the same as window height)

/// Constants for the player
pub const PLAYER_SIZE: i32 = 20; // Size of the player character square
pub const PLAYER_SPEED: i32 = 5; // Speed of the player character movement in pixels per frame

/// Constants for server
pub const BROADCAST_INTERVAL: Duration = Duration::from_millis(16); // 60fps game state updates

/// Constants for performance testing
pub const TEST_DURATION: Duration = Duration::from_millis(1000); // 1 second for performance tests
pub const PERFORMANCE_TEST_FREQUENCY: Duration = Duration::from_secs(10); // Frequency of performance tests

/// Constants for network
pub const DELAY_MS: i32 = 0; // Network delay in milliseconds
pub const PACKET_LOSS: i32 = 0; // Packet loss percentage (0-100)
pub const PING_INTERVAL: Duration = Duration::from_secs(1); // Interval for pinging the server

/// Constants for inputs from players
pub const INITIAL_DELAY: f32 = 0.15; // Initial delay before input starts repeating
pub const REPEAT_START: f32 = 0.1; // Delay before input starts repeating
pub const REPEAT_MIN: f32 = 0.0; // Minimum delay between repeated inputs while key is held down
pub const REPEAT_ACCEL: f32 = 0.3; // Acceleration factor for repeat input delay

/// New constants for improved interpolation
pub const MAX_POSITION_HISTORY: usize = 30; // Maximum number of position snapshots to keep for interpolation
pub const PREDICTION_ERROR_THRESHOLD: f32 = 5.0; // Maximum allowed prediction error before triggering reconciliation
pub const MAX_INTERPOLATION_TIME: f32 = 0.1; // Maximum time to interpolate positions (in seconds)



