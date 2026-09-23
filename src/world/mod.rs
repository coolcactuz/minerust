//! Voxel world management, procedural terrain generation, chunk storage,
//! and background multi-threaded mesh/generation streaming.

pub mod cluster;
pub mod grid;
pub mod spawn;
pub mod streaming;
pub mod terrain;
pub mod types;

#[cfg(test)]
mod tests;

pub use cluster::*;
pub use grid::WorldGrid;
pub use spawn::find_safe_surface_spawn;
pub use streaming::{
    ChunkGeneratorPool, ChunkMesherPool, WorldMeshAssets, WorldPlugin, WorldSettingsParams,
    WorldWorkerPools, apply_chunk_mesh, chunk_distance_sq_to_player, determine_chunk_tier,
    update_chunk_mesh, world_streaming_system,
};
pub use terrain::{calculate_biome_and_height, generate_chunk, lerp, smoothstep};
pub use types::{
    BiomeType, CHUNK_CACHE_CAPACITY, MAX_CHUNK_DISPATCH_PER_FRAME, MAX_MESHES_PER_FRAME, SEA_LEVEL,
    VIEW_DISTANCE, WorldSeed,
};
