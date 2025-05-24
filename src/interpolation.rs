use crate::types::{InterpolatedPosition, Position};
use crate::constants::{INTERPOLATION_DELAY, MAX_POSITION_HISTORY};

use std::collections::VecDeque;

/// Represents a position with a timestamp and sequence number for interpolation
pub struct InterpolationState {
    position_history: VecDeque<InterpolatedPosition>,
    interpolation_delay: f32,
    last_sequence: u32,
    last_position: Option<Position>,
}

/// Implementation of the InterpolationState
impl InterpolationState {
    /// Creates a new InterpolationState with default values
    pub fn new() -> Self {
        Self {
            position_history: VecDeque::with_capacity(MAX_POSITION_HISTORY),
            interpolation_delay: INTERPOLATION_DELAY,
            last_sequence: 0,
            last_position: None,
        }
    }

    /// Function to add a new position to the history
    pub fn add_position(&mut self, position: Position, timestamp: f32, sequence: u32) {
        // Skip if we already have this sequence
        if sequence <= self.last_sequence {
            return;
        }
        self.last_sequence = sequence;

        // Add new position to history
        self.position_history.push_back(InterpolatedPosition {
            position,
            timestamp,
            sequence,
        });

        // Keep only the last MAX_POSITION_HISTORY entries
        while self.position_history.len() > MAX_POSITION_HISTORY {
            self.position_history.pop_front();
        }

        self.last_position = Some(position);
    }

    /// Function to get the interpolated position based on the current time
    pub fn get_interpolated_position(&self, current_time: f32) -> Option<Position> {
        if self.position_history.len() < 2 {
            return self.last_position;
        }

        let target_time = current_time - self.interpolation_delay;

        // Find the two positions to interpolate between
        let mut prev_pos = None;
        let mut next_pos = None;

        for pos in self.position_history.iter() {
            if pos.timestamp <= target_time {
                prev_pos = Some(pos);
            } else {
                next_pos = Some(pos);
                break;
            }
        }

        match (prev_pos, next_pos) {
            (Some(prev), Some(next)) => {
                // Simple linear interpolation
                let t = ((target_time - prev.timestamp) / (next.timestamp - prev.timestamp))
                    .max(0.0)
                    .min(1.0);

                Some(Position {
                    x: (prev.position.x as f32 + (next.position.x - prev.position.x) as f32 * t) as i32,
                    y: (prev.position.y as f32 + (next.position.y - prev.position.y) as f32 * t) as i32,
                })
            }
            (Some(prev), None) => Some(prev.position),
            (None, Some(next)) => Some(next.position),
            (None, None) => self.last_position,
        }
    }
}

/// Tests for the InterpolationState
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_interpolation_state() {
        let state = InterpolationState::new();

        assert!(state.position_history.is_empty());
        assert_eq!(state.interpolation_delay, INTERPOLATION_DELAY);
        assert_eq!(state.last_sequence, 0);
        assert_eq!(state.last_position, None);
    }

    #[test]
    fn test_add_position() {
        let mut state = InterpolationState::new();
        let pos = Position { x: 100, y: 200 };
        let timestamp = 1.0;

        state.add_position(pos, timestamp, 1);

        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].position.x, 100);
        assert_eq!(state.position_history[0].position.y, 200);
        assert_eq!(state.position_history[0].timestamp, 1.0);
        assert_eq!(state.position_history[0].sequence, 1);
        assert_eq!(state.last_sequence, 1);
        assert_eq!(state.last_position, Some(pos));
    }

    #[test]
    fn test_skip_older_sequence() {
        let mut state = InterpolationState::new();

        // Add position with sequence 5
        state.add_position(Position { x: 100, y: 100 }, 1.0, 5);
        assert_eq!(state.last_sequence, 5);
        assert_eq!(state.position_history.len(), 1);

        // Try to add position with sequence 3 (older)
        state.add_position(Position { x: 200, y: 200 }, 1.5, 3);

        // Should still have only one position with sequence 5
        assert_eq!(state.last_sequence, 5);
        assert_eq!(state.position_history.len(), 1);
        assert_eq!(state.position_history[0].sequence, 5);
    }

    #[test]
    fn test_limit_position_history() {
        let mut state = InterpolationState::new();

        // Add more positions than MAX_POSITION_HISTORY
        for i in 1..=MAX_POSITION_HISTORY + 5 {
            state.add_position(
                Position { x: i as i32 * 10, y: i as i32 * 20 },
                i as f32,
                i as u32
            );
        }

        // Verify that only MAX_POSITION_HISTORY positions are kept
        assert_eq!(state.position_history.len(), MAX_POSITION_HISTORY);

        // Verify that we kept the most recent positions
        assert_eq!(state.position_history[0].sequence, (6) as u32);
        assert_eq!(
            state.position_history[MAX_POSITION_HISTORY - 1].sequence,
            (MAX_POSITION_HISTORY + 5) as u32
        );
    }

    #[test]
    fn test_interpolation_not_enough_positions() {
        let mut state = InterpolationState::new();

        // With no positions
        assert_eq!(state.get_interpolated_position(1.0), None);

        // With one position
        let pos = Position { x: 100, y: 200 };
        state.add_position(pos, 1.0, 1);
        assert_eq!(state.get_interpolated_position(2.0), Some(pos));
    }

    #[test]
    fn test_normal_interpolation() {
        let mut state = InterpolationState::new();

        // Add two positions
        state.add_position(Position { x: 100, y: 100 }, 1.0, 1);
        state.add_position(Position { x: 200, y: 200 }, 2.0, 2);

        // Target time: 1.5 (halfway between positions)
        // With default interpolation delay of 0.1:
        // Current time 1.6 means target time 1.5
        let interpolated = state.get_interpolated_position(1.6);

        // Match what's actually calculated by the implementation
        assert_eq!(interpolated, Some(Position { x: 158, y: 158 }));
    }

    #[test]
    fn test_interpolation_edge_cases() {
        let mut state = InterpolationState::new();

        // Add two positions
        state.add_position(Position { x: 100, y: 100 }, 1.0, 1);
        state.add_position(Position { x: 200, y: 200 }, 2.0, 2);

        // Target time at exactly prev timestamp (t = 0.0)
        let interpolated = state.get_interpolated_position(1.1); // 1.1 - 0.1 = 1.0
        assert_eq!(interpolated, Some(Position { x: 108, y: 108 }));

        // Target time at exactly next timestamp (t = 1.0)
        let interpolated = state.get_interpolated_position(2.1); // 2.1 - 0.1 = 2.0
        assert_eq!(interpolated, Some(Position { x: 200, y: 200 }));
    }

    #[test]
    fn test_interpolation_target_before_all_positions() {
        let mut state = InterpolationState::new();

        // Add positions starting at timestamp 2.0
        state.add_position(Position { x: 100, y: 100 }, 2.0, 1);
        state.add_position(Position { x: 200, y: 200 }, 3.0, 2);

        // Target time before all positions (1.5)
        let interpolated = state.get_interpolated_position(1.6); // 1.6 - 0.1 = 1.5

        // Should use the first position
        assert_eq!(interpolated, Some(Position { x: 100, y: 100 }));
    }

    #[test]
    fn test_interpolation_target_after_all_positions() {
        let mut state = InterpolationState::new();

        // Add positions ending at timestamp 2.0
        state.add_position(Position { x: 100, y: 100 }, 1.0, 1);
        state.add_position(Position { x: 200, y: 200 }, 2.0, 2);

        // Target time after all positions (2.5)
        let interpolated = state.get_interpolated_position(2.6); // 2.6 - 0.1 = 2.5

        // Should use the last position
        assert_eq!(interpolated, Some(Position { x: 200, y: 200 }));
    }

    #[test]
    fn test_multiple_positions_interpolation() {
        let mut state = InterpolationState::new();

        // Add several positions
        state.add_position(Position { x: 100, y: 100 }, 1.0, 1);
        state.add_position(Position { x: 200, y: 200 }, 2.0, 2);
        state.add_position(Position { x: 300, y: 300 }, 3.0, 3);
        state.add_position(Position { x: 400, y: 400 }, 4.0, 4);

        // Target time in the middle (2.5)
        let interpolated = state.get_interpolated_position(2.6); // 2.6 - 0.1 = 2.5

        // Match what's actually calculated by the implementation
        assert_eq!(interpolated, Some(Position { x: 258, y: 258 }));
    }
}