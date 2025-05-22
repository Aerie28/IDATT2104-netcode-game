use macroquad::prelude::*;
use std::collections::HashMap;
use crate::types::{PlayerInput, Direction, Position, PredictionState};
use crate::network::NetworkClient;
use crate::constants::{INITIAL_DELAY, REPEAT_START, REPEAT_MIN, REPEAT_ACCEL, PLAYER_SPEED, DELAY_MS, PACKET_LOSS};

pub struct InputHandler {
    key_timers: HashMap<KeyCode, f32>,
    key_states: HashMap<KeyCode, bool>,
    pub delay_ms: i32,
    pub packet_loss: i32,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            key_timers: HashMap::new(),
            key_states: HashMap::new(),
            delay_ms: DELAY_MS,
            packet_loss: PACKET_LOSS,
        }
    }
    pub fn handle_selector_input(&mut self) {
        if is_key_pressed(KeyCode::V) {
            self.delay_ms = (self.delay_ms - 10).max(0);
        }
        if is_key_pressed(KeyCode::B) {
            self.delay_ms = (self.delay_ms + 10).min(1000);
        }
        if is_key_pressed(KeyCode::N) {
            self.packet_loss = (self.packet_loss - 1).max(0);
        }
        if is_key_pressed(KeyCode::M) {
            self.packet_loss = (self.packet_loss + 1).min(100);
        }
    }
    pub fn handle_input(
        &mut self,
        my_pos: &mut Position,
        net: &mut NetworkClient,
        dt: f32,
        prediction: &mut PredictionState,
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

                let input = PlayerInput {
                    dir,
                    sequence: prediction.next_sequence,
                };
                prediction.pending_inputs.push_back((prediction.next_sequence, input));
                prediction.next_sequence += 1;
                net.send_input(input);

                // Predict movement
                match dir {
                    Direction::Up => my_pos.y = my_pos.y.saturating_sub(PLAYER_SPEED),
                    Direction::Down => my_pos.y = my_pos.y.saturating_add(PLAYER_SPEED),
                    Direction::Left => my_pos.x = my_pos.x.saturating_sub(PLAYER_SPEED),
                    Direction::Right => my_pos.x = my_pos.x.saturating_add(PLAYER_SPEED),
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

                    let input = PlayerInput {
                        dir,
                        sequence: prediction.next_sequence,
                    };
                    prediction.pending_inputs.push_back((prediction.next_sequence, input));
                    prediction.next_sequence += 1;
                    net.send_input(input);

                    // Predict movement
                    match dir {
                        Direction::Up => my_pos.y = my_pos.y.saturating_sub(PLAYER_SPEED),
                        Direction::Down => my_pos.y = my_pos.y.saturating_add(PLAYER_SPEED),
                        Direction::Left => my_pos.x = my_pos.x.saturating_sub(PLAYER_SPEED),
                        Direction::Right => my_pos.x = my_pos.x.saturating_add(PLAYER_SPEED),
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