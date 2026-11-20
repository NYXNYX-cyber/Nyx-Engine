pub mod components;
pub mod generators;
pub mod physics;
pub mod voxel;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub color: String,
}

impl Player {
    pub fn new(id: u32, name: String, color: String) -> Self {
        Self {
            id,
            name,
            x: 400.0,
            y: 300.0,
            vx: 0.0,
            vy: 0.0,
            size: 24.0,
            color,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub tick: u64,
    pub width: f32,
    pub height: f32,
    pub players: HashMap<u32, Player>,
}

impl WorldState {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            tick: 0,
            width,
            height,
            players: HashMap::new(),
        }
    }

    pub fn step(&mut self, inputs: &HashMap<u32, PlayerInput>, dt: f32) {
        self.tick += 1;
        let speed = 280.0 * dt; // Movement speed in pixels per second

        for (id, player) in self.players.iter_mut() {
            if let Some(input) = inputs.get(id) {
                let mut dx: f32 = 0.0;
                let mut dy: f32 = 0.0;
                if input.left { dx -= 1.0; }
                if input.right { dx += 1.0; }
                if input.up { dy -= 1.0; }
                if input.down { dy += 1.0; }

                // Normalize diagonal movement
                if dx != 0.0 && dy != 0.0 {
                    let len = (dx * dx + dy * dy).sqrt();
                    dx /= len;
                    dy /= len;
                }

                player.vx = dx * speed;
                player.vy = dy * speed;
            } else {
                player.vx = 0.0;
                player.vy = 0.0;
            }

            // Apply movement & world boundary collision
            player.x = (player.x + player.vx).clamp(player.size, self.width - player.size);
            player.y = (player.y + player.vy).clamp(player.size, self.height - player.size);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Join { name: String, color: String },
    Input(PlayerInput),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { your_id: u32, tick_rate: u32 },
    Sync(WorldState),
    PlayerLeft { id: u32 },
}
