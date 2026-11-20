use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chapter {
    /// Chapter I: Ancient dark woods, tutorial platforms, canopy branches.
    SilentForest,
    /// Chapter II: Abandoned civilization, stone rooftops, broken houses.
    ForgottenVillage,
    /// Chapter III: Subterranean caverns, stalactites, vertical drop pits.
    BeneathTheEarth,
    /// Chapter IV: Windswept ruins, crumbling sky bridges, fragmented voids.
    LastHorizon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Air,
    SolidGround,
    OneWayPlatform,
    HazardSpike,
    DarknessVoid,
    ClimbableWall,
}

/// 2D TileMap grid representing an entire platforming level chunk or sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMap2D {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    pub tiles: Vec<TileType>,
}

impl TileMap2D {
    pub fn new(width: usize, height: usize, tile_size: f32, default_tile: TileType) -> Self {
        Self {
            width,
            height,
            tile_size,
            tiles: vec![default_tile; width * height],
        }
    }

    #[inline]
    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x]
        } else {
            TileType::Air
        }
    }

    #[inline]
    pub fn set_tile(&mut self, x: usize, y: usize, tile: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    /// World coordinates (x, y) to Tile coordinates.
    pub fn world_to_tile(&self, wx: f32, wy: f32) -> (usize, usize) {
        let tx = (wx / self.tile_size).max(0.0) as usize;
        let ty = (wy / self.tile_size).max(0.0) as usize;
        (tx.min(self.width.saturating_sub(1)), ty.min(self.height.saturating_sub(1)))
    }
}

/// Procedural level generator tailored for The Last Flame's chapter progression.
pub struct PlatformerLevelGenerator {
    pub chapter: Chapter,
    pub seed: u64,
}

impl PlatformerLevelGenerator {
    pub fn new(chapter: Chapter, seed: u64) -> Self {
        Self { chapter, seed }
    }

    /// Generates a complete traversable 2D platforming stage based on chapter theme.
    pub fn generate(&self, width: usize, height: usize, tile_size: f32) -> TileMap2D {
        let mut map = TileMap2D::new(width, height, tile_size, TileType::Air);

        match self.chapter {
            Chapter::SilentForest => self.generate_silent_forest(&mut map),
            Chapter::ForgottenVillage => self.generate_forgotten_village(&mut map),
            Chapter::BeneathTheEarth => self.generate_beneath_earth(&mut map),
            Chapter::LastHorizon => self.generate_last_horizon(&mut map),
        }

        map
    }

    fn generate_silent_forest(&self, map: &mut TileMap2D) {
        let h = map.height;
        let w = map.width;

        // Base ground with occasional pit traps
        for x in 0..w {
            let pit = (x > 18 && x < 24) || (x > 50 && x < 57) || (x > 85 && x < 92);
            for y in 0..4 {
                if pit {
                    if y == 0 {
                        map.set_tile(x, y, TileType::DarknessVoid);
                    }
                } else {
                    map.set_tile(x, y, TileType::SolidGround);
                }
            }
        }

        // Branching tree canopy platforms
        for x in 8..w - 8 {
            let n = ((x as f64 * 0.25 + self.seed as f64 * 0.1).sin() * 3.0) as i32;
            let platform_y = (6 + n).clamp(4, (h - 3) as i32) as usize;
            if x % 6 >= 2 {
                map.set_tile(x, platform_y, TileType::OneWayPlatform);
            }
            if x % 16 == 0 && platform_y + 4 < h {
                map.set_tile(x, platform_y + 4, TileType::OneWayPlatform);
                map.set_tile(x + 1, platform_y + 4, TileType::OneWayPlatform);
            }
        }
    }

    fn generate_forgotten_village(&self, map: &mut TileMap2D) {
        let _h = map.height;
        let w = map.width;

        // Cobblestone ground
        for x in 0..w {
            for y in 0..3 {
                map.set_tile(x, y, TileType::SolidGround);
            }
        }

        // Ruined stone houses and staircases
        let mut house_x = 12;
        while house_x < w - 15 {
            let house_w = 8;
            let house_h = 5;

            // Walls
            for y in 3..3 + house_h {
                map.set_tile(house_x, y, TileType::ClimbableWall);
                map.set_tile(house_x + house_w, y, TileType::ClimbableWall);
            }

            // Rooftops
            for x in house_x..=house_x + house_w {
                map.set_tile(x, 3 + house_h, TileType::SolidGround);
            }

            // Interior floors
            for x in house_x + 1..house_x + house_w {
                map.set_tile(x, 5, TileType::OneWayPlatform);
            }

            house_x += house_w + 6;
        }
    }

    fn generate_beneath_earth(&self, map: &mut TileMap2D) {
        let h = map.height;
        let w = map.width;

        // Solid cavern ceiling & floor
        for x in 0..w {
            map.set_tile(x, 0, TileType::SolidGround);
            map.set_tile(x, 1, TileType::SolidGround);
            map.set_tile(x, h - 1, TileType::SolidGround);
            map.set_tile(x, h - 2, TileType::SolidGround);

            // Stalactites (hazard spikes on ceiling)
            if (x * 7 + self.seed as usize) % 9 == 0 {
                map.set_tile(x, h - 3, TileType::HazardSpike);
            }

            // Floor spikes in deep crevices
            if (x > 25 && x < 32) || (x > 65 && x < 72) {
                map.set_tile(x, 1, TileType::HazardSpike);
            }
        }

        // Meandering cave shelves
        for x in 5..w - 5 {
            let shelf_y = (4.0 + ((x as f64 * 0.18).sin() * 2.5) + ((x as f64 * 0.05).cos() * 2.0)) as usize;
            if shelf_y < h - 4 {
                map.set_tile(x, shelf_y, TileType::SolidGround);
            }
        }
    }

    fn generate_last_horizon(&self, map: &mut TileMap2D) {
        let h = map.height;
        let w = map.width;

        // Bottom is 100% bottomless darkness void
        for x in 0..w {
            map.set_tile(x, 0, TileType::DarknessVoid);
        }

        // Floating shattered bridge segments
        let mut x = 4;
        let mut curr_y = 5;
        while x < w - 6 {
            let seg_len = (4 + (x % 5)) as usize;
            for seg in 0..seg_len {
                if x + seg < w {
                    map.set_tile(x + seg, curr_y, TileType::SolidGround);
                }
            }
            x += seg_len + 3 + (x % 3);
            let y_delta = ((x as i32 % 3) - 1).clamp(-1, 1);
            curr_y = (curr_y as i32 + y_delta).clamp(4, (h - 4) as i32) as usize;
        }
    }
}
