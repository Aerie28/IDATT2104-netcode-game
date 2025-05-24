use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH, PLAYER_SIZE, PLAYER_SPEED, TOOL_BAR_HEIGHT};
use crate::types::{Position, PlayerInput, Direction};

use std::collections::VecDeque;

/// Represents the state of player movement prediction and reconciliation
pub struct PredictionState {
    pub next_sequence: u32,
    pub pending_inputs: VecDeque<(u32, PlayerInput)>,
    pub position_history: VecDeque<(u32, Position)>, // (sequence, position)
    pub last_confirmed_sequence: u32,
    pub last_confirmed_position: Position,
    pub last_reconciliation_time: f64,
}

/// Implementation of the PredictionState
impl PredictionState {
    /// Creates a new PredictionState with the initial position
    pub fn new(initial_position: Position) -> Self {
        Self {
            next_sequence: 0,
            pending_inputs: VecDeque::new(),
            position_history: VecDeque::new(),
            last_confirmed_sequence: 0,
            last_confirmed_position: initial_position,
            last_reconciliation_time: 0.0,
        }
    }

    /// Adds a prediction input to the pending inputs queue
    pub fn apply_prediction(&mut self, input: PlayerInput, current_position: &mut Position) {
        // Store the current position before applying the prediction
        self.position_history.push_back((input.sequence, *current_position));
        
        // Apply the movement prediction
        match input.dir {
            Direction::Up => current_position.y = current_position.y.saturating_sub(PLAYER_SPEED).max(PLAYER_SIZE),
            Direction::Down => current_position.y = current_position.y.saturating_add(PLAYER_SPEED).min(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT),
            Direction::Left => current_position.x = current_position.x.saturating_sub(PLAYER_SPEED).max(PLAYER_SIZE),
            Direction::Right => current_position.x = current_position.x.saturating_add(PLAYER_SPEED).min(BOARD_WIDTH - (PLAYER_SIZE)),
        }
    }

    /// Reconciles the client state with the server state
    pub fn reconcile(&mut self, server_position: Position, server_sequence: u32, current_time: f64) {
        // If we've received a newer server state
        if server_sequence > self.last_confirmed_sequence {
            // Calculate time since last reconciliation
            let time_since_last = current_time - self.last_reconciliation_time;
            self.last_reconciliation_time = current_time;

            // Update our confirmed state
            self.last_confirmed_sequence = server_sequence;
            self.last_confirmed_position = server_position;

            // Remove all pending inputs that have been confirmed
            while let Some((seq, _)) = self.pending_inputs.front() {
                if *seq <= server_sequence {
                    self.pending_inputs.pop_front();
                } else {
                    break;
                }
            }

            // Remove old position history
            while let Some((seq, _)) = self.position_history.front() {
                if *seq <= server_sequence {
                    self.position_history.pop_front();
                } else {
                    break;
                }
            }

            // If we have a large gap between server and client sequence,
            // or if it's been too long since last reconciliation, be more aggressive
            if server_sequence - self.last_confirmed_sequence > 5 || time_since_last > 0.5 {
                // Clear all pending inputs and position history
                self.pending_inputs.clear();
                self.position_history.clear();
            }
        }
    }

    /// Reapplies all pending inputs to the current position
    pub fn reapply_pending_inputs(&mut self, current_position: &mut Position) {
        // Start from the last confirmed position
        *current_position = self.last_confirmed_position;

        // Collect inputs into a Vec to avoid borrowing issues
        let inputs: Vec<_> = self.pending_inputs.iter().map(|(_, input)| input.clone()).collect();
        
        // Reapply all pending inputs
        for input in inputs {
            self.apply_prediction(input, current_position);
        }
    }

    /// Gets error in prediction by comparing the last confirmed position with the server position
    pub fn get_prediction_error(&self, server_position: Position) -> f32 {
        let dx = (server_position.x - self.last_confirmed_position.x) as f32;
        let dy = (server_position.y - self.last_confirmed_position.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Tests for the PredictionState
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_prediction_state() {
        let initial_position = Position { x: 100, y: 100 };
        let state = PredictionState::new(initial_position);

        assert_eq!(state.next_sequence, 0);
        assert!(state.pending_inputs.is_empty());
        assert!(state.position_history.is_empty());
        assert_eq!(state.last_confirmed_sequence, 0);
        assert_eq!(state.last_confirmed_position.x, initial_position.x);
        assert_eq!(state.last_confirmed_position.y, initial_position.y);
        assert_eq!(state.last_reconciliation_time, 0.0);
    }

    #[test]
    fn test_apply_prediction_up() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);
        let mut position = initial_position;

        let input = PlayerInput {
            dir: Direction::Up,
            sequence: 0,
            timestamp: 0,
        };

        state.apply_prediction(input, &mut position);

        assert_eq!(position.x, initial_position.x);
        assert_eq!(position.y, initial_position.y - PLAYER_SPEED);
        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].0, 0);  // sequence
        assert_eq!(state.position_history[0].1.x, initial_position.x);  // original position
        assert_eq!(state.position_history[0].1.y, initial_position.y);
    }

    #[test]
    fn test_apply_prediction_down() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);
        let mut position = initial_position;

        let input = PlayerInput {
            dir: Direction::Down,
            sequence: 1,
            timestamp: 0,
        };

        state.apply_prediction(input, &mut position);

        assert_eq!(position.x, initial_position.x);
        assert_eq!(position.y, initial_position.y + PLAYER_SPEED);
        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].0, 1);  // sequence
    }

    #[test]
    fn test_apply_prediction_left() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);
        let mut position = initial_position;

        let input = PlayerInput {
            dir: Direction::Left,
            sequence: 2,
            timestamp: 0,
        };

        state.apply_prediction(input, &mut position);

        assert_eq!(position.x, initial_position.x - PLAYER_SPEED);
        assert_eq!(position.y, initial_position.y);
        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].0, 2);  // sequence
    }

    #[test]
    fn test_apply_prediction_right() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);
        let mut position = initial_position;

        let input = PlayerInput {
            dir: Direction::Right,
            sequence: 3,
            timestamp: 0,
        };

        state.apply_prediction(input, &mut position);

        assert_eq!(position.x, initial_position.x + PLAYER_SPEED);
        assert_eq!(position.y, initial_position.y);
        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].0, 3);  // sequence
    }

    #[test]
    fn test_prediction_boundary_limits() {
        // Test hitting the left boundary
        let mut state = PredictionState::new(Position { x: PLAYER_SIZE + 1, y: 100 });
        let mut position = Position { x: PLAYER_SIZE + 1, y: 100 };

        state.apply_prediction(PlayerInput { dir: Direction::Left, sequence: 1, timestamp: 0 }, &mut position);
        assert_eq!(position.x, PLAYER_SIZE);  // Should stop at boundary

        // Test hitting the right boundary
        position = Position { x: BOARD_WIDTH - PLAYER_SIZE - 1, y: 100 };
        state.apply_prediction(PlayerInput { dir: Direction::Right, sequence: 2, timestamp: 0 }, &mut position);
        assert_eq!(position.x, BOARD_WIDTH - PLAYER_SIZE);  // Should stop at boundary

        // Test hitting the top boundary
        position = Position { x: 100, y: PLAYER_SIZE + 1 };
        state.apply_prediction(PlayerInput { dir: Direction::Up, sequence: 3, timestamp: 0 }, &mut position);
        assert_eq!(position.y, PLAYER_SIZE);  // Should stop at boundary

        // Test hitting the bottom boundary
        position = Position { x: 100, y: BOARD_HEIGHT - PLAYER_SIZE - TOOL_BAR_HEIGHT - 1 };
        state.apply_prediction(PlayerInput { dir: Direction::Down, sequence: 4, timestamp: 0 }, &mut position);
        assert_eq!(position.y, BOARD_HEIGHT - PLAYER_SIZE - TOOL_BAR_HEIGHT);  // Should stop at boundary
    }

    #[test]
    fn test_reconcile_normal_case() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);

        // Initialize last_reconciliation_time to avoid the time-based aggressive clean
        state.last_reconciliation_time = 0.8; // So the difference will be 0.2, below threshold

        // Add some pending inputs
        state.pending_inputs.push_back((1, PlayerInput { dir: Direction::Up, sequence: 1, timestamp: 0 }));
        state.pending_inputs.push_back((2, PlayerInput { dir: Direction::Left, sequence: 2, timestamp: 0 }));
        state.pending_inputs.push_back((3, PlayerInput { dir: Direction::Right, sequence: 3, timestamp: 0 }));

        // Add position history
        state.position_history.push_back((1, Position { x: 100, y: 100 }));
        state.position_history.push_back((2, Position { x: 100, y: 90 }));
        state.position_history.push_back((3, Position { x: 90, y: 90 }));

        // Server confirms up to sequence 2
        let server_position = Position { x: 95, y: 85 };  // Slightly different from client's prediction
        state.reconcile(server_position, 2, 1.0);

        // Check state after reconciliation
        assert_eq!(state.last_confirmed_sequence, 2);
        assert_eq!(state.last_confirmed_position.x, 95);
        assert_eq!(state.last_confirmed_position.y, 85);
        assert_eq!(state.pending_inputs.len(), 1);  // Only sequence 3 should remain
        assert_eq!(state.pending_inputs[0].0, 3);
        assert_eq!(state.position_history.len(), 1);  // Only sequence 3 position should remain
        assert_eq!(state.position_history[0].0, 3);
    }

    #[test]
    fn test_reapply_pending_inputs() {
        let initial_position = Position { x: 100, y: 100 };
        let mut state = PredictionState::new(initial_position);
        let mut current_position = Position { x: 200, y: 200 };  // Intentionally different

        // Add pending inputs: right, right, down
        state.pending_inputs.push_back((1, PlayerInput { dir: Direction::Right, sequence: 1, timestamp: 0 }));
        state.pending_inputs.push_back((2, PlayerInput { dir: Direction::Right, sequence: 2, timestamp: 0 }));
        state.pending_inputs.push_back((3, PlayerInput { dir: Direction::Down, sequence: 3, timestamp: 0 }));

        // Reapply all inputs
        state.reapply_pending_inputs(&mut current_position);

        // Should start from last_confirmed_position (100, 100)
        // Then apply: right (+PLAYER_SPEED, 0), right (+PLAYER_SPEED, 0), down (0, +PLAYER_SPEED)
        let expected_x = initial_position.x + 2 * PLAYER_SPEED;
        let expected_y = initial_position.y + PLAYER_SPEED;

        assert_eq!(current_position.x, expected_x);
        assert_eq!(current_position.y, expected_y);
    }

    #[test]
    fn test_prediction_error_calculation() {
        let initial_position = Position { x: 100, y: 100 };
        let state = PredictionState::new(initial_position);

        // Test with position offset by 3 horizontally and 4 vertically (5 units total distance)
        let server_position = Position { x: 103, y: 104 };
        let error = state.get_prediction_error(server_position);

        // Error should be sqrt(3^2 + 4^2) = 5.0
        assert_eq!(error, 5.0);
    }
}