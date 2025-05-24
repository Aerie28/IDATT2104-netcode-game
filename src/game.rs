use crate::colors::player_colors;
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT, PLAYER_SPEED, TIMEOUT, PLAYER_SIZE, TOOL_BAR_HEIGHT};
use crate::types::{Position, PlayerInput, Direction, GameState, PositionSnapshot};

use std::{collections::HashMap, net::SocketAddr, time::Instant};
use uuid::Uuid;

const MAX_POSITION_HISTORY: usize = 60; // Store 1 second of history at 60fps


/// Stores state for one player
pub struct PlayerState {
    pub position: Position,
    pub color: u32,
    pub last_active: Instant,
    pub position_history: Vec<PositionSnapshot>,
}

/// Game state that tracks all players and their positions, and ids for the players
pub struct Game {
    players: HashMap<SocketAddr, PlayerState>,
    id_to_addr: HashMap<Uuid, SocketAddr>,
    addr_to_id: HashMap<SocketAddr, Uuid>,
    last_processed: HashMap<Uuid, u32>, // Track inputs
}

/// Implementation of the Game state
impl Game {
    /// Creates a new Game instance
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

        // Check if player already connected
        if self.players.contains_key(&addr) {
            // Player already connected
            return *self.addr_to_id.get(&addr).unwrap();
        }

        // Generate a random position within the board bounds
        let mut rng = rand::rng();
        let x = rng.random_range(PLAYER_SIZE..(BOARD_WIDTH - (PLAYER_SIZE)));
        let y = rng.random_range(PLAYER_SIZE..(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT));
        
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

        // Initialize player position and history
        let initial_position = Position { x, y };
        let mut position_history = Vec::with_capacity(MAX_POSITION_HISTORY);
        position_history.push(PositionSnapshot {
            position: initial_position,
            timestamp: Instant::now().elapsed().as_millis() as u64,
        });

        // Insert the player state into the game
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

            // Update player position based on input direction for prediction
            match input.dir {
                Direction::Up => player.position.y = player.position.y.saturating_sub(PLAYER_SPEED).max(PLAYER_SIZE),
                Direction::Down => player.position.y = player.position.y.saturating_add(PLAYER_SPEED).min(BOARD_HEIGHT - (PLAYER_SIZE) - TOOL_BAR_HEIGHT),
                Direction::Left => player.position.x = player.position.x.saturating_sub(PLAYER_SPEED).max(PLAYER_SIZE),
                Direction::Right => player.position.x = player.position.x.saturating_add(PLAYER_SPEED).min(BOARD_WIDTH - (PLAYER_SIZE)),
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

    /// Get player address of active player
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

/// Unit tests for the Game state
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    // Helper function to create test socket addresses
    fn test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }

    #[test]
    fn test_new_game() {
        let game = Game::new();
        assert!(game.players.is_empty());
        assert!(game.id_to_addr.is_empty());
        assert!(game.addr_to_id.is_empty());
        assert!(game.last_processed.is_empty());
    }

    #[test]
    fn test_connect_player() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        let id = game.connect_player(addr);

        // Check player was added
        assert_eq!(game.players.len(), 1);
        assert!(game.players.contains_key(&addr));

        // Check mappings were created
        assert_eq!(game.id_to_addr.len(), 1);
        assert_eq!(game.addr_to_id.len(), 1);
        assert_eq!(game.id_to_addr.get(&id), Some(&addr));
        assert_eq!(game.addr_to_id.get(&addr), Some(&id));

        // Check position history initialization
        let player = game.players.get(&addr).unwrap();
        assert_eq!(player.position_history.len(), 1);

        // Position should be within bounds
        assert!(player.position.x >= PLAYER_SIZE);
        assert!(player.position.x <= BOARD_WIDTH - PLAYER_SIZE);
        assert!(player.position.y >= PLAYER_SIZE);
        assert!(player.position.y <= BOARD_HEIGHT - PLAYER_SIZE - TOOL_BAR_HEIGHT);
    }

    #[test]
    fn test_reconnect_existing_player() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        let id1 = game.connect_player(addr);
        let id2 = game.connect_player(addr);  // Reconnect same address

        // Should return same ID and not create new player
        assert_eq!(id1, id2);
        assert_eq!(game.players.len(), 1);
    }

    #[test]
    fn test_disconnect_player() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        game.connect_player(addr);
        game.disconnect_player(&addr);

        // Player should be removed
        assert!(game.players.is_empty());
        assert!(game.id_to_addr.is_empty());
        assert!(game.addr_to_id.is_empty());
    }

    #[test]
    fn test_handle_input() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        let id = game.connect_player(addr);
        let initial_pos = game.players.get(&addr).unwrap().position;

        // Test movement and input tracking
        game.handle_input(addr, PlayerInput { dir: Direction::Right, sequence: 1, timestamp: 0 });

        // Position should change according to direction
        let player = game.players.get(&addr).unwrap();
        assert_eq!(player.position.x, initial_pos.x + PLAYER_SPEED);
        assert_eq!(player.position.y, initial_pos.y);

        // Sequence should be updated
        assert_eq!(game.last_processed.get(&id), Some(&1));

        // Position history should be updated
        assert_eq!(player.position_history.len(), 2);
    }

    #[test]
    fn test_position_history_limit() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        game.connect_player(addr);

        // Add more positions than the history limit
        for i in 0..MAX_POSITION_HISTORY + 10 {
            game.handle_input(addr, PlayerInput { dir: Direction::Right, sequence: i as u32, timestamp: 0 });
        }

        // History length should be capped
        assert_eq!(game.players.get(&addr).unwrap().position_history.len(), MAX_POSITION_HISTORY);
    }

    #[test]
    fn test_active_player_addrs() {
        let mut game = Game::new();
        let addr1 = test_addr(8080);
        let addr2 = test_addr(8081);

        game.connect_player(addr1);
        game.connect_player(addr2);

        let addrs = game.active_player_addrs();
        assert_eq!(addrs.len(), 2);
        assert!(addrs.contains(&addr1));
        assert!(addrs.contains(&addr2));
    }

    #[test]
    fn test_build_snapshot() {
        let mut game = Game::new();
        let addr1 = test_addr(8080);
        let addr2 = test_addr(8081);

        let id1 = game.connect_player(addr1);
        let _id2 = game.connect_player(addr2);

        game.handle_input(addr1, PlayerInput { dir: Direction::Up, sequence: 5, timestamp: 0 });

        let snapshot = game.build_snapshot();

        // Should contain two players
        assert_eq!(snapshot.players.len(), 2);

        // Should track processed inputs
        assert_eq!(snapshot.last_processed.get(&id1), Some(&5));

        // No need to check timestamp >= 0 as u64 is always >= 0
        assert!(true);
    }

    #[test]
    fn test_movement_boundaries() {
        let mut game = Game::new();
        let addr = test_addr(8080);
        game.connect_player(addr);

        // Test minimum X boundary
        {
            let player = game.players.get_mut(&addr).unwrap();
            player.position.x = PLAYER_SIZE;
        }  // Release borrow with scope

        game.handle_input(addr, PlayerInput { dir: Direction::Left, sequence: 1, timestamp: 0 });
        assert_eq!(game.players.get(&addr).unwrap().position.x, PLAYER_SIZE); // Shouldn't move past boundary

        // Test maximum X boundary
        {
            let player = game.players.get_mut(&addr).unwrap();
            player.position.x = BOARD_WIDTH - PLAYER_SIZE;
        }

        game.handle_input(addr, PlayerInput { dir: Direction::Right, sequence: 2, timestamp: 0 });
        assert_eq!(game.players.get(&addr).unwrap().position.x, BOARD_WIDTH - PLAYER_SIZE);

        // Test minimum Y boundary
        {
            let player = game.players.get_mut(&addr).unwrap();
            player.position.y = PLAYER_SIZE;
        }

        game.handle_input(addr, PlayerInput { dir: Direction::Up, sequence: 3, timestamp: 0 });
        assert_eq!(game.players.get(&addr).unwrap().position.y, PLAYER_SIZE);

        // Test maximum Y boundary
        {
            let player = game.players.get_mut(&addr).unwrap();
            player.position.y = BOARD_HEIGHT - PLAYER_SIZE - TOOL_BAR_HEIGHT;
        }

        game.handle_input(addr, PlayerInput { dir: Direction::Down, sequence: 4, timestamp: 0 });
        assert_eq!(game.players.get(&addr).unwrap().position.y, BOARD_HEIGHT - PLAYER_SIZE - TOOL_BAR_HEIGHT);
    }

    #[test]
    fn test_update_server_dropped() {
        let mut game = Game::new();
        let addr = test_addr(8080);

        game.connect_player(addr);

        // Manually set last_active to be longer than timeout
        {
            let player = game.players.get_mut(&addr).unwrap();
            player.last_active = Instant::now() - TIMEOUT - Duration::from_secs(1);
        }

        // Check player exists before timeout
        assert_eq!(game.players.len(), 1);

        // Run timeout check
        game.update_server_dropped();

        // Player should be removed after timeout
        assert!(game.players.is_empty());
        assert!(game.id_to_addr.is_empty());
        assert!(game.addr_to_id.is_empty());
    }
}