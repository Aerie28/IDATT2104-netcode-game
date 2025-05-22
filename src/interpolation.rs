use std::collections::VecDeque;
use std::time::Instant;
use crate::types::Position;
use crate::constants::{INTERPOLATION_DELAY, MAX_POSITION_HISTORY, MAX_INTERPOLATION_TIME};

#[derive(Debug, Clone)]
pub struct InterpolatedPosition {
    pub position: Position,
    pub timestamp: f32,
    pub sequence: u32,
}

pub struct InterpolationState {
    position_history: VecDeque<InterpolatedPosition>,
    interpolation_delay: f32,
    last_sequence: u32,
    last_position: Option<Position>,
}

impl InterpolationState {
    pub fn new() -> Self {
        Self {
            position_history: VecDeque::with_capacity(MAX_POSITION_HISTORY),
            interpolation_delay: INTERPOLATION_DELAY,
            last_sequence: 0,
            last_position: None,
        }
    }

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

    pub fn update(&mut self, current_time: f32) {
        // Remove positions that are too old
        while let Some(oldest) = self.position_history.front() {
            if current_time - oldest.timestamp > MAX_INTERPOLATION_TIME {
                self.position_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn set_interpolation_delay(&mut self, delay: f32) {
        self.interpolation_delay = delay;
    }

    pub fn get_last_sequence(&self) -> u32 {
        self.last_sequence
    }
} 