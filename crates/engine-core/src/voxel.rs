use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Ore = 4,
    Water = 5,
    Wood = 6,
    Leaves = 7,
}

impl BlockType {
    pub fn is_opaque(&self) -> bool {
        matches!(self, BlockType::Grass | BlockType::Dirt | BlockType::Stone | BlockType::Ore | BlockType::Wood)
    }

    pub fn is_solid(&self) -> bool {
        matches!(self, BlockType::Grass | BlockType::Dirt | BlockType::Stone | BlockType::Ore | BlockType::Wood | BlockType::Leaves)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBlockInstance {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub block_type: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelChunkResult {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub size_x: usize,
    pub size_z: usize,
    pub max_height: usize,
    pub sea_level: usize,
    pub block_size: f32,
    pub total_blocks_computed: usize,
    pub culled_blocks_count: usize,
    pub visible_blocks_count: usize,
    pub generation_time_ms: f32,
    pub blocks: Vec<VoxelBlockInstance>,
    // Heightmap flattened: array of (x, z, top_y)
    pub heightmap: Vec<(usize, usize, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoxelWorldParams {
    pub size_x: usize,
    pub size_z: usize,
    pub max_height: usize,
    pub base_height: usize,
    pub sea_level: usize,
    pub block_size: f32,
    pub seed: u32,
    pub spawn_trees: bool,
    pub spawn_ores: bool,
    pub enable_culling: bool,
}

impl Default for VoxelWorldParams {
    fn default() -> Self {
        Self {
            size_x: 16,
            size_z: 16,
            max_height: 12,
            base_height: 3,
            sea_level: 3,
            block_size: 1.0,
            seed: 12345,
            spawn_trees: true,
            spawn_ores: true,
            enable_culling: true,
        }
    }
}

// Multi-octave Procedural Simplex-like Noise function
fn calculate_terrain_height(x: usize, z: usize, params: &VoxelWorldParams) -> usize {
    let seed_f = params.seed as f32;
    let nx = (x as f32 + seed_f) * 0.11;
    let nz = (z as f32 + seed_f) * 0.11;

    let mut n = (nx.sin() * nz.cos());
    n += ((nx * 2.1 + 1.4).sin() * (nz * 2.1 + 0.7).cos()) * 0.45;
    n += ((nx * 4.4 + 2.3).sin() * (nz * 4.4 + 3.1).cos()) * 0.22;

    let norm = (n + 1.0) * 0.5; // [0.0 .. 1.0]
    let span = (params.max_height.saturating_sub(params.base_height)) as f32;
    let raw_h = params.base_height as f32 + norm * span;

    raw_h.floor() as usize
}

pub struct VoxelGenerator;

impl VoxelGenerator {
    /// Generates a single chunk using Linux CPU with 3D grid voxel calculation and hidden face culling
    pub fn generate_chunk(chunk_x: i32, chunk_z: i32, params: &VoxelWorldParams) -> VoxelChunkResult {
        let start = Instant::now();

        let sx = params.size_x;
        let sz = params.size_z;
        let sy = params.max_height + 8; // Extra headroom for trees

        // 3D Voxel Grid: 0 = Air
        let mut grid = vec![BlockType::Air; sx * sy * sz];
        let idx = |x: usize, y: usize, z: usize| -> usize {
            x + (y * sx) + (z * sx * sy)
        };

        let mut heightmap = Vec::with_capacity(sx * sz);
        let mut tree_candidates: Vec<(usize, usize, usize)> = Vec::new();

        // 1. Terrain generation pass
        for x in 0..sx {
            for z in 0..sz {
                let h = calculate_terrain_height(x, z, params).clamp(1, params.max_height);
                heightmap.push((x, z, h as f32 * params.block_size));

                // Vertical column
                for y in 0..=h {
                    let b = if y == h {
                        if y >= params.sea_level {
                            BlockType::Grass
                        } else {
                            BlockType::Dirt
                        }
                    } else if y >= h.saturating_sub(2) {
                        BlockType::Dirt
                    } else {
                        // Deep underground
                        let ore_noise = ((x as f32 * 17.3 + y as f32 * 29.1 + z as f32 * 41.7).sin()).abs();
                        if params.spawn_ores && y < 4 && ore_noise > 0.88 {
                            BlockType::Ore
                        } else {
                            BlockType::Stone
                        }
                    };
                    grid[idx(x, y, z)] = b;
                }

                // Water fill in depressions
                if h < params.sea_level {
                    for y in (h + 1)..=params.sea_level {
                        grid[idx(x, y, z)] = BlockType::Water;
                    }
                }

                // Potential tree spot
                if params.spawn_trees && h >= params.sea_level + 1 && x > 2 && x < sx - 2 && z > 2 && z < sz - 2 {
                    let tree_noise = ((x as f32 * 12.5 + z as f32 * 33.1 + params.seed as f32).sin());
                    if tree_noise > 0.65 {
                        tree_candidates.push((x, h + 1, z));
                    }
                }
            }
        }

        // 2. Spawn Trees with strict Poisson spacing (never nyambung)
        let mut spawned_trees: Vec<(usize, usize)> = Vec::new();
        for (tx, ty, tz) in tree_candidates {
            if spawned_trees.len() >= 3 {
                break;
            }
            let too_close = spawned_trees.iter().any(|(ox, oz)| {
                let dx = (*ox as f32) - (tx as f32);
                let dz = (*oz as f32) - (tz as f32);
                (dx * dx + dz * dz).sqrt() < 5.5
            });

            if !too_close {
                spawned_trees.push((tx, tz));
                let trunk_height = 4;

                // Trunk (Wood)
                for dy in 0..trunk_height {
                    let y = ty + dy;
                    if y < sy {
                        grid[idx(tx, y, tz)] = BlockType::Wood;
                    }
                }

                // Leaves (3x3 canopy)
                let leaf_base_y = ty + trunk_height - 1;
                for lx in -1_i32..=1 {
                    for lz in -1_i32..=1 {
                        for ly in 0..2 {
                            if lx == 0 && lz == 0 && ly == 0 {
                                continue;
                            }
                            let px = tx as i32 + lx;
                            let py = leaf_base_y + ly;
                            let pz = tz as i32 + lz;

                            if px >= 0 && px < sx as i32 && py < sy && pz >= 0 && pz < sz as i32 {
                                let target_idx = idx(px as usize, py, pz as usize);
                                if grid[target_idx] == BlockType::Air {
                                    grid[target_idx] = BlockType::Leaves;
                                }
                            }
                        }
                    }
                }

                // Crown top leaf
                let crown_y = leaf_base_y + 2;
                if crown_y < sy {
                    let c_idx = idx(tx, crown_y, tz);
                    if grid[c_idx] == BlockType::Air {
                        grid[c_idx] = BlockType::Leaves;
                    }
                }
            }
        }

        // 3. Hidden Face Culling & Visible Block Extraction
        let half_x = (sx as f32 * params.block_size) / 2.0;
        let half_z = (sz as f32 * params.block_size) / 2.0;
        let offset_x = (chunk_x as f32) * (sx as f32 * params.block_size);
        let offset_z = (chunk_z as f32) * (sz as f32 * params.block_size);

        let mut total_blocks = 0;
        let mut culled_blocks = 0;
        let mut visible_blocks = Vec::with_capacity(sx * sz * 4);

        for x in 0..sx {
            for y in 0..sy {
                for z in 0..sz {
                    let b = grid[idx(x, y, z)];
                    if b == BlockType::Air {
                        continue;
                    }
                    total_blocks += 1;

                    if params.enable_culling && b != BlockType::Water && b != BlockType::Leaves {
                        // Check 6 neighbors: +X, -X, +Y, -Y, +Z, -Z
                        let has_air_neighbor = 
                            (x == 0 || !grid[idx(x - 1, y, z)].is_opaque()) ||
                            (x == sx - 1 || !grid[idx(x + 1, y, z)].is_opaque()) ||
                            (y == 0 || !grid[idx(x, y - 1, z)].is_opaque()) ||
                            (y == sy - 1 || !grid[idx(x, y + 1, z)].is_opaque()) ||
                            (z == 0 || !grid[idx(x, y, z - 1)].is_opaque()) ||
                            (z == sz - 1 || !grid[idx(x, y, z + 1)].is_opaque());

                        if !has_air_neighbor {
                            // Buried deep underground, completely surrounded -> CULL!
                            culled_blocks += 1;
                            continue;
                        }
                    }

                    let wx = (x as f32 * params.block_size) - half_x + (params.block_size / 2.0) + offset_x;
                    let wy = (y as f32 * params.block_size) + (params.block_size / 2.0);
                    let wz = (z as f32 * params.block_size) - half_z + (params.block_size / 2.0) + offset_z;

                    visible_blocks.push(VoxelBlockInstance {
                        x: wx,
                        y: wy,
                        z: wz,
                        block_type: b as u8,
                    });
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f32() * 1000.0; // milliseconds

        VoxelChunkResult {
            chunk_x,
            chunk_z,
            size_x: sx,
            size_z: sz,
            max_height: params.max_height,
            sea_level: params.sea_level,
            block_size: params.block_size,
            total_blocks_computed: total_blocks,
            culled_blocks_count: culled_blocks,
            visible_blocks_count: visible_blocks.len(),
            generation_time_ms: elapsed,
            blocks: visible_blocks,
            heightmap,
        }
    }

    /// Parallel Multi-Chunk Generation across all Linux CPU cores using Rayon!
    pub fn generate_region_parallel(
        radius_chunks: i32,
        params: &VoxelWorldParams,
    ) -> Vec<VoxelChunkResult> {
        let mut chunk_coords = Vec::new();
        for cx in -radius_chunks..=radius_chunks {
            for cz in -radius_chunks..=radius_chunks {
                chunk_coords.push((cx, cz));
            }
        }

        // Rayon par_iter spreads load across ALL CPU CORES!
        chunk_coords
            .par_iter()
            .map(|&(cx, cz)| Self::generate_chunk(cx, cz, params))
            .collect()
    }
}
