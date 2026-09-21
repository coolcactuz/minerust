use bevy::ecs::system::SystemParam;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::coords::{BlockPos, ChunkPos};
use crate::error::WorldError;
use crate::menu::GraphicsSettings;
use crate::mesher::build_chunk_mesh;
use crate::noise::NoiseGenerator;

pub const SEA_LEVEL: i32 = 64;
pub const VIEW_DISTANCE: i32 = 16;
pub const MAX_CHUNK_DISPATCH_PER_FRAME: usize = 12;
pub const MAX_MESHES_PER_FRAME: usize = 6;

/// Represents the world seed (numeric or derived from string/text)
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(133742)
    }
}

impl std::str::FromStr for WorldSeed {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if let Ok(num) = trimmed.parse::<u64>() {
            Ok(Self(num))
        } else {
            // Deterministic 64-bit FNV-1a hash algorithm for strings
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in trimmed.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Self(hash))
        }
    }
}

impl WorldSeed {
    #[must_use]
    pub fn random() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(133742, |d| d.as_nanos());
        let mut z = (nanos as u64).wrapping_add(0x9e3779b97f4a7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        Self((z ^ (z >> 31)) & 0x7fff_ffff_ffff_ffff)
    }

    #[must_use]
    pub fn from_seed_str(s: &str) -> Self {
        use std::str::FromStr;
        Self::from_str(s).unwrap_or_default()
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

pub const CHUNK_CACHE_CAPACITY: usize = 512;

#[derive(Resource)]
pub struct WorldGrid {
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_cache: quick_cache::sync::Cache<IVec2, Chunk>,
    pub chunk_entities: HashMap<IVec2, Entity>,
    pub modified_chunks: HashSet<IVec2>,
    pub in_progress_chunks: HashSet<IVec2>,
    pub in_progress_meshes: HashSet<IVec2>,
    pub last_player_chunk: IVec2,
    pub generation_queue: Vec<IVec2>,
    pub mesh_queue: Vec<IVec2>,
    pub queued_for_mesh: HashSet<IVec2>,
    pub dirty_chunks: HashSet<IVec2>,
    pub save_dir: PathBuf,
    pub seed: WorldSeed,
    pub noise: NoiseGenerator,
    pub block_material: Option<Handle<StandardMaterial>>,
    pub total_vertices: usize,
    pub chunk_vertices: HashMap<IVec2, usize>,
    pub chunk_lod: HashMap<IVec2, u8>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        let seed = WorldSeed::default();
        let noise = NoiseGenerator::new(seed.0);
        Self {
            chunks: HashMap::default(),
            chunk_cache: quick_cache::sync::Cache::new(CHUNK_CACHE_CAPACITY),
            chunk_entities: HashMap::default(),
            modified_chunks: HashSet::default(),
            dirty_chunks: HashSet::default(),
            in_progress_chunks: HashSet::default(),
            in_progress_meshes: HashSet::default(),
            last_player_chunk: IVec2::new(i32::MAX, i32::MAX),
            generation_queue: Vec::new(),
            mesh_queue: Vec::new(),
            queued_for_mesh: HashSet::default(),
            save_dir: PathBuf::from(format!("saves/world_{}/chunks", seed.0)),
            seed,
            noise,
            block_material: None,
            total_vertices: 0,
            chunk_vertices: HashMap::default(),
            chunk_lod: HashMap::default(),
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

#[derive(Resource)]
pub struct ChunkMesherPool {
    pub tx: Sender<(IVec2, Option<Mesh>, u8)>,
    pub rx: Mutex<Receiver<(IVec2, Option<Mesh>, u8)>>,
}

impl Default for ChunkMesherPool {
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
            chunk_cache: quick_cache::sync::Cache::new(CHUNK_CACHE_CAPACITY),
            chunk_entities: HashMap::default(),
            modified_chunks: HashSet::default(),
            dirty_chunks: HashSet::default(),
            in_progress_chunks: HashSet::default(),
            in_progress_meshes: HashSet::default(),
            last_player_chunk: IVec2::new(i32::MAX, i32::MAX),
            generation_queue: Vec::new(),
            mesh_queue: Vec::new(),
            queued_for_mesh: HashSet::default(),
            save_dir: PathBuf::from(format!("saves/world_{}/chunks", seed.0)),
            seed,
            noise,
            block_material: None,
            total_vertices: 0,
            chunk_vertices: HashMap::default(),
            chunk_lod: HashMap::default(),
        }
    }

    #[inline]
    pub fn queue_mesh(&mut self, coord: impl Into<ChunkPos>) {
        let c = coord.into().0;
        if self.queued_for_mesh.insert(c) {
            self.mesh_queue.push(c);
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

    pub fn get_block(&self, pos: impl Into<BlockPos>) -> BlockType {
        let pos = pos.into();
        if pos.y() < 0 || pos.y() >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }
        let (c_coord, local) = pos.to_chunk_and_local();
        if let Some(chunk) = self.chunks.get(&c_coord.0) {
            chunk.get_local(local)
        } else {
            BlockType::Air
        }
    }

    pub fn is_solid_at(&self, pos: impl Into<BlockPos>) -> bool {
        self.get_block(pos).is_solid()
    }

    pub fn set_block(&mut self, pos: impl Into<BlockPos>, block: BlockType) -> Vec<IVec2> {
        let pos = pos.into();
        if pos.y() < 0 || pos.y() >= CHUNK_HEIGHT as i32 {
            return Vec::new();
        }

        let (c_coord, local) = pos.to_chunk_and_local();
        let mut dirty_chunks = Vec::new();

        if let Some(chunk) = self.chunks.get_mut(&c_coord.0) {
            chunk.set_local(local, block);
            dirty_chunks.push(c_coord.0);
            self.modified_chunks.insert(c_coord.0);
            self.dirty_chunks.insert(c_coord.0);

            if local.x == 0 {
                dirty_chunks.push(c_coord.0 + IVec2::new(-1, 0));
            } else if usize::from(local.x) == CHUNK_WIDTH - 1 {
                dirty_chunks.push(c_coord.0 + IVec2::new(1, 0));
            }

            if local.z == 0 {
                dirty_chunks.push(c_coord.0 + IVec2::new(0, -1));
            } else if usize::from(local.z) == CHUNK_DEPTH - 1 {
                dirty_chunks.push(c_coord.0 + IVec2::new(0, 1));
            }
        }

        dirty_chunks
    }

    #[inline]
    pub fn world_dir(&self) -> PathBuf {
        PathBuf::from(format!("saves/world_{}", self.seed.0))
    }

    #[inline]
    pub fn player_save_path(&self) -> PathBuf {
        self.world_dir().join("player.json")
    }

    /// Saves all chunks that have been modified in memory and not yet flushed to disk.
    pub fn save_all_dirty_chunks(&mut self) -> Result<usize, WorldError> {
        if self.dirty_chunks.is_empty() {
            return Ok(0);
        }
        std::fs::create_dir_all(&self.save_dir)?;
        let mut saved = 0;
        let coords: Vec<IVec2> = self.dirty_chunks.drain().collect();
        for coord in coords {
            if let Some(chunk) = self.chunks.get(&coord) {
                let path = self
                    .save_dir
                    .join(format!("chunk_{}_{}.bin", coord.x, coord.y));
                let compressed = chunk.to_compressed_bytes();
                std::fs::write(&path, compressed)?;
                saved += 1;
            }
        }
        Ok(saved)
    }

    /// Saves all modified chunks currently in memory to disk.
    pub fn save_all_modified(&mut self) -> Result<usize, WorldError> {
        if self.modified_chunks.is_empty() {
            return Ok(0);
        }
        std::fs::create_dir_all(&self.save_dir)?;
        let mut saved = 0;
        for &coord in &self.modified_chunks {
            if let Some(chunk) = self.chunks.get(&coord) {
                let path = self
                    .save_dir
                    .join(format!("chunk_{}_{}.bin", coord.x, coord.y));
                let compressed = chunk.to_compressed_bytes();
                std::fs::write(&path, compressed)?;
                saved += 1;
            }
        }
        self.dirty_chunks.clear();
        Ok(saved)
    }

    pub fn save_chunk_to_disk(&self, coord: impl Into<ChunkPos>) -> Result<(), WorldError> {
        let coord = coord.into();
        let Some(chunk) = self.chunks.get(&coord.0) else {
            return Ok(());
        };
        std::fs::create_dir_all(&self.save_dir)?;
        let path = self
            .save_dir
            .join(format!("chunk_{}_{}.bin", coord.x(), coord.z()));
        std::fs::write(path, chunk.to_compressed_bytes())?;
        Ok(())
    }

    pub fn load_chunk_from_disk_path(
        save_dir: &std::path::Path,
        coord: impl Into<ChunkPos>,
    ) -> Result<Chunk, WorldError> {
        let coord = coord.into();
        let path = save_dir.join(format!("chunk_{}_{}.bin", coord.x(), coord.z()));
        let bytes = std::fs::read(path)?;
        Chunk::from_compressed_bytes(&bytes)
    }

    #[allow(dead_code)]
    pub fn load_chunk_from_disk(&self, coord: impl Into<ChunkPos>) -> Result<Chunk, WorldError> {
        Self::load_chunk_from_disk_path(&self.save_dir, coord)
    }

    /// Despawns all active chunk meshes and resets world state to switch to a new seed.
    pub fn reinitialize_with_seed(&mut self, new_seed: WorldSeed, commands: &mut Commands) {
        for (_, entity) in self.chunk_entities.drain() {
            commands.entity(entity).despawn();
        }
        self.chunks.clear();
        self.chunk_lod.clear();
        self.chunk_vertices.clear();
        self.total_vertices = 0;
        self.in_progress_chunks.clear();
        self.in_progress_meshes.clear();
        self.generation_queue.clear();
        self.mesh_queue.clear();
        self.queued_for_mesh.clear();
        self.dirty_chunks.clear();
        self.modified_chunks.clear();
        self.chunk_cache = quick_cache::sync::Cache::new(CHUNK_CACHE_CAPACITY);
        self.last_player_chunk = IVec2::new(i32::MAX, i32::MAX);

        self.seed = new_seed;
        self.noise = NoiseGenerator::new(new_seed.0);
        self.save_dir = PathBuf::from(format!("saves/world_{}/chunks", new_seed.0));
    }

    /// Pre-generates the initial 9x9 chunk grid around a center chunk in parallel across CPU cores.
    pub fn pregenerate_spawn_grid(
        &mut self,
        center_chunk: IVec2,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        let seed = self.seed.0;
        let noise = self.noise.clone();
        let save_dir = self.save_dir.clone();
        let initial_coords: Vec<IVec2> = (-4..=4)
            .flat_map(|cx| (-4..=4).map(move |cz| center_chunk + IVec2::new(cx, cz)))
            .collect();

        let mut loaded_chunks: Vec<(IVec2, Chunk, bool)> = Vec::with_capacity(initial_coords.len());

        std::thread::scope(|s| {
            let mut handles = Vec::with_capacity(initial_coords.len());
            for coord in &initial_coords {
                let noise_ref = &noise;
                let save_dir_ref = &save_dir;
                handles.push(s.spawn(move || {
                    if let Ok(chunk) = WorldGrid::load_chunk_from_disk_path(save_dir_ref, *coord) {
                        (*coord, chunk, true)
                    } else {
                        let chunk = generate_chunk(coord.x, coord.y, noise_ref, seed);
                        (*coord, chunk, false)
                    }
                }));
            }
            for handle in handles {
                if let Ok(res) = handle.join() {
                    loaded_chunks.push(res);
                }
            }
        });

        for (coord, chunk, from_disk) in loaded_chunks {
            if from_disk {
                self.modified_chunks.insert(coord);
            }
            self.chunks.insert(coord, chunk);
        }

        for coord in &initial_coords {
            update_chunk_mesh(coord, commands, self, meshes, materials, true, true);
        }
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
        // Deep ocean basin (depth ~14..36 blocks below sea level 64)
        let ocean_h = 42.0 + (cont + 0.15) * 30.0;
        if temp < -0.25 {
            (BiomeType::FrozenOcean, ocean_h)
        } else {
            (BiomeType::Ocean, ocean_h)
        }
    } else if cont < 0.02 {
        // Coast and beach
        (BiomeType::Beach, (SEA_LEVEL as f64) + hills * 2.0)
    } else {
        // Inland
        if cont > 0.10 && mountain > 0.35 {
            // High mountain range (peaks reaching up to Y=105..122)
            let m_h = (SEA_LEVEL as f64 + 14.0) + hills * 12.0 + mountain * 35.0;
            (BiomeType::Mountains, m_h)
        } else if temp > 0.26 && humid < -0.05 {
            // Hot desert
            let d_h = (SEA_LEVEL as f64 + 4.0) + hills * 8.0;
            (BiomeType::Desert, d_h)
        } else if temp < -0.22 {
            // Snowy tundra
            let t_h = (SEA_LEVEL as f64 + 6.0) + hills * 10.0;
            (BiomeType::SnowyTundra, t_h)
        } else if humid > 0.15 {
            // Forest
            let f_h = (SEA_LEVEL as f64 + 5.0) + hills * 10.0;
            (BiomeType::Forest, f_h)
        } else {
            // Plains
            let p_h = (SEA_LEVEL as f64 + 4.0) + hills * 8.0;
            (BiomeType::Plains, p_h)
        }
    };

    // Rivers: carve winding river valleys toward the sea
    let river_noise = noise
        .perlin_2d(wx * 0.004 + 100.0, wz * 0.004 + 200.0)
        .abs();
    let is_river = river_noise < 0.038 && cont > -0.10 && biome != BiomeType::Desert;

    let final_height = if is_river {
        let river_factor = (river_noise / 0.038).clamp(0.0, 1.0);
        let river_bed = (SEA_LEVEL as f64 - 4.0).min(base_height - 4.0);
        river_bed + (base_height - river_bed) * river_factor
    } else {
        base_height
    };

    let clamped = (final_height.round() as i32).clamp(5, (CHUNK_HEIGHT - 6) as i32);
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
                            if h > 105 {
                                BlockType::Snow
                            } else if h > 88 {
                                BlockType::Stone
                            } else {
                                BlockType::Grass
                            }
                        } else if y >= h - 2 && h <= 88 {
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

/// Computes the squared 3D Euclidean distance in world units from a player camera position to a chunk's geometry (AABB)
#[inline]
pub fn chunk_distance_sq_to_player(coord: IVec2, player_pos: Vec3, chunk: Option<&Chunk>) -> f32 {
    let min_x = (coord.x * CHUNK_WIDTH as i32) as f32;
    let max_x = min_x + CHUNK_WIDTH as f32;
    let min_z = (coord.y * CHUNK_DEPTH as i32) as f32;
    let max_z = min_z + CHUNK_DEPTH as f32;

    let max_y = chunk.map_or(SEA_LEVEL as f32, |c| (c.max_y as f32 + 1.0).max(1.0));

    let closest_x = player_pos.x.clamp(min_x, max_x);
    let closest_y = player_pos.y.clamp(0.0, max_y);
    let closest_z = player_pos.z.clamp(min_z, max_z);

    let dx = player_pos.x - closest_x;
    let dy = player_pos.y - closest_y;
    let dz = player_pos.z - closest_z;

    dx * dx + dy * dy + dz * dz
}

/// Applies or despawns a chunk mesh on the GPU
pub fn apply_chunk_mesh(
    coord: IVec2,
    new_mesh: Option<Mesh>,
    lod: u8,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let world_pos = Vec3::new(
        (coord.x * CHUNK_WIDTH as i32) as f32,
        0.0,
        (coord.y * CHUNK_DEPTH as i32) as f32,
    );

    // Track vertex counts and LOD
    world.chunk_lod.insert(coord, lod);
    let new_vert_count = new_mesh.as_ref().map_or(0, Mesh::count_vertices);
    let old_vert_count = world
        .chunk_vertices
        .insert(coord, new_vert_count)
        .unwrap_or(0);
    world.total_vertices = world.total_vertices.saturating_sub(old_vert_count) + new_vert_count;

    let material = world.block_material.clone().unwrap_or_else(|| {
        materials.add(StandardMaterial {
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            ..default()
        })
    });

    if let Some(&entity) = world.chunk_entities.get(&coord) {
        if let Some(mesh) = new_mesh {
            commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
        } else {
            commands.entity(entity).despawn();
            world.chunk_entities.remove(&coord);
        }
    } else if let Some(mesh) = new_mesh {
        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(world_pos),
            ))
            .id();

        world.chunk_entities.insert(coord, entity);
    }
}

/// Updates or creates the mesh for the specified chunk synchronously
pub fn update_chunk_mesh(
    coord: &IVec2,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    max_y_skip: bool,
    greedy: bool,
) {
    let Some(chunk) = world.chunks.get(coord) else {
        return;
    };

    let north = world.chunks.get(&(*coord + IVec2::new(0, 1)));
    let south = world.chunks.get(&(*coord + IVec2::new(0, -1)));
    let east = world.chunks.get(&(*coord + IVec2::new(1, 0)));
    let west = world.chunks.get(&(*coord + IVec2::new(-1, 0)));

    let lod = u8::from(greedy);
    let new_mesh = build_chunk_mesh(chunk, north, south, east, west, max_y_skip, greedy);
    apply_chunk_mesh(*coord, new_mesh, lod, commands, world, meshes, materials);
}

#[derive(SystemParam)]
pub struct WorldSettingsParams<'w> {
    pub graphics: Option<Res<'w, GraphicsSettings>>,
    pub dev: Option<Res<'w, crate::menu::DevSettings>>,
}

#[derive(SystemParam)]
pub struct WorldWorkerPools<'w> {
    pub generator: Res<'w, ChunkGeneratorPool>,
    pub mesher: Res<'w, ChunkMesherPool>,
}

#[derive(SystemParam)]
pub struct WorldMeshAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
}

/// Continuous chunk streaming system based on player camera position with multithreaded generation
pub fn world_streaming_system(
    mut commands: Commands,
    mut camera_query: Query<
        (&Transform, &mut Projection, Option<&mut DistanceFog>),
        With<FpsCamera>,
    >,
    mut world: ResMut<WorldGrid>,
    mut assets: WorldMeshAssets,
    settings: WorldSettingsParams,
    pools: WorldWorkerPools,
) {
    let Ok((cam_transform, mut projection, mut fog)) = camera_query.single_mut() else {
        return;
    };

    let px = cam_transform.translation.x as i32;
    let pz = cam_transform.translation.z as i32;
    let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(px, pz);

    let view_dist = settings
        .graphics
        .as_ref()
        .map_or(VIEW_DISTANCE, |s| s.view_distance);
    let pregen_margin = settings.dev.as_ref().map_or(2, |d| d.pregen_margin);
    let gen_dist = view_dist + pregen_margin;
    let unload_dist = gen_dist + 2;

    let settings_changed = settings.graphics.as_ref().is_some_and(|s| s.is_changed());
    let dev_changed = settings.dev.as_ref().is_some_and(|d| d.is_changed());

    if settings_changed {
        if let Projection::Perspective(ref mut persp) = *projection {
            persp.far = ((view_dist + 4) * 16) as f32 * 1.5;
        }
        if let Some(ref mut fog) = fog {
            fog.falloff = FogFalloff::Linear {
                start: (view_dist * 16) as f32 * 0.70,
                end: (view_dist * 16) as f32 * 0.95,
            };
        }
    }

    if player_chunk != world.last_player_chunk || settings_changed || dev_changed {
        world.last_player_chunk = player_chunk;

        let mut needed_chunks = Vec::new();
        for dx in -gen_dist..=gen_dist {
            for dz in -gen_dist..=gen_dist {
                let coord = player_chunk + IVec2::new(dx, dz);
                // Chunk needs to be generated if it is not already loaded or in progress
                if !world.chunks.contains_key(&coord) && !world.in_progress_chunks.contains(&coord)
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

        // 2-Tier streaming lifecycle:
        // Tier 1 (within visual view_dist): ensure mesh is queued if missing
        // Tier 2 (outside view_dist): despawn GPU mesh to save draw calls & VRAM, keep voxels in RAM
        let mut chunks_to_queue = Vec::new();
        let mut chunks_to_demesh = Vec::new();

        for (&coord, _) in &world.chunks {
            let diff = coord - player_chunk;
            if diff.x.abs() <= view_dist && diff.y.abs() <= view_dist {
                if !world.chunk_entities.contains_key(&coord) {
                    chunks_to_queue.push(coord);
                }
            } else if world.chunk_entities.contains_key(&coord) {
                chunks_to_demesh.push(coord);
            }
        }

        for coord in chunks_to_queue {
            world.queue_mesh(coord);
        }

        for coord in chunks_to_demesh {
            if let Some(entity) = world.chunk_entities.remove(&coord) {
                commands.entity(entity).despawn();
            }
            if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                world.total_vertices = world.total_vertices.saturating_sub(old_v);
            }
            world.chunk_lod.remove(&coord);
            world.queued_for_mesh.remove(&coord);
            world.in_progress_meshes.remove(&coord);
        }

        let mut chunks_to_remove = Vec::new();
        for coord in world.chunks.keys() {
            let diff = *coord - player_chunk;
            if diff.x.abs() > unload_dist || diff.y.abs() > unload_dist {
                chunks_to_remove.push(*coord);
            }
        }

        for coord in chunks_to_remove {
            world.queued_for_mesh.remove(&coord);
            world.in_progress_meshes.remove(&coord);
            world.chunk_lod.remove(&coord);
            if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                world.total_vertices = world.total_vertices.saturating_sub(old_v);
            }
            // Despawn 3D mesh entity from GPU
            if let Some(entity) = world.chunk_entities.remove(&coord) {
                commands.entity(entity).despawn();
            }

            // If the chunk was modified by the player, persist it to disk
            if world.dirty_chunks.remove(&coord) || world.modified_chunks.contains(&coord) {
                if let Err(e) = world.save_chunk_to_disk(coord) {
                    tracing::warn!("Failed to save chunk at {coord:?}: {e}");
                }
            }
            if let Some(removed_chunk) = world.chunks.remove(&coord) {
                // Keep recently unloaded chunks in RAM LRU cache to prevent thrashing
                world.chunk_cache.insert(coord, removed_chunk);
            }
        }
    }

    // 1. Dispatch background chunk generation tasks across all CPU cores
    let mut dispatched = 0;
    while dispatched < MAX_CHUNK_DISPATCH_PER_FRAME && !world.generation_queue.is_empty() {
        let Some(coord) = world.generation_queue.pop() else {
            break;
        };
        if world.chunks.contains_key(&coord) || world.in_progress_chunks.contains(&coord) {
            continue;
        }

        // Fast path: recover from in-memory LRU cache if player turned back into this chunk
        if let Some(cached_chunk) = world.chunk_cache.get(&coord) {
            world.chunks.insert(coord, cached_chunk);
            let diff = coord - player_chunk;
            if diff.x.abs() <= view_dist && diff.y.abs() <= view_dist {
                world.queue_mesh(coord);
            }
            continue;
        }

        world.in_progress_chunks.insert(coord);

        let tx = pools.generator.tx.clone();
        let noise = world.noise.clone();
        let seed = world.seed.0;
        let save_dir = world.save_dir.clone();

        AsyncComputeTaskPool::get()
            .spawn(async move {
                // Check disk cache first
                match WorldGrid::load_chunk_from_disk_path(&save_dir, coord) {
                    Ok(loaded_chunk) => {
                        let _ = tx.send((coord, loaded_chunk, true));
                        return;
                    }
                    Err(WorldError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
                        // Expected: chunk not saved yet, proceed to procedural generation
                    }
                    Err(e) => {
                        tracing::warn!("Disk cache error for chunk {coord:?}: {e}");
                    }
                }

                // Procedural generation in parallel on thread pool
                let chunk = generate_chunk(coord.x, coord.y, &noise, seed);
                let _ = tx.send((coord, chunk, false));
            })
            .detach();

        dispatched += 1;
    }

    // 2. Receive finished chunks from background threads and queue for meshing
    if let Ok(rx) = pools.generator.rx.lock() {
        while let Ok((coord, chunk, from_disk)) = rx.try_recv() {
            world.in_progress_chunks.remove(&coord);

            let diff = coord - player_chunk;
            if diff.x.abs() > unload_dist || diff.y.abs() > unload_dist {
                continue;
            }

            if from_disk {
                world.modified_chunks.insert(coord);
            }
            world.chunks.insert(coord, chunk);

            // Tier 1 meshing: only queue mesh if chunk is within visual view distance
            if diff.x.abs() <= view_dist && diff.y.abs() <= view_dist {
                world.queue_mesh(coord);
            }

            // Only queue neighbor chunks if they are within visual range and already have an active GPU mesh that needs seam update
            for neighbor_coord in [
                coord + IVec2::new(-1, 0),
                coord + IVec2::new(1, 0),
                coord + IVec2::new(0, -1),
                coord + IVec2::new(0, 1),
            ] {
                let n_diff = neighbor_coord - player_chunk;
                if n_diff.x.abs() <= view_dist
                    && n_diff.y.abs() <= view_dist
                    && world.chunk_entities.contains_key(&neighbor_coord)
                {
                    world.queue_mesh(neighbor_coord);
                }
            }
        }
    }

    // 3. Receive finished asynchronous meshes from background threads
    if let Ok(rx) = pools.mesher.rx.lock() {
        while let Ok((coord, mesh, lod)) = rx.try_recv() {
            world.in_progress_meshes.remove(&coord);

            // If chunk was unloaded while meshing, ignore
            if !world.chunks.contains_key(&coord) {
                continue;
            }

            apply_chunk_mesh(
                coord,
                mesh,
                lod,
                &mut commands,
                &mut world,
                &mut assets.meshes,
                &mut assets.materials,
            );
        }
    }

    let distance_lod = settings.dev.as_ref().is_none_or(|d| d.distance_lod);
    let lod_threshold = settings.dev.as_ref().map_or(4, |d| d.lod_threshold);
    let global_greedy = settings.dev.as_ref().is_none_or(|d| d.greedy_meshing);

    let player_pos = cam_transform.translation;
    let threshold_world = (lod_threshold as f32) * 16.0;
    let threshold_sq = threshold_world * threshold_world;

    // Dynamic 3D LOD transitions: check if any active chunks need to change LOD as player moves in 3D (including vertical flight)
    let mut chunks_needing_lod_update = Vec::new();
    for (&coord, chunk) in &world.chunks {
        let diff = coord - player_chunk;
        let dist_2d = diff.x.abs().max(diff.y.abs());
        if dist_2d <= view_dist && world.chunk_entities.contains_key(&coord) {
            let dist_sq = chunk_distance_sq_to_player(coord, player_pos, Some(chunk));
            let target_lod = if distance_lod {
                u8::from(dist_sq > threshold_sq)
            } else {
                u8::from(global_greedy)
            };

            if world.chunk_lod.get(&coord) != Some(&target_lod)
                && !world.queued_for_mesh.contains(&coord)
                && !world.in_progress_meshes.contains(&coord)
            {
                chunks_needing_lod_update.push(coord);
            }
        }
    }

    for coord in chunks_needing_lod_update {
        world.queue_mesh(coord);
    }

    // 4. Process mesh queue with frame budget, asynchronous dispatch, and 3D distance priority
    if !world.mesh_queue.is_empty() {
        let world_ref = &mut *world;
        let chunks = &world_ref.chunks;
        world_ref.mesh_queue.sort_unstable_by_key(|c| {
            let chunk_opt = chunks.get(c);
            let d_sq = chunk_distance_sq_to_player(*c, player_pos, chunk_opt);
            -(d_sq as i64)
        });

        let max_y_skip = settings.dev.as_ref().is_none_or(|d| d.max_y_skip);
        let budget_enabled = settings.dev.as_ref().is_none_or(|d| d.mesh_budget);
        let async_meshing = settings.dev.as_ref().is_none_or(|d| d.async_meshing);
        let max_meshes_per_frame = if budget_enabled {
            MAX_MESHES_PER_FRAME
        } else {
            usize::MAX
        };

        let mut meshed = 0;
        while meshed < max_meshes_per_frame && !world.mesh_queue.is_empty() {
            let Some(coord) = world.mesh_queue.pop() else {
                break;
            };
            world.queued_for_mesh.remove(&coord);

            if let Some(chunk) = world.chunks.get(&coord) {
                let diff = coord - player_chunk;
                let dist_2d = diff.x.abs().max(diff.y.abs());
                if dist_2d <= view_dist {
                    let dist_sq = chunk_distance_sq_to_player(coord, player_pos, Some(chunk));
                    let target_lod = if distance_lod {
                        u8::from(dist_sq > threshold_sq)
                    } else {
                        u8::from(global_greedy)
                    };
                    let use_greedy = target_lod == 1;

                    if async_meshing {
                        if world.in_progress_meshes.contains(&coord) {
                            continue;
                        }
                        let chunk = chunk.clone();
                        world.in_progress_meshes.insert(coord);
                        let north = world.chunks.get(&(coord + IVec2::new(0, 1))).cloned();
                        let south = world.chunks.get(&(coord + IVec2::new(0, -1))).cloned();
                        let east = world.chunks.get(&(coord + IVec2::new(1, 0))).cloned();
                        let west = world.chunks.get(&(coord + IVec2::new(-1, 0))).cloned();

                        let tx = pools.mesher.tx.clone();
                        AsyncComputeTaskPool::get()
                            .spawn(async move {
                                let mesh = build_chunk_mesh(
                                    &chunk,
                                    north.as_ref(),
                                    south.as_ref(),
                                    east.as_ref(),
                                    west.as_ref(),
                                    max_y_skip,
                                    use_greedy,
                                );
                                let _ = tx.send((coord, mesh, target_lod));
                            })
                            .detach();
                    } else {
                        update_chunk_mesh(
                            &coord,
                            &mut commands,
                            &mut world,
                            &mut assets.meshes,
                            &mut assets.materials,
                            max_y_skip,
                            use_greedy,
                        );
                    }
                    meshed += 1;
                }
            }
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGeneratorPool>()
            .init_resource::<ChunkMesherPool>()
            .add_systems(
                Update,
                world_streaming_system.in_set(crate::stage::VoxelStage::WorldStreaming),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::LocalBlockPos;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_chunk_3d_distance_and_vertical_flight() {
        let coord = IVec2::new(0, 0); // bounds: [0..16, 0..16]
        let mut chunk = Chunk::new();
        chunk.max_y = 70; // terrain reaches up to y=70

        // 1. Standing directly on the terrain
        let player_ground = Vec3::new(8.0, 71.0, 8.0);
        let d_sq_ground = chunk_distance_sq_to_player(coord, player_ground, Some(&chunk));
        assert_eq!(d_sq_ground, 0.0);

        // 2. Flying high vertically in creative mode (e.g. y = 200)
        let player_high = Vec3::new(8.0, 200.0, 8.0);
        let d_sq_high = chunk_distance_sq_to_player(coord, player_high, Some(&chunk));
        let expected_dy = 200.0 - 71.0; // 129.0
        assert_eq!(d_sq_high, expected_dy * expected_dy);

        // Threshold of 4 chunks = 64 meters (threshold_sq = 4096)
        let threshold_sq = (4.0 * 16.0_f32).powi(2); // 4096.0
        assert!(
            d_sq_ground <= threshold_sq,
            "Ground chunk should be detailed (LOD 0)"
        );
        assert!(
            d_sq_high > threshold_sq,
            "Chunk viewed from high altitude should compress to Greedy Mesh (LOD 1)"
        );
    }

    #[test]
    fn test_lru_chunk_cache_insertion_and_retrieval() {
        let world = WorldGrid::default();
        let coord = IVec2::new(42, -99);
        let mut chunk = Chunk::new();
        chunk.set_local(LocalBlockPos::new(1, 2, 3), BlockType::DiamondOre);

        assert!(world.chunk_cache.get(&coord).is_none());

        world.chunk_cache.insert(coord, chunk);

        let retrieved = world
            .chunk_cache
            .get(&coord)
            .expect("chunk should be in cache");
        assert_eq!(
            retrieved.get_local(LocalBlockPos::new(1, 2, 3)),
            BlockType::DiamondOre
        );
    }

    #[test]
    fn test_fused_chunk_generation_strata_and_bedrock() {
        let noise = NoiseGenerator::new(42);
        let chunk = generate_chunk(0, 0, &noise, 42);

        // Bedrock is guaranteed at layer 0 across all columns
        for lx in 0..CHUNK_WIDTH {
            for lz in 0..CHUNK_DEPTH {
                assert_eq!(chunk.get_fast(lx, 0, lz), BlockType::Bedrock);
            }
        }

        // Chunk max_y must reflect populated terrain height
        assert!(chunk.max_y >= 5, "Terrain height must be at least 5");
        assert!(
            chunk.max_y < CHUNK_HEIGHT,
            "Terrain height must fit within CHUNK_HEIGHT"
        );
    }

    #[test]
    fn test_lookahead_buffer_tier_radii() {
        let view_dist = 16;
        let pregen_margin = 2;
        let gen_dist = view_dist + pregen_margin; // 18
        let unload_dist = gen_dist + 2; // 20

        // Visual Tier 1: radius 16 chunks
        let tier1_chunk = IVec2::new(16, 0);
        assert!(tier1_chunk.x.abs() <= view_dist && tier1_chunk.y.abs() <= view_dist);

        // Lookahead Tier 2 (pre-generated RAM buffer, unmeshed): radius 17..18 chunks
        let tier2_chunk = IVec2::new(17, 1);
        let diff_2d = tier2_chunk.x.abs().max(tier2_chunk.y.abs());
        assert!(
            diff_2d > view_dist,
            "Tier 2 chunk must be outside visual view distance"
        );
        assert!(
            diff_2d <= gen_dist,
            "Tier 2 chunk must be inside lookahead generation radius"
        );
        assert!(diff_2d <= unload_dist, "Tier 2 chunk must not be unloaded");

        // Far chunk: radius 21 chunks
        let far_chunk = IVec2::new(21, 0);
        let far_diff = far_chunk.x.abs().max(far_chunk.y.abs());
        assert!(
            far_diff > unload_dist,
            "Far chunk must exceed unload threshold"
        );
    }

    #[test]
    fn test_world_seed_random_and_alphanumeric() {
        let seed1 = WorldSeed::random();
        let seed2 = WorldSeed::random();

        // Valid positive numbers
        assert!(seed1.0 > 0);
        assert!(seed2.0 > 0);

        // Numeric string parsing
        let num_seed = WorldSeed::from_seed_str("987654321");
        assert_eq!(num_seed.0, 987654321);

        // Alphanumeric string parsing (deterministic)
        let alpha1 = WorldSeed::from_seed_str("minecraft");
        let alpha2 = WorldSeed::from_seed_str("minecraft");
        let alpha3 = WorldSeed::from_seed_str("custom_seed_42");

        assert_eq!(alpha1, alpha2);
        assert_ne!(alpha1, alpha3);
        assert_ne!(alpha1.0, 0);
    }
}
