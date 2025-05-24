use std::collections::VecDeque;
use crate::types::{Position, PlayerInput, Direction};
use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH, PLAYER_SIZE, PLAYER_SPEED, TOOL_BAR_HEIGHT};

pub struct PredictionState {
    pub next_sequence: u32,
    pub pending_inputs: VecDeque<(u32, PlayerInput)>,
    pub position_history: VecDeque<(u32, Position)>, // (sequence, position)
    pub last_confirmed_sequence: u32,
    pub last_confirmed_position: Position,
    pub last_reconciliation_time: f64,
}

impl PredictionState {
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

    pub fn apply_prediction(&mut self, input: PlayerInput, current_position: &mut Position) {
        // Store the current position before applying the prediction
        self.position_history.push_back((input.sequence, *current_position));
        
        // Apply the movement prediction
        match input.dir {
            Direction::Up => current_position.y = (current_position.y.saturating_sub(PLAYER_SPEED)).max(PLAYER_SIZE),
            Direction::Down => current_position.y = (current_position.y.saturating_add(PLAYER_SPEED)).min(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT),
            Direction::Left => current_position.x = (current_position.x.saturating_sub(PLAYER_SPEED)).max(PLAYER_SIZE),
            Direction::Right => current_position.x = (current_position.x.saturating_add(PLAYER_SPEED)).min(BOARD_WIDTH - (PLAYER_SIZE)),
        }
    }

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

    pub fn get_prediction_error(&self, server_position: Position) -> f32 {
        let dx = (server_position.x - self.last_confirmed_position.x) as f32;
        let dy = (server_position.y - self.last_confirmed_position.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
} 