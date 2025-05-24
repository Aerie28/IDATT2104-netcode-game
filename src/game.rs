use uuid::Uuid;
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::Instant,
};
use crate::colors::player_colors;
use crate::types::{Position, PlayerInput, Direction, GameState, PositionSnapshot};
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT, PLAYER_SPEED, TIMEOUT, PLAYER_SIZE, TOOL_BAR_HEIGHT};

const MAX_POSITION_HISTORY: usize = 60; // Store 1 second of history at 60fps


/// Stores state for one player
pub struct PlayerState {
    pub position: Position,
    pub color: u32,
    pub last_active: Instant,
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
        let x = rng.random_range((PLAYER_SIZE)..(BOARD_WIDTH - (PLAYER_SIZE)));
        let y = rng.random_range((PLAYER_SIZE)..(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT));
        
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
                position_history,
            },
        );
        id
    }

    /// Handle player input and update position + activity
    pub fn handle_input(&mut self, addr: SocketAddr, input: PlayerInput) {
        if let Some(player) = self.players.get_mut(&addr) {
            player.last_active = Instant::now();

            // Update last processed input
            if let Some(id) = self.addr_to_id.get(&addr) {
                self.last_processed.insert(*id, input.sequence);
            }

            match input.dir {
                Direction::Up => player.position.y = (player.position.y.saturating_sub(PLAYER_SPEED)).max(PLAYER_SIZE),
                Direction::Down => player.position.y = (player.position.y.saturating_add(PLAYER_SPEED)).min(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT),
                Direction::Left => player.position.x = (player.position.x.saturating_sub(PLAYER_SPEED)).max(PLAYER_SIZE),
                Direction::Right => player.position.x = (player.position.x.saturating_add(PLAYER_SPEED)).min(BOARD_WIDTH - (PLAYER_SIZE)),
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

    /// Marks players inactive if timeout exceeded
    pub fn update_server_dropped(&mut self) {
        let now = Instant::now();
        let mut to_disconnect = Vec::new();
        
        // Check for players that haven't sent a ping in TIMEOUT duration
        for (addr, player) in self.players.iter() {
            if now.duration_since(player.last_active) >= TIMEOUT {
                to_disconnect.push(*addr);
            }
        }
        
        // Disconnect inactive players
        for addr in to_disconnect {
            if let Some(id) = self.addr_to_id.get(&addr) {
                println!("Player {} disconnected due to timeout", id);
            }
            self.disconnect_player(&addr);
        }
    }

    pub fn active_player_addrs(&self) -> Vec<SocketAddr> {
        self.players.keys().cloned().collect()
    }

    /// Remove player on disconnect
    pub fn disconnect_player(&mut self, addr: &SocketAddr) {
        if let Some(id) = self.addr_to_id.remove(addr) {
            self.id_to_addr.remove(&id);
            self.last_processed.remove(&id);
        }
        self.players.remove(addr);
    }

    /// Build a snapshot of active players for broadcasting
    pub fn build_snapshot(&self) -> GameState {
        let players = self.players.iter()
            .map(|(addr, p)| {
                let player_id = *self.addr_to_id.get(addr).unwrap();
                (player_id, p.position, p.color)
            })
            .collect();
        GameState {
            players,
            last_processed: self.last_processed.clone(),
            server_timestamp: Instant::now().elapsed().as_millis() as u64,
        }
    }

    /// Mutable access to players (use only when necessary)
    pub fn get_players_mut(&mut self) -> &mut HashMap<SocketAddr, PlayerState> {
        &mut self.players
    }
}