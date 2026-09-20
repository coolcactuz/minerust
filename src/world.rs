use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::menu::GraphicsSettings;
use crate::mesher::build_chunk_mesh;
use crate::noise::NoiseGenerator;

pub const SEA_LEVEL: i32 = 128;
pub const VIEW_DISTANCE: i32 = 16;
pub const MAX_CHUNK_DISPATCH_PER_FRAME: usize = 32;

/// Represents the world seed (numeric or derived from string/text)
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(133742)
    }
}

impl WorldSeed {
    pub fn from_str(s: &str) -> Self {
        let trimmed = s.trim();
        if let Ok(num) = trimmed.parse::<u64>() {
            Self(num)
        } else {
            // Deterministic 64-bit FNV-1a hash algorithm for strings
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in trimmed.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Self(hash)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BiomeType {
    Ocean,
    FrozenOcean,
    Beach,
    Plains,
    Forest,
    Desert,
    SnowyTundra,
    Mountains,
}

#[derive(Resource)]
pub struct WorldGrid {
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_entities: HashMap<IVec2, Entity>,
    pub modified_chunks: HashSet<IVec2>,
    pub in_progress_chunks: HashSet<IVec2>,
    pub last_player_chunk: IVec2,
    pub generation_queue: Vec<IVec2>,
    pub save_dir: PathBuf,
    pub seed: WorldSeed,
    pub noise: NoiseGenerator,
    pub block_material: Option<Handle<StandardMaterial>>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        let seed = WorldSeed::default();
        let noise = NoiseGenerator::new(seed.0);
        Self {
            chunks: HashMap::default(),
            chunk_entities: HashMap::default(),
            modified_chunks: HashSet::default(),
            in_progress_chunks: HashSet::default(),
            last_player_chunk: IVec2::new(i32::MAX, i32::MAX),
            generation_queue: Vec::new(),
            save_dir: PathBuf::from("saves/world/chunks"),
            seed,
            noise,
            block_material: None,
        }
    }
}

#[derive(Resource)]
pub struct ChunkGeneratorPool {
    pub tx: Sender<(IVec2, Chunk, bool)>,
    pub rx: Mutex<Receiver<(IVec2, Chunk, bool)>>,
}

impl Default for ChunkGeneratorPool {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx: Mutex::new(rx),
        }
    }
}

impl WorldGrid {
    pub fn new(seed: WorldSeed) -> Self {
        let noise = NoiseGenerator::new(seed.0);
        Self {
            chunks: HashMap::default(),
            chunk_entities: HashMap::default(),
            modified_chunks: HashSet::default(),
            in_progress_chunks: HashSet::default(),
            last_player_chunk: IVec2::new(i32::MAX, i32::MAX),
            generation_queue: Vec::new(),
            save_dir: PathBuf::from(format!("saves/world_{}/chunks", seed.0)),
            seed,
            noise,
            block_material: None,
        }
    }

    #[inline]
    pub fn world_to_chunk_coord(wx: i32, wz: i32) -> (IVec2, usize, usize) {
        let cx = wx.div_euclid(CHUNK_WIDTH as i32);
        let cz = wz.div_euclid(CHUNK_DEPTH as i32);
        let lx = wx.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_DEPTH as i32) as usize;
        (IVec2::new(cx, cz), lx, lz)
    }

    pub fn get_block(&self, pos: IVec3) -> BlockType {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }
        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get(&c_coord) {
            chunk.get(lx as i32, pos.y, lz as i32)
        } else {
            BlockType::Air
        }
    }

    pub fn is_solid_at(&self, pos: IVec3) -> bool {
        self.get_block(pos).is_solid()
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec2> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return Vec::new();
        }

        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        let mut dirty_chunks = Vec::new();

        if let Some(chunk) = self.chunks.get_mut(&c_coord) {
            chunk.set(lx as i32, pos.y, lz as i32, block);
            dirty_chunks.push(c_coord);
            self.modified_chunks.insert(c_coord);

            if lx == 0 {
                dirty_chunks.push(c_coord + IVec2::new(-1, 0));
            } else if lx == CHUNK_WIDTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(1, 0));
            }

            if lz == 0 {
                dirty_chunks.push(c_coord + IVec2::new(0, -1));
            } else if lz == CHUNK_DEPTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(0, 1));
            }
        }

        dirty_chunks
    }

    pub fn save_chunk_to_disk(&self, coord: &IVec2) -> std::io::Result<()> {
        let Some(chunk) = self.chunks.get(coord) else {
            return Ok(());
        };
        std::fs::create_dir_all(&self.save_dir)?;
        let path = self.save_dir.join(format!("chunk_{}_{}.bin", coord.x, coord.y));
        std::fs::write(path, chunk.to_bytes())?;
        Ok(())
    }

    pub fn load_chunk_from_disk_path(save_dir: &std::path::Path, coord: &IVec2) -> Option<Chunk> {
        let path = save_dir.join(format!("chunk_{}_{}.bin", coord.x, coord.y));
        if path.exists() {
            if let Ok(bytes) = std::fs::read(path) {
                return Chunk::from_bytes(&bytes);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn load_chunk_from_disk(&self, coord: &IVec2) -> Option<Chunk> {
        Self::load_chunk_from_disk_path(&self.save_dir, coord)
    }
}

/// Determines biome type, surface height, and presence of rivers
pub fn calculate_biome_and_height(
    wx: f64,
    wz: f64,
    noise: &NoiseGenerator,
) -> (BiomeType, i32, bool) {
    let cont = noise.fbm_2d(wx * 0.0025, wz * 0.0025, 3, 0.5, 2.0);
    let temp = noise.fbm_2d(wx * 0.0018 + 500.0, wz * 0.0018 + 500.0, 3, 0.5, 2.0);
    let humid = noise.fbm_2d(wx * 0.0020 - 500.0, wz * 0.0020 - 500.0, 3, 0.5, 2.0);
    let mountain = noise.ridged_fbm_2d(wx * 0.006, wz * 0.006, 4, 0.5, 2.0);
    let hills = noise.fbm_2d(wx * 0.012, wz * 0.012, 3, 0.5, 2.0);

    let (biome, base_height) = if cont < -0.15 {
        // Deep ocean basin (trenches down to Y=40..80)
        let ocean_h = 45.0 + (cont + 0.15) * 60.0;
        if temp < -0.25 {
            (BiomeType::FrozenOcean, ocean_h)
        } else {
            (BiomeType::Ocean, ocean_h)
        }
    } else if cont < 0.02 {
        // Coast and beach
        (BiomeType::Beach, (SEA_LEVEL as f64) + hills * 3.0)
    } else {
        // Inland
        if cont > 0.10 && mountain > 0.35 {
            // High mountain range (peaks reaching up to Y=260..310)
            let m_h = (SEA_LEVEL as f64 + 18.0) + hills * 24.0 + mountain * 130.0;
            (BiomeType::Mountains, m_h)
        } else if temp > 0.26 && humid < -0.05 {
            // Hot desert
            let d_h = (SEA_LEVEL as f64 + 5.0) + hills * 14.0;
            (BiomeType::Desert, d_h)
        } else if temp < -0.22 {
            // Snowy tundra
            let t_h = (SEA_LEVEL as f64 + 6.0) + hills * 16.0;
            (BiomeType::SnowyTundra, t_h)
        } else if humid > 0.15 {
            // Forest
            let f_h = (SEA_LEVEL as f64 + 6.0) + hills * 18.0;
            (BiomeType::Forest, f_h)
        } else {
            // Plains
            let p_h = (SEA_LEVEL as f64 + 4.0) + hills * 14.0;
            (BiomeType::Plains, p_h)
        }
    };

    // Rivers: carve winding river valleys toward the sea
    let river_noise = noise.perlin_2d(wx * 0.004 + 100.0, wz * 0.004 + 200.0).abs();
    let is_river = river_noise < 0.038 && cont > -0.10 && biome != BiomeType::Desert;

    let final_height = if is_river {
        let river_factor = (river_noise / 0.038).clamp(0.0, 1.0);
        let river_bed = (SEA_LEVEL as f64 - 5.0).min(base_height - 6.0);
        river_bed + (base_height - river_bed) * river_factor
    } else {
        base_height
    };

    let clamped = (final_height.round() as i32).clamp(5, (CHUNK_HEIGHT - 12) as i32);
    (biome, clamped, is_river)
}

fn pseudo_hash_3d(x: i32, y: i32, z: i32, seed: u64) -> u32 {
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

    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = (world_base_x + lx as i32) as f64;
            let wz = (world_base_z + lz as i32) as f64;
            let (biome, h, is_river) = calculate_biome_and_height(wx, wz, noise);
            surface_heights[lx][lz] = h;
            biomes[lx][lz] = biome;
            river_flags[lx][lz] = is_river;
        }
    }

    // 1. Terrain and Geological Stratification of Biomes
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = surface_heights[lx][lz];
            let biome = biomes[lx][lz];

            // Indestructible bedrock at world base
            chunk.set_fast(lx, 0, lz, BlockType::Bedrock);
            if pseudo_hash_3d(wx, 1, wz, seed) % 2 == 0 {
                chunk.set_fast(lx, 1, lz, BlockType::Bedrock);
            }

            for y in 1..=h {
                let block = match biome {
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
                            if pseudo_hash_3d(wx, y, wz, seed) % 5 == 0 {
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
                            if h > 210 {
                                BlockType::Snow
                            } else if h > 175 {
                                BlockType::Stone
                            } else {
                                BlockType::Grass
                            }
                        } else if y >= h - 2 && h <= 175 {
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

    // 2. Underground Ore Vein Generation (Coal, Iron, Gold, Diamond)
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = surface_heights[lx][lz];

            for y in 2..(h - 4) {
                let yu = y as usize;
                if chunk.get_fast(lx, yu, lz) == BlockType::Stone {
                    let hash = pseudo_hash_3d(wx, y, wz, seed);

                    // Diamond: deep underground (levels 2-48)
                    if y <= 48 && hash % 179 == 0 {
                        chunk.set_fast(lx, yu, lz, BlockType::DiamondOre);
                    }
                    // Gold: rare (levels 4-96)
                    else if y <= 96 && hash % 109 == 0 {
                        chunk.set_fast(lx, yu, lz, BlockType::GoldOre);
                    }
                    // Iron: common (levels 6-200)
                    else if y <= 200 && hash % 41 == 0 {
                        chunk.set_fast(lx, yu, lz, BlockType::IronOre);
                    }
                    // Coal: abundant (levels 10-280)
                    else if y <= 280 && hash % 25 == 0 {
                        chunk.set_fast(lx, yu, lz, BlockType::CoalOre);
                    }
                    // Underground gravel pockets
                    else if hash % 79 == 0 {
                        chunk.set_fast(lx, yu, lz, BlockType::Gravel);
                    }
                }
            }
        }
    }

    // 3. 3D Underground Caves and Tunnels
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = (world_base_x + lx as i32) as f64;
            let wz = (world_base_z + lz as i32) as f64;
            let h = surface_heights[lx][lz];

            let max_cave_y = (h - 4).min(260);
            if max_cave_y <= 4 {
                continue;
            }

            for y in 4..max_cave_y {
                let wy = y as f64;

                // Protect waterbed floors
                if h <= SEA_LEVEL && y >= h - 4 {
                    continue;
                }

                // Winding 3D tunnels
                let n1 = noise.fbm_3d(wx * 0.025, wy * 0.035, wz * 0.025, 2, 0.5, 2.0);
                let n2 = noise.fbm_3d(wx * 0.025 + 31.4, wy * 0.035, wz * 0.025 + 73.1, 2, 0.5, 2.0);
                let is_tunnel = (n1 * n1 + n2 * n2) < 0.013;

                // Large underground cavern rooms (only evaluate noise if not already a tunnel)
                let is_room = if !is_tunnel && y < 90 {
                    let n_room = noise.fbm_3d(wx * 0.02, wy * 0.025, wz * 0.02, 2, 0.5, 2.0);
                    n_room < -0.42
                } else {
                    false
                };

                if is_tunnel || is_room {
                    chunk.set_fast(lx, y as usize, lz, BlockType::Air);
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
                    if hash % 41 == 0 && chunk.get(lx as i32, h, lz as i32) == BlockType::Sand {
                        let cactus_h = 2 + (hash % 2) as i32;
                        for cy in 1..=cactus_h {
                            chunk.set(lx as i32, h + cy, lz as i32, BlockType::Cactus);
                        }
                    }
                }
                BiomeType::Forest => {
                    // Dense forest trees
                    if hash % 16 == 0 && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, false);
                    }
                }
                BiomeType::Plains => {
                    // Scattered plains trees
                    if hash % 45 == 0 && chunk.get(lx as i32, h, lz as i32) == BlockType::Grass {
                        spawn_tree(&mut chunk, lx as i32, h, lz as i32, false);
                    }
                }
                BiomeType::SnowyTundra => {
                    // Conical pine trees in snowy tundra
                    if hash % 35 == 0 && chunk.get(lx as i32, h, lz as i32) == BlockType::Snow {
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
            let radius: i32 = if dy == h + 6 { 0 } else if dy >= h + 5 { 1 } else { 2 };
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

/// Updates or creates the mesh for the specified chunk
pub fn update_chunk_mesh(
    coord: &IVec2,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let Some(chunk) = world.chunks.get(coord) else {
        return;
    };

    let north = world.chunks.get(&(*coord + IVec2::new(0, 1)));
    let south = world.chunks.get(&(*coord + IVec2::new(0, -1)));
    let east = world.chunks.get(&(*coord + IVec2::new(1, 0)));
    let west = world.chunks.get(&(*coord + IVec2::new(-1, 0)));

    let new_mesh = build_chunk_mesh(chunk, north, south, east, west);

    let world_pos = Vec3::new(
        (coord.x * CHUNK_WIDTH as i32) as f32,
        0.0,
        (coord.y * CHUNK_DEPTH as i32) as f32,
    );

    let material = world.block_material.clone().unwrap_or_else(|| {
        materials.add(StandardMaterial {
            cull_mode: None,
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            ..default()
        })
    });

    if let Some(&entity) = world.chunk_entities.get(coord) {
        if let Some(mesh) = new_mesh {
            commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
        } else {
            commands.entity(entity).despawn();
            world.chunk_entities.remove(coord);
        }
    } else if let Some(mesh) = new_mesh {
        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(world_pos),
            ))
            .id();

        world.chunk_entities.insert(*coord, entity);
    }
}

/// Continuous chunk streaming system based on player camera position with multithreaded generation
pub fn world_streaming_system(
    mut commands: Commands,
    camera_query: Query<&Transform, With<FpsCamera>>,
    mut world: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Option<Res<GraphicsSettings>>,
    pool: Res<ChunkGeneratorPool>,
) {
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };

    let px = cam_transform.translation.x as i32;
    let pz = cam_transform.translation.z as i32;
    let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(px, pz);

    let view_dist = settings.as_ref().map_or(VIEW_DISTANCE, |s| s.view_distance);
    let settings_changed = settings.as_ref().map_or(false, |s| s.is_changed());

    if player_chunk != world.last_player_chunk || settings_changed {
        world.last_player_chunk = player_chunk;

        let mut needed_chunks = Vec::new();
        for dx in -view_dist..=view_dist {
            for dz in -view_dist..=view_dist {
                let coord = player_chunk + IVec2::new(dx, dz);
                // Chunk needs to be loaded if it has no active GPU mesh and is not already loaded or in progress
                if !world.chunk_entities.contains_key(&coord)
                    && !world.chunks.contains_key(&coord)
                    && !world.in_progress_chunks.contains(&coord)
                {
                    needed_chunks.push(coord);
                }
            }
        }

        // Sort descending so pop() takes the closest chunks first
        needed_chunks.sort_by_key(|c| {
            let diff = *c - player_chunk;
            -(diff.x * diff.x + diff.y * diff.y)
        });

        world.generation_queue = needed_chunks;

        let max_dist = view_dist + 2;
        let mut chunks_to_remove = Vec::new();

        for coord in world.chunks.keys() {
            let diff = *coord - player_chunk;
            if diff.x.abs() > max_dist || diff.y.abs() > max_dist {
                chunks_to_remove.push(*coord);
            }
        }

        for coord in chunks_to_remove {
            // Despawn 3D mesh entity from GPU
            if let Some(entity) = world.chunk_entities.remove(&coord) {
                commands.entity(entity).despawn();
            }

            // If the chunk was modified by the player, persist it to disk and keep it tracked
            if world.modified_chunks.contains(&coord) {
                let _ = world.save_chunk_to_disk(&coord);
            } else {
                // Unmodified chunks can be unloaded from RAM to conserve memory
                world.chunks.remove(&coord);
            }
        }
    }

    // 1. Dispatch background chunk generation tasks across all CPU cores
    let mut dispatched = 0;
    while dispatched < MAX_CHUNK_DISPATCH_PER_FRAME && !world.generation_queue.is_empty() {
        let coord = world.generation_queue.pop().unwrap();
        if world.chunks.contains_key(&coord) || world.in_progress_chunks.contains(&coord) {
            continue;
        }

        world.in_progress_chunks.insert(coord);

        let tx = pool.tx.clone();
        let noise = world.noise.clone();
        let seed = world.seed.0;
        let save_dir = world.save_dir.clone();

        AsyncComputeTaskPool::get()
            .spawn(async move {
                // Check disk cache first
                if let Some(loaded_chunk) = WorldGrid::load_chunk_from_disk_path(&save_dir, &coord) {
                    let _ = tx.send((coord, loaded_chunk, true));
                    return;
                }

                // Procedural generation in parallel on thread pool
                let chunk = generate_chunk(coord.x, coord.y, &noise, seed);
                let _ = tx.send((coord, chunk, false));
            })
            .detach();

        dispatched += 1;
    }

    // 2. Receive finished chunks from background threads and queue for meshing
    let max_dist = view_dist + 2;
    let mut chunks_to_mesh: Vec<IVec2> = Vec::new();

    if let Ok(rx) = pool.rx.lock() {
        while let Ok((coord, chunk, from_disk)) = rx.try_recv() {
            world.in_progress_chunks.remove(&coord);

            let diff = coord - player_chunk;
            if diff.x.abs() > max_dist || diff.y.abs() > max_dist {
                continue;
            }

            if from_disk {
                world.modified_chunks.insert(coord);
            }
            world.chunks.insert(coord, chunk);
            chunks_to_mesh.push(coord);

            for neighbor_coord in [
                coord + IVec2::new(-1, 0),
                coord + IVec2::new(1, 0),
                coord + IVec2::new(0, -1),
                coord + IVec2::new(0, 1),
            ] {
                if world.chunks.contains_key(&neighbor_coord) {
                    chunks_to_mesh.push(neighbor_coord);
                }
            }
        }
    }

    // 3. Update meshes for all affected chunks (once per frame per unique chunk)
    if !chunks_to_mesh.is_empty() {
        chunks_to_mesh.sort_unstable_by_key(|c| (c.x, c.y));
        chunks_to_mesh.dedup();

        for coord in chunks_to_mesh {
            update_chunk_mesh(
                &coord,
                &mut commands,
                &mut world,
                &mut meshes,
                &mut materials,
            );
        }
    }
}
