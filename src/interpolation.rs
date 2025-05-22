use std::collections::VecDeque;
use std::time::Instant;
use crate::types::Position;
use crate::constants::INTERPOLATION_DELAY;

const MAX_INTERPOLATION_TIME: f32 = 0.5; // Maximum time to keep old positions

#[derive(Debug, Clone)]
pub struct InterpolatedPosition {
    pub position: Position,
    pub timestamp: f32,
}

pub struct InterpolationState {
    position_history: VecDeque<InterpolatedPosition>,
    interpolation_delay: f32,
}

impl InterpolationState {
    pub fn new() -> Self {
        Self {
            position_history: VecDeque::new(),
            interpolation_delay: INTERPOLATION_DELAY,
        }
    }

    pub fn add_position(&mut self, position: Position, timestamp: f32) {
        // Add new position to history
        self.position_history.push_back(InterpolatedPosition {
            position,
            timestamp,
        });

        // Remove old positions
        let current_time = timestamp;
        while let Some(oldest) = self.position_history.front() {
            if current_time - oldest.timestamp > MAX_INTERPOLATION_TIME {
                self.position_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_interpolated_position(&self, current_time: f32) -> Option<Position> {
        if self.position_history.len() < 2 {
            return None;
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
                let t = (target_time - prev.timestamp) / (next.timestamp - prev.timestamp);
                Some(Position {
                    x: (prev.position.x as f32 + (next.position.x - prev.position.x) as f32 * t) as i32,
                    y: (prev.position.y as f32 + (next.position.y - prev.position.y) as f32 * t) as i32,
                })
            }
            _ => None,
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
} 