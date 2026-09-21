use crate::block::BlockType;
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::noise::NoiseGenerator;
use crate::world::types::{BiomeType, SEA_LEVEL};

/// Hermite smoothstep interpolation (3t^2 - 2t^3)
#[inline]
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    if (edge1 - edge0).abs() < 1e-9 {
        return 0.0;
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation between `a` and `b` by factor `t`
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Determines biome type, surface height, and presence of rivers with continuous slopes
pub fn calculate_biome_and_height(
    wx: f64,
    wz: f64,
    noise: &NoiseGenerator,
) -> (BiomeType, i32, bool) {
    let cont = noise.fbm_2d(wx * 0.0025, wz * 0.0025, 3, 0.5, 2.0);
    let temp = noise.fbm_2d(wx * 0.0018 + 500.0, wz * 0.0018 + 500.0, 3, 0.5, 2.0);
    let humid = noise.fbm_2d(wx * 0.0020 - 500.0, wz * 0.0020 - 500.0, 3, 0.5, 2.0);
    let mountain_noise = noise.ridged_fbm_2d(wx * 0.0035, wz * 0.0035, 4, 0.5, 2.0);
    let hills = noise.fbm_2d(wx * 0.009, wz * 0.009, 3, 0.5, 2.0);

    // 1. Continuous continental elevation curve (C1-smooth, eliminates sheer ocean/coast drop-offs)
    let base_cont_h = if cont < -0.25 {
        // Deep ocean basin (depth ~40..48)
        let ocean_t = smoothstep(-0.60, -0.25, cont);
        lerp(40.0, 48.0, ocean_t)
    } else if cont < -0.05 {
        // Continental shelf (depth ~48..61.5)
        let shelf_t = smoothstep(-0.25, -0.05, cont);
        lerp(48.0, 61.5, shelf_t)
    } else if cont < 0.05 {
        // Beach & coastal shoreline (Y ~61.5..65.5, crossing sea level 64.0)
        let coast_t = smoothstep(-0.05, 0.05, cont);
        lerp(61.5, 65.5, coast_t)
    } else if cont < 0.25 {
        // Coastal lowlands & plains (Y ~65.5..70.0)
        let land_t = smoothstep(0.05, 0.25, cont);
        lerp(65.5, 70.0, land_t)
    } else {
        // Inland plateaus (Y ~70.0..73.5)
        let high_t = smoothstep(0.25, 0.60, cont);
        lerp(70.0, 73.5, high_t)
    };

    // 2. Rolling hills: gentle ripples on the ocean floor, rolling terrain inland
    let hill_weight = smoothstep(-0.10, 0.15, cont);
    let hill_amp = lerp(1.5, 5.0, hill_weight);
    let hill_h = hills * hill_amp;

    // 3. Mountain elevation: inland gating with smoothstep foothills and quadratic alpine peaks
    let inland_factor = smoothstep(0.08, 0.28, cont);
    let mountain_weight = smoothstep(0.22, 0.68, mountain_noise) * inland_factor;
    // Linear term gives gentle, walkable foothills; quadratic term creates majestic peaks up to Y=118
    let mountain_h = mountain_weight * 10.0 + (mountain_weight * mountain_weight) * 34.0;

    let base_height = base_cont_h + hill_h + mountain_h;

    // 4. Biome classification
    let biome = if cont < -0.05 {
        if temp < -0.22 {
            BiomeType::FrozenOcean
        } else {
            BiomeType::Ocean
        }
    } else if cont < 0.05 {
        BiomeType::Beach
    } else if mountain_weight > 0.38 {
        BiomeType::Mountains
    } else if temp > 0.26 && humid < -0.05 {
        BiomeType::Desert
    } else if temp < -0.22 {
        BiomeType::SnowyTundra
    } else if humid > 0.15 {
        BiomeType::Forest
    } else {
        BiomeType::Plains
    };

    // 5. Smooth U-shaped river valley carving
    let river_noise = noise
        .perlin_2d(wx * 0.0035 + 100.0, wz * 0.0035 + 200.0)
        .abs();
    let is_river = river_noise < 0.035 && cont > -0.05 && biome != BiomeType::Desert;

    let final_height = if is_river {
        let river_factor = river_noise / 0.035;
        let t = smoothstep(0.0, 1.0, river_factor);
        let max_carve = 7.0 + 3.0 * (1.0 - mountain_weight);
        let river_bed = (base_height - max_carve).max(SEA_LEVEL as f64 - 3.0);
        lerp(river_bed, base_height, t)
    } else {
        base_height
    };

    let clamped = (final_height.round() as i32).clamp(5, (CHUNK_HEIGHT - 6) as i32);
    (biome, clamped, is_river)
}

pub(crate) fn pseudo_hash_3d(x: i32, y: i32, z: i32, seed: u64) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x45d9f3b)
        ^ (y as u32).wrapping_mul(0x1b192e23)
        ^ (z as u32).wrapping_mul(0x27d4eb2d)
        ^ (seed as u32);
    h = ((h >> 16) ^ h).wrapping_mul(0x45d9f3b);
    (h >> 16) ^ h
}

/// Generates a single chunk with biomes, ores, 3D caves, rivers, and trees based on the Seed
pub fn generate_chunk(cx: i32, cz: i32, noise: &NoiseGenerator, seed: u64) -> Chunk {
    let mut chunk = Chunk::new();
    let world_base_x = cx * CHUNK_WIDTH as i32;
    let world_base_z = cz * CHUNK_DEPTH as i32;

    let mut surface_heights = [[0i32; CHUNK_DEPTH]; CHUNK_WIDTH];
    let mut biomes = [[BiomeType::Plains; CHUNK_DEPTH]; CHUNK_WIDTH];
    let mut river_flags = [[false; CHUNK_DEPTH]; CHUNK_WIDTH];

    // Single-pass loop fusion: biomes, terrain stratification, 3D caves, ores, and liquids.
    // Traverses column-by-column, writing each voxel directly into contiguous memory without redundant reads or overwrites.
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx_i = world_base_x + lx as i32;
            let wz_i = world_base_z + lz as i32;
            let wx_f = wx_i as f64;
            let wz_f = wz_i as f64;

            let (biome, h, is_river) = calculate_biome_and_height(wx_f, wz_f, noise);
            surface_heights[lx][lz] = h;
            biomes[lx][lz] = biome;
            river_flags[lx][lz] = is_river;

            // Indestructible bedrock at world base
            chunk.set_fast(lx, 0, lz, BlockType::Bedrock);
            let bedrock_1 = pseudo_hash_3d(wx_i, 1, wz_i, seed).is_multiple_of(2);
            if bedrock_1 {
                chunk.set_fast(lx, 1, lz, BlockType::Bedrock);
            }

            let max_cave_y = (h - 4).min(110);

            // Vertical terrain column sweep from bedrock to surface height h
            for y in 1..=h {
                if y == 1 && bedrock_1 {
                    continue;
                }

                // 3D Underground Caves and Caverns
                let in_cave_zone = y >= 4 && y < max_cave_y && !(h <= SEA_LEVEL && y >= h - 4);
                if in_cave_zone {
                    let wy = y as f64;
                    let n1 = noise.fbm_3d(wx_f * 0.025, wy * 0.035, wz_f * 0.025, 2, 0.5, 2.0);
                    let n2 = noise.fbm_3d(
                        wx_f * 0.025 + 31.4,
                        wy * 0.035,
                        wz_f * 0.025 + 73.1,
                        2,
                        0.5,
                        2.0,
                    );
                    let is_tunnel = (n1 * n1 + n2 * n2) < 0.013;

                    let is_cave = if is_tunnel {
                        true
                    } else if y < 45 {
                        let n_room =
                            noise.fbm_3d(wx_f * 0.02, wy * 0.025, wz_f * 0.02, 2, 0.5, 2.0);
                        n_room < -0.42
                    } else {
                        false
                    };

                    if is_cave {
                        // Chunk is pre-initialized to Air; skip writes and ore evaluation
                        continue;
                    }
                }

                // Geological layer determination
                let base_block = match biome {
                    BiomeType::Desert => {
                        if y == h || y >= h - 3 {
                            BlockType::Sand
                        } else if y >= h - 7 {
                            BlockType::Sandstone
                        } else {
                            BlockType::Stone
                        }
                    }
                    BiomeType::Ocean | BiomeType::FrozenOcean => {
                        if y == h {
                            if pseudo_hash_3d(wx_i / 4, 0, wz_i / 4, seed).is_multiple_of(3) {
                                BlockType::Gravel
                            } else {
                                BlockType::Sand
                            }
                        } else if y >= h - 3 {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        }
                    }
                    BiomeType::Beach => {
                        if y >= h - 3 {
                            BlockType::Sand
                        } else {
                            BlockType::Stone
                        }
                    }
                    BiomeType::SnowyTundra => {
                        if y == h {
                            BlockType::Snow
                        } else if y >= h - 3 {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        }
                    }
                    BiomeType::Mountains => {
                        if y == h {
                            if h > 102 {
                                BlockType::Snow
                            } else if h > 86 {
                                BlockType::Stone
                            } else {
                                BlockType::Grass
                            }
                        } else if y >= h - 3 && h <= 86 {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        }
                    }
                    BiomeType::Plains | BiomeType::Forest => {
                        if y == h {
                            BlockType::Grass
                        } else if y >= h - 3 {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        }
                    }
                };

                // Ore vein generation within deep stone strata
                let block = if base_block == BlockType::Stone && y >= 2 && y < h - 4 {
                    let hash = pseudo_hash_3d(wx_i, y, wz_i, seed);
                    if y <= 16 && hash.is_multiple_of(179) {
                        BlockType::DiamondOre
                    } else if y <= 32 && hash.is_multiple_of(109) {
                        BlockType::GoldOre
                    } else if y <= 64 && hash.is_multiple_of(41) {
                        BlockType::IronOre
                    } else if y <= 115 && hash.is_multiple_of(25) {
                        BlockType::CoalOre
                    } else if hash.is_multiple_of(79) {
                        BlockType::Gravel
                    } else {
                        BlockType::Stone
                    }
                } else {
                    base_block
                };

                chunk.set_fast(lx, y as usize, lz, block);
            }

            // Liquid filling up to SEA_LEVEL for oceans, lakes, and rivers
            if h < SEA_LEVEL {
                for y in (h + 1)..=SEA_LEVEL {
                    let liquid = if biome == BiomeType::FrozenOcean && y == SEA_LEVEL {
                        BlockType::Ice
                    } else {
                        BlockType::Water
                    };
                    chunk.set_fast(lx, y as usize, lz, liquid);
                }
            }
        }
    }

    // 4. Vegetation and Surface Features (Trees & Cacti)
    for lx in 2..(CHUNK_WIDTH - 2) {
        for lz in 2..(CHUNK_DEPTH - 2) {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = surface_heights[lx][lz];
            let biome = biomes[lx][lz];
            let is_river = river_flags[lx][lz];

            if is_river || h <= SEA_LEVEL + 1 || h + 6 >= CHUNK_HEIGHT as i32 {
                continue;
            }

            let hash = pseudo_hash_3d(wx, h, wz, seed);

            match biome {
                BiomeType::Desert => {
                    // Desert cacti
                    if hash.is_multiple_of(41)
                        && chunk.get(lx as i32, h, lz as i32) == BlockType::Sand
                    {
                        let cactus_h = 2 + (hash % 2) as i32;
                        for cy in 1..=cactus_h {
                            chunk.set(lx as i32, h + cy, lz as i32, BlockType::Cactus);
                        }
                    }
                }
                BiomeType::Forest => {
                    // Dense forest trees
                    if hash.is_multiple_of(16)
                        && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass
                    {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, false);
                    }
                }
                BiomeType::Plains => {
                    // Scattered plains trees
                    if hash.is_multiple_of(45)
                        && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass
                    {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, false);
                    }
                }
                BiomeType::SnowyTundra => {
                    // Conical pine trees in snowy tundra
                    if hash.is_multiple_of(35)
                        && chunk.get(lx as i32, h, lz as i32) == BlockType::Snow
                    {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, true);
                    }
                }
                BiomeType::Mountains => {
                    // Conical pine trees in lower mountain foothills below the alpine tree line
                    if h <= 84
                        && hash.is_multiple_of(30)
                        && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass
                    {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, true);
                    }
                }
                _ => {}
            }
        }
    }

    chunk
}

fn spawn_tree(chunk: &mut Chunk, lx: i32, h: i32, lz: i32, is_pine: bool) {
    let trunk_h = if is_pine { 5 } else { 4 };
    for ty in (h + 1)..=(h + trunk_h) {
        chunk.set(lx, ty, lz, BlockType::Wood);
    }

    if is_pine {
        // Conical pine
        for dy in (h + 3)..=(h + 6) {
            let radius: i32 = if dy == h + 6 {
                0
            } else if dy >= h + 5 {
                1
            } else {
                2
            };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if radius == 2 && dx.abs() == 2 && dz.abs() == 2 {
                        continue;
                    }
                    let tx = lx + dx;
                    let tz = lz + dz;
                    if Chunk::in_bounds(tx, dy, tz) && chunk.get(tx, dy, tz) == BlockType::Air {
                        chunk.set(tx, dy, tz, BlockType::Leaves);
                    }
                }
            }
        }
    } else {
        // Round canopy oak
        for dx in -2_i32..=2_i32 {
            for dz in -2_i32..=2_i32 {
                for dy in (h + 3)..=(h + 6) {
                    if dy == h + 6 && (dx.abs() > 1 || dz.abs() > 1) {
                        continue;
                    }
                    if dx.abs() == 2 && dz.abs() == 2 && dy >= h + 5 {
                        continue;
                    }
                    let tx = lx + dx;
                    let tz = lz + dz;
                    if Chunk::in_bounds(tx, dy, tz) && chunk.get(tx, dy, tz) == BlockType::Air {
                        chunk.set(tx, dy, tz, BlockType::Leaves);
                    }
                }
            }
        }
    }
}
