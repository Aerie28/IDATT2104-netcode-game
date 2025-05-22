use uuid::Uuid;
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::Instant,
};
use crate::colors::player_colors;
use crate::types::{Position, PlayerInput, Direction, GameState};
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT, PLAYER_SPEED, TIMEOUT};

const MAX_POSITION_HISTORY: usize = 60; // Store 1 second of history at 60fps

#[derive(Clone)]
pub struct PositionSnapshot {
    pub position: Position,
    pub timestamp: u64,
}

/// Stores state for one player
pub struct PlayerState {
    pub position: Position,
    pub color: u32,
    pub last_active: Instant,
    pub active: bool,
    pub position_history: Vec<PositionSnapshot>,
}

pub struct Game {
    players: HashMap<SocketAddr, PlayerState>,
    id_to_addr: HashMap<Uuid, SocketAddr>,
    addr_to_id: HashMap<SocketAddr, Uuid>,
    last_processed: HashMap<Uuid, u32>, // Track inputs
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            id_to_addr: HashMap::new(),
            addr_to_id: HashMap::new(),
            last_processed: HashMap::new(),
        }
    }

    /// Handles new connection by adding player at random pos/color
    pub fn connect_player(&mut self, addr: SocketAddr) -> Uuid {
        use rand::Rng;

        if self.players.contains_key(&addr) {
            // Player already connected
            return *self.addr_to_id.get(&addr).unwrap();
        }

        let mut rng = rand::rng();
        let x = rng.random_range(0..(BOARD_WIDTH as i32));
        let y = rng.random_range(0..(BOARD_HEIGHT as i32));
        
        // Pick a color from the palette randomly
        let palette = player_colors::get_palette();
        let color_base = palette[rng.random_range(0..palette.len())];
        // Pack the color as u32 for serialization
        let color = ((color_base.r * 255.0) as u32) << 16
            | ((color_base.g * 255.0) as u32) << 8
            | ((color_base.b * 255.0) as u32);

        // Store the player ID
        let id = Uuid::new_v4();
        self.id_to_addr.insert(id, addr);
        self.addr_to_id.insert(addr, id);

        let initial_position = Position { x, y };
        let mut position_history = Vec::with_capacity(MAX_POSITION_HISTORY);
        position_history.push(PositionSnapshot {
            position: initial_position,
            timestamp: Instant::now().elapsed().as_millis() as u64,
        });

        self.players.insert(
            addr,
            PlayerState {
                position: initial_position,
                color,
                last_active: Instant::now(),
                active: true,
                position_history,
            },
        );
        id
    }

    /// Handle player input and update position + activity
    pub fn handle_input(&mut self, addr: SocketAddr, input: PlayerInput) {
        if let Some(player) = self.players.get_mut(&addr) {
            player.last_active = Instant::now();
            player.active = true;

            // Update last processed input
            if let Some(id) = self.addr_to_id.get(&addr) {
                self.last_processed.insert(*id, input.sequence);
            }

            match input.dir {
                Direction::Up => player.position.y = player.position.y.saturating_sub(PLAYER_SPEED),
                Direction::Down => player.position.y = player.position.y.saturating_add(PLAYER_SPEED),
                Direction::Left => player.position.x = player.position.x.saturating_sub(PLAYER_SPEED),
                Direction::Right => player.position.x = player.position.x.saturating_add(PLAYER_SPEED),
            }

            // Store current position in history
            let current_time = Instant::now().elapsed().as_millis() as u64;
            player.position_history.push(PositionSnapshot {
                position: player.position,
                timestamp: current_time,
            });

            // Keep only the last MAX_POSITION_HISTORY entries
            if player.position_history.len() > MAX_POSITION_HISTORY {
                player.position_history.remove(0);
            }
        }
    }

    /// Get player position at a specific timestamp using interpolation
    pub fn get_player_position_at_time(&self, player_id: &Uuid, timestamp: u64) -> Option<Position> {
        let addr = self.id_to_addr.get(player_id)?;
        let player = self.players.get(addr)?;
        
        if player.position_history.is_empty() {
            return None;
        }

        // Find the two closest snapshots
        let mut before = None;
        let mut after = None;

        for snapshot in &player.position_history {
            if snapshot.timestamp <= timestamp {
                before = Some(snapshot);
            } else {
                after = Some(snapshot);
                break;
            }
        }

        match (before, after) {
            (Some(before), Some(after)) => {
                // Interpolate between the two snapshots
                let t = (timestamp - before.timestamp) as f32 / (after.timestamp - before.timestamp) as f32;
                Some(Position {
                    x: (before.position.x as f32 + (after.position.x - before.position.x) as f32 * t) as i32,
                    y: (before.position.y as f32 + (after.position.y - before.position.y) as f32 * t) as i32,
                })
            }
            (Some(before), None) => Some(before.position),
            (None, Some(after)) => Some(after.position),
            (None, None) => None,
        }
    }

    /// Check for collisions at a specific timestamp
    pub fn check_collisions_at_time(&self, timestamp: u64) -> Vec<(Uuid, Uuid)> {
        let mut collisions = Vec::new();
        let player_ids: Vec<Uuid> = self.id_to_addr.keys().cloned().collect();

        for i in 0..player_ids.len() {
            for j in (i + 1)..player_ids.len() {
                let id1 = &player_ids[i];
                let id2 = &player_ids[j];

                if let (Some(pos1), Some(pos2)) = (
                    self.get_player_position_at_time(id1, timestamp),
                    self.get_player_position_at_time(id2, timestamp),
                ) {
                    // Simple collision detection (players are considered colliding if they're at the same position)
                    if pos1.x == pos2.x && pos1.y == pos2.y {
                        collisions.push((*id1, *id2));
                    }
                }
            }
        }

        collisions
    }

    /// Marks players inactive if timeout exceeded
    pub fn update_inactive(&mut self) {
        let now = Instant::now();
        for player in self.players.values_mut() {
            if now.duration_since(player.last_active) >= TIMEOUT {
                player.active = false;
            }
        }
    }
    pub fn active_player_addrs(&self) -> Vec<SocketAddr> {
        self.players.iter()
            .filter(|(_, p)| p.active)
            .map(|(addr, _)| *addr)
            .collect()
    }

    /// Remove player on disconnect
    pub fn disconnect_player(&mut self, addr: &SocketAddr) {
        self.players.remove(addr);
    }

    /// Build a snapshot of active players for broadcasting
    pub fn build_snapshot(&self) -> GameState {
        let players = self.players.iter()
            .map(|(addr, p)| {
                let player_id = *self.addr_to_id.get(addr).unwrap();
                (player_id, p.position, p.color, p.active)
            })
            .collect();
        GameState {
            players,
            last_processed: self.last_processed.clone(),
            server_timestamp: Instant::now().elapsed().as_millis() as u64,
        }
    }
    pub fn players(&self) -> &HashMap<SocketAddr, PlayerState> {
        &self.players
    }

    /// Mutable access to players (use only when necessary)
    pub fn players_mut(&mut self) -> &mut HashMap<SocketAddr, PlayerState> {
        &mut self.players
    }
}
