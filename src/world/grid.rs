use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use std::path::PathBuf;

use crate::block::BlockType;
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::coords::{BlockPos, ChunkPos};
use crate::error::WorldError;
use crate::noise::NoiseGenerator;
use crate::voxel_material::VoxelBlockMaterial;
use crate::world::streaming::{chunk_distance_sq_to_player, determine_chunk_tier, update_chunk_mesh};
use crate::world::terrain::generate_chunk;
use crate::world::types::{CHUNK_CACHE_CAPACITY, WorldSeed};

#[derive(Resource)]
pub struct WorldGrid {
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_cache: quick_cache::sync::Cache<IVec2, Chunk>,
    pub chunk_entities: HashMap<IVec2, Entity>,
    pub water_entities: HashMap<IVec2, Entity>,
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
    pub block_material: Option<Handle<VoxelBlockMaterial>>,
    pub water_material: Option<Handle<VoxelBlockMaterial>>,
    pub total_vertices: usize,
    pub chunk_vertices: HashMap<IVec2, usize>,
    pub chunk_lod: HashMap<IVec2, u8>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        Self::new(WorldSeed::default())
    }
}

impl WorldGrid {
    pub fn new(seed: WorldSeed) -> Self {
        let noise = NoiseGenerator::new(seed.0);
        Self {
            chunks: HashMap::default(),
            chunk_cache: quick_cache::sync::Cache::new(CHUNK_CACHE_CAPACITY),
            chunk_entities: HashMap::default(),
            water_entities: HashMap::default(),
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
            water_material: None,
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

    /// Despawns all active chunk meshes and clears loaded chunks and internal queues.
    pub fn despawn_all_chunks(&mut self, commands: &mut Commands) {
        for (_, entity) in self.chunk_entities.drain() {
            commands.entity(entity).despawn();
        }
        for (_, entity) in self.water_entities.drain() {
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
    }

    /// Despawns all active chunk meshes and resets world state to switch to a new seed.
    pub fn reinitialize_with_seed(&mut self, new_seed: WorldSeed, commands: &mut Commands) {
        self.despawn_all_chunks(commands);
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
        materials: &mut Assets<VoxelBlockMaterial>,
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

        let center_world_x = center_chunk.x as f32 * 16.0 + 8.0;
        let center_world_z = center_chunk.y as f32 * 16.0 + 8.0;
        let player_pos = Vec3::new(center_world_x, 90.0, center_world_z);

        for coord in &initial_coords {
            let chunk_opt = self.chunks.get(coord);
            let dist_sq = chunk_distance_sq_to_player(*coord, player_pos, chunk_opt);
            let (tier, _, _) = determine_chunk_tier(dist_sq);
            update_chunk_mesh(coord, commands, self, meshes, materials, true, tier);
        }
    }
}
