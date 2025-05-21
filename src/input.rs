use macroquad::prelude::*;
use std::collections::HashMap;
use crate::types::{PlayerInput, Direction, Position};
use crate::network::NetworkClient;

const INITIAL_DELAY: f32 = 0.35;
const REPEAT_START: f32 = 0.15;
const REPEAT_MIN: f32 = 0.05;
const REPEAT_ACCEL: f32 = 0.90;

pub struct InputHandler {
    key_timers: HashMap<KeyCode, f32>,
    key_states: HashMap<KeyCode, bool>,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            key_timers: HashMap::new(),
            key_states: HashMap::new(),
        }
    }

    pub fn handle_input(
        &mut self,
        my_pos: &mut Position,
        net: &mut NetworkClient,
        dt: f32,
    ) {
        // Input handling and prediction
        for &key in &[KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D] {
            let is_down = is_key_down(key);
            let was_down = *self.key_states.get(&key).unwrap_or(&false);

            if is_down && !was_down {
                // Key pressed, initialize timer
                self.key_timers.insert(key, INITIAL_DELAY);
                self.key_states.insert(key, true);

                // Send input immediately
                let dir = match key {
                    KeyCode::W => Direction::Up,
                    KeyCode::A => Direction::Left,
                    KeyCode::S => Direction::Down,
                    KeyCode::D => Direction::Right,
                    _ => continue,
                };
                net.send_input(PlayerInput { dir });

                // Predict movement
                match dir {
                    Direction::Up => my_pos.y = my_pos.y.saturating_sub(5),
                    Direction::Down => my_pos.y = my_pos.y.saturating_add(5),
                    Direction::Left => my_pos.x = my_pos.x.saturating_sub(5),
                    Direction::Right => my_pos.x = my_pos.x.saturating_add(5),
                }
            } else if is_down && was_down {
                // Key is still down, update timer
                let timer = self.key_timers.entry(key).or_insert(INITIAL_DELAY);
                *timer -= dt;

                if *timer <= 0.0 {
                    // Accelerate repeat
                    let next_interval = (*timer + REPEAT_START) * REPEAT_ACCEL;
                    *timer = next_interval.max(REPEAT_MIN);

                    // Send input immediately
                    let dir = match key {
                        KeyCode::W => Direction::Up,
                        KeyCode::A => Direction::Left,
                        KeyCode::S => Direction::Down,
                        KeyCode::D => Direction::Right,
                        _ => continue,
                    };
                    net.send_input(PlayerInput { dir });

                    // Predict movement
                    match dir {
                        Direction::Up => my_pos.y = my_pos.y.saturating_sub(5),
                        Direction::Down => my_pos.y = my_pos.y.saturating_add(5),
                        Direction::Left => my_pos.x = my_pos.x.saturating_sub(5),
                        Direction::Right => my_pos.x = my_pos.x.saturating_add(5),
                    }
                }
            } else if !is_down && was_down {
                // Key released: reset state
                self.key_states.insert(key, false);
                self.key_timers.remove(&key);
            }
        }
    }
}