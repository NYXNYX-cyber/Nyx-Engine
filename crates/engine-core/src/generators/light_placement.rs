use crate::components::flame::{Checkpoint, CheckpointType};
use super::level_2d::{TileMap2D, TileType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalBrazier {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub light_radius: f32,
    pub is_lit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmberShardPickup {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: u32,
    pub is_collected: bool,
}

pub struct LightAndPickupGenerator;

impl LightAndPickupGenerator {
    /// Places check points, unlit/lit braziers, and collectible ember shards across the level.
    pub fn populate(
        map: &TileMap2D,
        seed: u64,
    ) -> (Vec<Checkpoint>, Vec<EnvironmentalBrazier>, Vec<EmberShardPickup>) {
        let mut checkpoints = Vec::new();
        let mut braziers = Vec::new();
        let mut pickups = Vec::new();

        let ts = map.tile_size;
        let mut cp_id = 1u32;
        let mut br_id = 1u32;
        let mut pk_id = 1u32;

        let interval = (map.width / 4).max(20);

        // 1. Place Checkpoints at paced intervals (Store vs Recharge vs Dual)
        for i in 1..4 {
            let target_x = i * interval;
            for y in 1..map.height - 2 {
                if map.get_tile(target_x, y) == TileType::Air
                    && map.get_tile(target_x, y - 1) == TileType::SolidGround
                {
                    let cp_type = match i {
                        1 => CheckpointType::Recharge,
                        2 => CheckpointType::Store,
                        _ => CheckpointType::Dual,
                    };

                    let world_x = target_x as f32 * ts + ts * 0.5;
                    let world_y = y as f32 * ts;

                    checkpoints.push(Checkpoint::new(cp_id, cp_type, world_x, world_y, 48.0));
                    cp_id += 1;
                    break;
                }
            }
        }

        // 2. Place Braziers that create warm safe zones
        for x in (15..map.width.saturating_sub(15)).step_by(18) {
            for y in 1..map.height - 2 {
                if map.get_tile(x, y) == TileType::Air && map.get_tile(x, y - 1) == TileType::SolidGround {
                    let world_x = x as f32 * ts + ts * 0.5;
                    let world_y = y as f32 * ts + ts * 0.5;

                    braziers.push(EnvironmentalBrazier {
                        id: br_id,
                        x: world_x,
                        y: world_y,
                        light_radius: 120.0,
                        is_lit: (br_id + seed as u32) % 2 == 0,
                    });
                    br_id += 1;
                    break;
                }
            }
        }

        // 3. Place Ember Shard pickups in elevated platforms
        for x in (8..map.width.saturating_sub(8)).step_by(10) {
            for y in 4..map.height - 2 {
                if map.get_tile(x, y) == TileType::OneWayPlatform {
                    let world_x = x as f32 * ts + ts * 0.5;
                    let world_y = (y + 1) as f32 * ts + ts * 0.5;

                    pickups.push(EmberShardPickup {
                        id: pk_id,
                        x: world_x,
                        y: world_y,
                        amount: 10 + (pk_id * 5) % 30,
                        is_collected: false,
                    });
                    pk_id += 1;
                    break;
                }
            }
        }

        (checkpoints, braziers, pickups)
    }
}
