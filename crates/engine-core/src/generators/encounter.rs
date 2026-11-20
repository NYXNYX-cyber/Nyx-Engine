use crate::components::enemy::{EnemyArchetype, EnemyEntity};
use super::level_2d::{Chapter, TileMap2D, TileType};

pub struct EncounterGenerator;

impl EncounterGenerator {
    /// Generates appropriate enemy spawns across the generated tilemap.
    pub fn populate_enemies(map: &TileMap2D, chapter: Chapter, seed: u64) -> Vec<EnemyEntity> {
        let mut enemies = Vec::new();
        let mut enemy_id = 1u32;
        let ts = map.tile_size;

        for x in 10..map.width.saturating_sub(10) {
            // Find ground surface tile
            for y in 1..map.height.saturating_sub(4) {
                let current_tile = map.get_tile(x, y);
                let tile_below = map.get_tile(x, y - 1);

                // Spawn on top of solid floor
                if current_tile == TileType::Air && (tile_below == TileType::SolidGround || tile_below == TileType::OneWayPlatform) {
                    let hash = (x as u64 * 73856093) ^ (y as u64 * 19349663) ^ seed;
                    let spawn_roll = hash % 100;

                    let should_spawn = match chapter {
                        Chapter::SilentForest => spawn_roll < 12,
                        Chapter::ForgottenVillage => spawn_roll < 16,
                        Chapter::BeneathTheEarth => spawn_roll < 20,
                        Chapter::LastHorizon => spawn_roll < 24,
                    };

                    if should_spawn {
                        let archetype_roll = (hash >> 8) % 100;
                        let archetype = match chapter {
                            Chapter::SilentForest => {
                                if archetype_roll < 75 {
                                    EnemyArchetype::Crawler
                                } else {
                                    EnemyArchetype::Wisp
                                }
                            }
                            Chapter::ForgottenVillage => {
                                if archetype_roll < 45 {
                                    EnemyArchetype::Crawler
                                } else if archetype_roll < 75 {
                                    EnemyArchetype::Wisp
                                } else {
                                    EnemyArchetype::Hollow
                                }
                            }
                            Chapter::BeneathTheEarth => {
                                if archetype_roll < 35 {
                                    EnemyArchetype::Hollow
                                } else if archetype_roll < 65 {
                                    EnemyArchetype::Ashwing
                                } else if archetype_roll < 85 {
                                    EnemyArchetype::Crawler
                                } else {
                                    EnemyArchetype::Devourer
                                }
                            }
                            Chapter::LastHorizon => {
                                if archetype_roll < 35 {
                                    EnemyArchetype::Ashwing
                                } else if archetype_roll < 70 {
                                    EnemyArchetype::Hollow
                                } else {
                                    EnemyArchetype::Devourer
                                }
                            }
                        };

                        let world_x = x as f32 * ts + ts * 0.5;
                        let world_y = y as f32 * ts + ts * 0.5;

                        enemies.push(EnemyEntity::new(enemy_id, archetype, world_x, world_y));
                        enemy_id += 1;
                    }
                }
            }
        }

        enemies
    }
}
