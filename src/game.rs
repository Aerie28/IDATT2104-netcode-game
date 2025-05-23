use uuid::Uuid;
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::Instant,
};
use crate::colors::player_colors;
use crate::types::{Position, PlayerInput, Direction, GameState};
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT, PLAYER_SPEED, TIMEOUT, ID_GRACE_PERIOD};

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
    pub position_history: Vec<PositionSnapshot>,
}

/// Stores information about disconnected players
#[derive(Debug)]
pub struct DisconnectedPlayer {
    pub position: Position,
    pub color: u32,
    pub disconnected_at: Instant,
}

pub struct Game {
    players: HashMap<SocketAddr, PlayerState>,
    id_to_addr: HashMap<Uuid, SocketAddr>,
    addr_to_id: HashMap<SocketAddr, Uuid>,
    last_processed: HashMap<Uuid, u32>, // Track inputs
    disconnected_players: HashMap<Uuid, DisconnectedPlayer>, // Track disconnected players
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            id_to_addr: HashMap::new(),
            addr_to_id: HashMap::new(),
            last_processed: HashMap::new(),
            disconnected_players: HashMap::new(),
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

    

    /// Marks players inactive if timeout exceeded
    pub fn update_server_dropped(&mut self) {
        let now = Instant::now();
        let mut to_disconnect = Vec::new();
        
        // Check for players that haven't sent a ping in TIMEOUT duration
        for (addr, player) in self.players.iter() {
            if now.duration_since(player.last_active) >= TIMEOUT {
                if let Some(id) = self.addr_to_id.get(addr) {
                    // Check if player is already in disconnected_players
                    if !self.disconnected_players.contains_key(id) {
                        to_disconnect.push(*addr);
                    }
                }
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

    /// Clean up expired disconnected players
    pub fn cleanup_disconnected(&mut self) {
        self.cleanup_disconnected_with_time(Instant::now());
    }

    /// Clean up expired disconnected players with explicit time
    pub fn cleanup_disconnected_with_time(&mut self, now: Instant) {
        println!("Running cleanup of disconnected players at {:?}", now);
        let mut expired = Vec::new();
        
        // Find expired players
        for (id, player) in self.disconnected_players.iter() {
            let duration = now.duration_since(player.disconnected_at);
            println!("Checking player {}: disconnected for {:?}", id, duration);
            if duration >= ID_GRACE_PERIOD {
                // Double check that the player is not in active players
                if !self.id_to_addr.contains_key(id) {
                    expired.push(*id);
                    println!("Player {} grace period expired at {:?}", id, duration);
                } else {
                    println!("Player {} is still active, skipping cleanup", id);
                }
            } else {
                println!("Player {} still within grace period: {:?} remaining", id, ID_GRACE_PERIOD - duration);
            }
        }
        
        // Remove expired players
        for id in expired {
            self.disconnected_players.remove(&id);
            println!("Removed expired disconnected player {}", id);
        }
        println!("Cleanup complete. Remaining disconnected players: {:?}", self.disconnected_players.keys().collect::<Vec<_>>());
    }

    /// Remove player on disconnect
    pub fn disconnect_player(&mut self, addr: &SocketAddr) {
        println!("Attempting to disconnect player at address {}", addr);
        if let Some(id) = self.addr_to_id.remove(addr) {
            println!("Found player ID {} for address {}", id, addr);
            if let Some(player) = self.players.get(addr) {
                let now = Instant::now();
                // Store player info for grace period
                self.disconnected_players.insert(id, DisconnectedPlayer {
                    position: player.position,
                    color: player.color,
                    disconnected_at: now,
                });
                println!("Stored disconnected player {} with position {:?} at {:?}", id, player.position, now);
                println!("Current disconnected players after storing: {:?}", self.disconnected_players.keys().collect::<Vec<_>>());
            } else {
                println!("No player state found for address {}", addr);
            }
            self.id_to_addr.remove(&id);
            self.last_processed.remove(&id);
            println!("Removed player {} from active mappings", id);
        } else {
            println!("No player ID found for address {}", addr);
        }
        self.players.remove(addr);
        println!("Removed player from active players map");
    }

    /// Reconnect a player with their previous ID and position
    pub fn reconnect_player(&mut self, addr: SocketAddr, id: Uuid, position: Position) {
        let now = Instant::now();
        println!("Attempting to reconnect player {} from address {}", id, addr);
        println!("Current disconnected players: {:?}", self.disconnected_players.keys().collect::<Vec<_>>());
        
        // Check if player was recently disconnected
        if let Some(disconnected) = self.disconnected_players.remove(&id) {
            let disconnect_duration = now.duration_since(disconnected.disconnected_at);
            println!("Found disconnected player {} after {:?}", id, disconnect_duration);
            println!("Disconnected player position: {:?}", disconnected.position);
            println!("Disconnected player color: {}", disconnected.color);
            
            // Update address mappings
            self.id_to_addr.insert(id, addr);
            self.addr_to_id.insert(addr, id);
            println!("Updated address mappings for player {}", id);

            // Create position history with the reconnected position
            let mut position_history = Vec::with_capacity(MAX_POSITION_HISTORY);
            position_history.push(PositionSnapshot {
                position,
                timestamp: now.elapsed().as_millis() as u64,
            });

            // Recreate player state
            self.players.insert(
                addr,
                PlayerState {
                    position,
                    color: disconnected.color,
                    last_active: now,
                    position_history,
                },
            );
            println!("Successfully reconnected player {} at position {:?}", id, position);
        } else {
            println!("Failed to find disconnected player {} in disconnected_players map", id);
            println!("Current disconnected players: {:?}", self.disconnected_players.keys().collect::<Vec<_>>());
        }
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

    pub fn players(&self) -> &HashMap<SocketAddr, PlayerState> {
        &self.players
    }

    /// Mutable access to players (use only when necessary)
    pub fn get_players_mut(&mut self) -> &mut HashMap<SocketAddr, PlayerState> {
        &mut self.players
    }

    /// Get a reference to the address to ID mapping
    pub fn get_addr_to_id(&self) -> &HashMap<SocketAddr, Uuid> {
        &self.addr_to_id
    }
    
    /// Get a reference to the ID to address mapping
     pub fn get_id_to_addr(&self) -> &HashMap<Uuid, SocketAddr> {
        &self.id_to_addr
    }
    pub fn get_disconnected_players(&self) -> &HashMap<Uuid, DisconnectedPlayer> {
        &self.disconnected_players
    }
    pub fn get_disconnected_at(&self, id: &Uuid) -> Option<&Instant> {
        self.disconnected_players.get(id).map(|p| &p.disconnected_at)
    }

    pub fn get_disconnected_player(&self, id: &Uuid) -> Option<&DisconnectedPlayer> {
        self.disconnected_players.get(id)
    }
}