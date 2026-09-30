use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoAutoAabb;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::camera::FpsCamera;
use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};
use crate::error::WorldError;
use crate::menu::GraphicsSettings;
use crate::mesher::{CHUNK_SECTIONS, ChunkMeshes, build_chunk_mesh_lod};
use crate::voxel_material::{VoxelBlockMaterial, VoxelExtension};
use crate::world::frontier::{get_entered_chunks, get_exited_chunks};
use crate::world::grid::{ChunkSection, WorldGrid};
use crate::world::terrain::generate_chunk;
use crate::world::types::{
    MAX_CHUNK_DISPATCH_PER_FRAME, MAX_MESHES_PER_FRAME, SEA_LEVEL, VIEW_DISTANCE,
};

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
    pub tx: Sender<(IVec2, ChunkMeshes, u8)>,
    pub rx: Mutex<Receiver<(IVec2, ChunkMeshes, u8)>>,
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

pub const TIER_STANDARD_VOXEL: u8 = 0;
pub const TIER_GREEDY_VOXEL: u8 = 1;
pub const TIER_SLOPED_LOD: u8 = 2;

/// Applies or despawns chunk meshes (solid terrain and water) on the GPU
pub fn apply_chunk_mesh(
    coord: IVec2,
    meshes_res: ChunkMeshes,
    tier: u8,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelBlockMaterial>,
) {
    let world_pos = Vec3::new(
        (coord.x * CHUNK_WIDTH as i32) as f32,
        0.0,
        (coord.y * CHUNK_DEPTH as i32) as f32,
    );

    // Track vertex counts, LOD and connectivity
    world.chunk_lod.insert(coord, tier);
    world
        .chunk_connectivity
        .insert(coord, meshes_res.connectivity);
    let new_vert_count = meshes_res.total_vertices();
    let old_vert_count = world
        .chunk_vertices
        .insert(coord, new_vert_count)
        .unwrap_or(0);
    world.total_vertices = world.total_vertices.saturating_sub(old_vert_count) + new_vert_count;

    if tier >= TIER_SLOPED_LOD {
        // Despawn any existing LOD 0 sub-chunk entities if transitioning from near LOD 0
        if let Some(solid_entities) = world.chunk_entities.remove(&coord) {
            for entity in solid_entities.into_iter().flatten() {
                commands.entity(entity).despawn();
            }
        }
        if let Some(water_entities) = world.water_entities.remove(&coord) {
            for entity in water_entities.into_iter().flatten() {
                commands.entity(entity).despawn();
            }
        }

        // Insert into 4x4 macro-chunk cluster for consolidated rendering
        let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
        let macro_chunk = world.macro_chunks.entry(macro_coord).or_default();
        let s0 = meshes_res.sections.into_iter().next().unwrap_or_default();
        macro_chunk.chunks[slot] = Some(s0);
        macro_chunk.dirty = true;
        world.dirty_macro_chunks.insert(macro_coord);
        return;
    }

    // LOD 0 (tier 0 or 1): remove from macro-chunk if transitioning from distant LOD 2
    let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
    if let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) {
        if macro_chunk.chunks[slot].take().is_some() {
            macro_chunk.dirty = true;
            world.dirty_macro_chunks.insert(macro_coord);
        }
    }

    let solid_material = world.block_material.clone().unwrap_or_else(|| {
        materials.add(ExtendedMaterial {
            base: StandardMaterial {
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                perceptual_roughness: 0.85,
                reflectance: 0.15,
                ..default()
            },
            extension: VoxelExtension {
                array_texture: Handle::default(),
            },
        })
    });

    let water_material = world.water_material.clone().unwrap_or_else(|| {
        materials.add(ExtendedMaterial {
            base: StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                perceptual_roughness: 0.08,
                reflectance: 0.5,
                ..default()
            },
            extension: VoxelExtension {
                array_texture: Handle::default(),
            },
        })
    });

    // Get existing section entity arrays or create empty ones
    let mut solid_entities = world
        .chunk_entities
        .remove(&coord)
        .unwrap_or([None; CHUNK_SECTIONS]);
    let mut water_entities = world
        .water_entities
        .remove(&coord)
        .unwrap_or([None; CHUNK_SECTIONS]);

    for (sy, section_mesh) in meshes_res.sections.into_iter().enumerate() {
        let section_y = sy as u8;
        let aabb = Aabb::from_min_max(
            Vec3::new(0.0, (sy * 16) as f32, 0.0),
            Vec3::new(16.0, ((sy + 1) * 16) as f32, 16.0),
        );

        // Solid terrain mesh entity
        if let Some(entity) = solid_entities[sy] {
            if let Some(mesh) = section_mesh.solid {
                commands
                    .entity(entity)
                    .insert((Mesh3d(meshes.add(mesh)), aabb, NoAutoAabb, Visibility::Inherited));
            } else {
                commands.entity(entity).despawn();
                solid_entities[sy] = None;
            }
        } else if let Some(mesh) = section_mesh.solid {
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(solid_material.clone()),
                    Transform::from_translation(world_pos),
                    aabb,
                    NoAutoAabb,
                    ChunkSection {
                        chunk: coord,
                        section_y,
                    },
                ))
                .id();
            solid_entities[sy] = Some(entity);
        }

        // Water surface mesh entity
        if let Some(entity) = water_entities[sy] {
            if let Some(mesh) = section_mesh.water {
                commands
                    .entity(entity)
                    .insert((Mesh3d(meshes.add(mesh)), aabb, NoAutoAabb, Visibility::Inherited));
            } else {
                commands.entity(entity).despawn();
                water_entities[sy] = None;
            }
        } else if let Some(mesh) = section_mesh.water {
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(water_material.clone()),
                    Transform::from_translation(world_pos),
                    NotShadowCaster,
                    aabb,
                    NoAutoAabb,
                    ChunkSection {
                        chunk: coord,
                        section_y,
                    },
                ))
                .id();
            water_entities[sy] = Some(entity);
        }
    }

    if solid_entities.iter().any(Option::is_some) {
        world.chunk_entities.insert(coord, solid_entities);
    }
    if water_entities.iter().any(Option::is_some) {
        world.water_entities.insert(coord, water_entities);
    }
}

pub const GREEDY_THRESHOLD_WORLD: f32 = 32.0; // 2 chunks = 32m
pub const GREEDY_THRESHOLD_SQ: f32 = GREEDY_THRESHOLD_WORLD * GREEDY_THRESHOLD_WORLD; // 1024.0
pub const LOD_THRESHOLD_WORLD: f32 = 128.0; // 8 chunks = 128m
pub const LOD_THRESHOLD_SQ: f32 = LOD_THRESHOLD_WORLD * LOD_THRESHOLD_WORLD; // 16384.0

/// Determines the chunk mesh tier and meshing parameters based on distance to player and user graphics settings.
/// - Tier 0: Standard 1x1 Voxel Meshing
/// - Tier 1: Greedy Voxel Meshing
/// - Tier 2: Sloped Heightfield LOD
#[inline]
pub fn determine_chunk_tier(
    dist_sq: f32,
    distance_lod: bool,
    lod_threshold_sq: f32,
    greedy_meshing: bool,
    greedy_threshold: i32,
    greedy_threshold_sq: f32,
) -> (u8, bool, u8) {
    if distance_lod && dist_sq > lod_threshold_sq {
        (2, false, 1) // Tier 2: Sloped Heightfield LOD
    } else if greedy_meshing && (greedy_threshold <= 0 || dist_sq >= greedy_threshold_sq) {
        (1, true, 0) // Tier 1: Greedy Voxel Meshing
    } else {
        (0, false, 0) // Tier 0: Standard 1x1 Voxel Meshing
    }
}

/// Computes the maximum chunk radius around the player where LOD transitions between tiers can occur.
/// Any chunk farther than this radius is mathematically guaranteed to remain in Tier 2 (Sloped LOD).
#[inline]
pub fn calculate_lod_scan_radius(
    distance_lod: bool,
    lod_threshold_sq: f32,
    greedy_meshing: bool,
    greedy_threshold_sq: f32,
    view_dist: i32,
) -> i32 {
    let max_threshold_sq = if distance_lod {
        lod_threshold_sq
    } else if greedy_meshing {
        greedy_threshold_sq
    } else {
        0.0
    };
    if max_threshold_sq <= 0.0 {
        return 0;
    }
    let threshold_world = max_threshold_sq.sqrt();
    let threshold_chunks = (threshold_world / 16.0).ceil() as i32;
    // Add safety margin of 2 chunks to account for sub-chunk player offset within chunk bounds
    (threshold_chunks + 2).min(view_dist)
}

/// Updates or creates the mesh for the specified chunk synchronously
pub fn update_chunk_mesh(
    coord: &IVec2,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelBlockMaterial>,
    max_y_skip: bool,
    tier: u8,
) {
    let Some(chunk) = world.chunks.get(coord) else {
        return;
    };

    let north = world.chunks.get(&(*coord + IVec2::new(0, 1)));
    let south = world.chunks.get(&(*coord + IVec2::new(0, -1)));
    let east = world.chunks.get(&(*coord + IVec2::new(1, 0)));
    let west = world.chunks.get(&(*coord + IVec2::new(-1, 0)));

    let (greedy, lod) = match tier {
        2 => (false, 1),
        1 => (true, 0),
        _ => (false, 0),
    };
    let new_mesh = build_chunk_mesh_lod(chunk, north, south, east, west, max_y_skip, greedy, lod);
    apply_chunk_mesh(*coord, new_mesh, tier, commands, world, meshes, materials);
}

#[derive(SystemParam)]
pub struct WorldSettingsParams<'w> {
    pub graphics: Option<Res<'w, GraphicsSettings>>,
    pub menu: Option<Res<'w, crate::menu::MenuState>>,
    pub bench_state: Option<Res<'w, crate::benchmark::BenchmarkState>>,
}

#[derive(SystemParam)]
pub struct WorldWorkerPools<'w> {
    pub generator: Res<'w, ChunkGeneratorPool>,
    pub mesher: Res<'w, ChunkMesherPool>,
}

#[derive(SystemParam)]
pub struct WorldMeshAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<VoxelBlockMaterial>>,
}

/// Continuous chunk streaming system based on player camera position with multithreaded generation
pub fn world_streaming_system(
    mut commands: Commands,
    mut camera_query: Query<(&Transform, &mut Projection), With<FpsCamera>>,
    mut world: ResMut<WorldGrid>,
    mut assets: WorldMeshAssets,
    settings: WorldSettingsParams,
    pools: WorldWorkerPools,
) {
    if settings
        .menu
        .as_ref()
        .is_some_and(|m| m.screen == crate::menu::MenuScreen::Main || !m.world_active)
    {
        return;
    }

    let Ok((cam_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let px = cam_transform.translation.x as i32;
    let pz = cam_transform.translation.z as i32;
    let (player_chunk, _, _) = WorldGrid::world_to_chunk_coord(px, pz);

    let view_dist = settings
        .graphics
        .as_ref()
        .map_or(VIEW_DISTANCE, |s| s.view_distance);
    let pregen_margin = 2;
    let gen_dist = view_dist + pregen_margin;
    let unload_dist = gen_dist + 2;

    let settings_changed = settings.graphics.as_ref().is_some_and(|s| s.is_changed());

    if settings_changed {
        if let Projection::Perspective(ref mut persp) = *projection {
            persp.far = ((view_dist + 4) * 16) as f32 * 1.5;
        }
    }

    if player_chunk != world.last_player_chunk || settings_changed {
        let prev_player_chunk = world.last_player_chunk;
        world.last_player_chunk = player_chunk;

        let delta = player_chunk - prev_player_chunk;
        let is_initial_or_teleport = settings_changed || delta.x.abs() > 4 || delta.y.abs() > 4;

        let (chunks_to_queue, chunks_to_demesh, chunks_to_remove) = if is_initial_or_teleport {
            let mut needed_chunks = Vec::new();
            for dx in -gen_dist..=gen_dist {
                for dz in -gen_dist..=gen_dist {
                    let coord = player_chunk + IVec2::new(dx, dz);
                    // Chunk needs to be generated if it is not already loaded or in progress
                    if !world.chunks.contains_key(&coord)
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

            let mut to_queue = Vec::new();
            let mut to_demesh = Vec::new();
            let mut to_remove = Vec::new();

            for &coord in world.chunks.keys() {
                let diff = coord - player_chunk;
                let dx = diff.x.abs();
                let dz = diff.y.abs();

                if dx > unload_dist || dz > unload_dist {
                    to_remove.push(coord);
                } else if dx <= view_dist && dz <= view_dist {
                    if !world.chunk_lod.contains_key(&coord) {
                        to_queue.push(coord);
                    }
                } else if world.chunk_lod.contains_key(&coord) {
                    to_demesh.push(coord);
                }
            }

            (to_queue, to_demesh, to_remove)
        } else {
            // Incremental frontier scanning: only scan newly entered / exited boundaries
            let entered_gen = get_entered_chunks(prev_player_chunk, player_chunk, gen_dist);
            let mut newly_needed = Vec::new();
            for coord in entered_gen {
                if !world.chunks.contains_key(&coord)
                    && !world.in_progress_chunks.contains(&coord)
                {
                    newly_needed.push(coord);
                }
            }

            // Prune any chunks in generation_queue that are now outside gen_dist
            world.generation_queue.retain(|c| {
                let diff = *c - player_chunk;
                diff.x.abs() <= gen_dist && diff.y.abs() <= gen_dist
            });

            newly_needed.sort_by_key(|c| {
                let diff = *c - player_chunk;
                -(diff.x * diff.x + diff.y * diff.y)
            });

            // Prepend new frontier chunks to the front of generation_queue
            // so closer chunks remain at the back and are popped first
            let mut new_queue = newly_needed;
            new_queue.append(&mut world.generation_queue);
            world.generation_queue = new_queue;

            let entered_view = get_entered_chunks(prev_player_chunk, player_chunk, view_dist);
            let mut to_queue = Vec::new();
            for coord in entered_view {
                if world.chunks.contains_key(&coord) && !world.chunk_lod.contains_key(&coord) {
                    to_queue.push(coord);
                }
            }

            let exited_view = get_exited_chunks(prev_player_chunk, player_chunk, view_dist);
            let mut to_demesh = Vec::new();
            for coord in exited_view {
                if world.chunk_lod.contains_key(&coord) {
                    to_demesh.push(coord);
                }
            }

            let exited_unload = get_exited_chunks(prev_player_chunk, player_chunk, unload_dist);
            let mut to_remove = Vec::new();
            for coord in exited_unload {
                if world.chunks.contains_key(&coord) {
                    to_remove.push(coord);
                }
            }

            (to_queue, to_demesh, to_remove)
        };

        for coord in chunks_to_queue {
            world.queue_mesh(coord);
        }

        for coord in chunks_to_demesh {
            let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
            if let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) {
                macro_chunk.chunks[slot] = None;
                if !macro_chunk.has_any_chunk() {
                    if let Some(e) = macro_chunk.solid_entity.take() {
                        commands.entity(e).despawn();
                    }
                    if let Some(e) = macro_chunk.water_entity.take() {
                        commands.entity(e).despawn();
                    }
                    world.dirty_macro_chunks.remove(&macro_coord);
                }
            }
            if let Some(entities) = world.chunk_entities.remove(&coord) {
                for entity in entities.into_iter().flatten() {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(entities) = world.water_entities.remove(&coord) {
                for entity in entities.into_iter().flatten() {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                world.total_vertices = world.total_vertices.saturating_sub(old_v);
            }
            world.chunk_lod.remove(&coord);
            world.chunk_connectivity.remove(&coord);
            world.queued_for_mesh.remove(&coord);
            world.in_progress_meshes.remove(&coord);
        }

        // Prune stale mesh queue entries that are outside the current visual distance
        world.mesh_queue.retain(|c| {
            let diff = *c - player_chunk;
            diff.x.abs() <= view_dist && diff.y.abs() <= view_dist
        });
        world.queued_for_mesh.retain(|c| {
            let diff = *c - player_chunk;
            diff.x.abs() <= view_dist && diff.y.abs() <= view_dist
        });

        for coord in chunks_to_remove {
            world.queued_for_mesh.remove(&coord);
            world.in_progress_meshes.remove(&coord);
            world.chunk_lod.remove(&coord);
            world.chunk_connectivity.remove(&coord);
            if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                world.total_vertices = world.total_vertices.saturating_sub(old_v);
            }
            let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
            if let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) {
                macro_chunk.chunks[slot] = None;
                if !macro_chunk.has_any_chunk() {
                    if let Some(e) = macro_chunk.solid_entity.take() {
                        commands.entity(e).despawn();
                    }
                    if let Some(e) = macro_chunk.water_entity.take() {
                        commands.entity(e).despawn();
                    }
                    world.dirty_macro_chunks.remove(&macro_coord);
                }
            }
            // Despawn 3D mesh entities from GPU
            if let Some(entities) = world.chunk_entities.remove(&coord) {
                for entity in entities.into_iter().flatten() {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(entities) = world.water_entities.remove(&coord) {
                for entity in entities.into_iter().flatten() {
                    commands.entity(entity).despawn();
                }
            }

            // If the chunk was modified by the player, persist it to disk
            let was_modified =
                world.dirty_chunks.remove(&coord) || world.modified_chunks.remove(&coord);
            if was_modified {
                if let Err(e) = world.save_chunk_to_disk(coord) {
                    tracing::warn!("Failed to save chunk at {coord:?}: {e}");
                    world.modified_chunks.insert(coord);
                }
            }
            if let Some(removed_chunk) = world.chunks.remove(&coord) {
                // Keep recently unloaded chunks in RAM LRU cache to prevent thrashing
                world.chunk_cache.insert(coord, removed_chunk);
            }
        }
    }

    // 1. Dispatch background chunk generation tasks across all CPU cores
    let is_bench_initializing = settings
        .bench_state
        .as_ref()
        .is_some_and(|b| b.phase == crate::benchmark::BenchmarkPhase::InitializingWorld);
    let max_dispatch = if is_bench_initializing {
        64
    } else {
        MAX_CHUNK_DISPATCH_PER_FRAME
    };
    let max_in_flight_chunks = if is_bench_initializing { 128 } else { 32 };

    let mut dispatched = 0;
    while dispatched < max_dispatch
        && world.in_progress_chunks.len() < max_in_flight_chunks
        && !world.generation_queue.is_empty()
    {
        let Some(coord) = world.generation_queue.pop() else {
            break;
        };
        let diff = coord - player_chunk;
        if diff.x.abs() > gen_dist || diff.y.abs() > gen_dist {
            continue;
        }
        if world.chunks.contains_key(&coord) || world.in_progress_chunks.contains(&coord) {
            continue;
        }

        // Fast path: recover from in-memory LRU cache if player turned back into this chunk
        if let Some(cached_chunk) = world.chunk_cache.get(&coord) {
            world.chunks.insert(coord, cached_chunk);
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

            // Only queue neighbor chunks if they are within visual range and need seam updates (voxel tiers only).
            // Tier 2 (Sloped LOD) macro-chunks use skirts and do not cull block faces against neighbors;
            // skipping them avoids tens of thousands of redundant re-meshes and macro-chunk invalidations.
            for neighbor_coord in [
                coord + IVec2::new(-1, 0),
                coord + IVec2::new(1, 0),
                coord + IVec2::new(0, -1),
                coord + IVec2::new(0, 1),
            ] {
                let n_diff = neighbor_coord - player_chunk;
                if n_diff.x.abs() <= view_dist && n_diff.y.abs() <= view_dist {
                    let needs_voxel_seam_update = match world.chunk_lod.get(&neighbor_coord) {
                        Some(&t) => t < TIER_SLOPED_LOD,
                        None => world.has_chunk_mesh(&neighbor_coord),
                    };
                    if needs_voxel_seam_update {
                        world.queue_mesh(neighbor_coord);
                    }
                }
            }
        }
    }

    // 3. Receive finished asynchronous meshes from background threads
    if let Ok(rx) = pools.mesher.rx.lock() {
        while let Ok((coord, meshes_res, lod)) = rx.try_recv() {
            world.in_progress_meshes.remove(&coord);

            // If chunk was unloaded while meshing, ignore
            if !world.chunks.contains_key(&coord) {
                continue;
            }

            // Guard against race conditions / zombie entities:
            // If the player moved away and this chunk is now outside visual view distance,
            // discard the completed mesh immediately rather than spawning an off-screen entity.
            let diff = coord - player_chunk;
            if diff.x.abs() > view_dist || diff.y.abs() > view_dist {
                let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
                if let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) {
                    macro_chunk.chunks[slot] = None;
                    if !macro_chunk.has_any_chunk() {
                        if let Some(e) = macro_chunk.solid_entity.take() {
                            commands.entity(e).despawn();
                        }
                        if let Some(e) = macro_chunk.water_entity.take() {
                            commands.entity(e).despawn();
                        }
                        world.dirty_macro_chunks.remove(&macro_coord);
                    }
                }
                if let Some(entities) = world.chunk_entities.remove(&coord) {
                    for entity in entities.into_iter().flatten() {
                        commands.entity(entity).despawn();
                    }
                }
                if let Some(entities) = world.water_entities.remove(&coord) {
                    for entity in entities.into_iter().flatten() {
                        commands.entity(entity).despawn();
                    }
                }
                if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                    world.total_vertices = world.total_vertices.saturating_sub(old_v);
                }
                world.chunk_lod.remove(&coord);
                world.chunk_connectivity.remove(&coord);
                continue;
            }

            apply_chunk_mesh(
                coord,
                meshes_res,
                lod,
                &mut commands,
                &mut world,
                &mut assets.meshes,
                &mut assets.materials,
            );
        }
    }

    let player_pos = cam_transform.translation;

    let (distance_lod, lod_threshold_sq, greedy_meshing, greedy_threshold, greedy_threshold_sq) =
        if let Some(ref g) = settings.graphics {
            let l_sq = (g.lod_threshold as f32 * 16.0).powi(2);
            let g_sq = if g.greedy_threshold <= 0 {
                0.0
            } else {
                (g.greedy_threshold as f32 * 16.0).powi(2)
            };
            (
                g.distance_lod,
                l_sq,
                g.greedy_meshing,
                g.greedy_threshold,
                g_sq,
            )
        } else {
            (true, 128.0 * 128.0, true, 2, 32.0 * 32.0)
        };

    // Dynamic 3D LOD transitions: check if any active chunks need to change mesh tier as player moves in 3D.
    // Spatial throttling: only scan chunk LODs if graphics settings changed or player moved >= 2.0m (4.0m sq).
    let should_check_lod =
        settings_changed || player_pos.distance_squared(world.last_lod_player_pos) >= 4.0;

    if should_check_lod {
        world.last_lod_player_pos = player_pos;
        let mut chunks_needing_lod_update = Vec::new();

        if settings_changed {
            // Full sweep across loaded chunks when graphics settings change
            for (&coord, chunk) in &world.chunks {
                let diff = coord - player_chunk;
                let dist_2d = diff.x.abs().max(diff.y.abs());
                if dist_2d <= view_dist && world.chunk_lod.contains_key(&coord) {
                    let dist_sq = chunk_distance_sq_to_player(coord, player_pos, Some(chunk));
                    let (target_tier, _, _) = determine_chunk_tier(
                        dist_sq,
                        distance_lod,
                        lod_threshold_sq,
                        greedy_meshing,
                        greedy_threshold,
                        greedy_threshold_sq,
                    );

                    if world.chunk_lod.get(&coord) != Some(&target_tier)
                        && !world.queued_for_mesh.contains(&coord)
                        && !world.in_progress_meshes.contains(&coord)
                    {
                        chunks_needing_lod_update.push(coord);
                    }
                }
            }
        } else {
            // Candidate frontier sweep: only scan coordinates within reach of LOD tier transitions
            let scan_radius = calculate_lod_scan_radius(
                distance_lod,
                lod_threshold_sq,
                greedy_meshing,
                greedy_threshold_sq,
                view_dist,
            );
            for dz in -scan_radius..=scan_radius {
                for dx in -scan_radius..=scan_radius {
                    let coord = player_chunk + IVec2::new(dx, dz);
                    if let Some(chunk) = world.chunks.get(&coord) {
                        if world.chunk_lod.contains_key(&coord) {
                            let dist_sq =
                                chunk_distance_sq_to_player(coord, player_pos, Some(chunk));
                            let (target_tier, _, _) = determine_chunk_tier(
                                dist_sq,
                                distance_lod,
                                lod_threshold_sq,
                                greedy_meshing,
                                greedy_threshold,
                                greedy_threshold_sq,
                            );

                            if world.chunk_lod.get(&coord) != Some(&target_tier)
                                && !world.queued_for_mesh.contains(&coord)
                                && !world.in_progress_meshes.contains(&coord)
                            {
                                chunks_needing_lod_update.push(coord);
                            }
                        }
                    }
                }
            }
        }

        for coord in chunks_needing_lod_update {
            world.queue_mesh(coord);
        }
    }

    // 4. Process mesh queue with frame budget, asynchronous dispatch, and 3D distance priority
    if !world.mesh_queue.is_empty() {
        if world.mesh_queue.len() > 1 && world.mesh_queue_dirty {
            let world_ref = &mut *world;
            let chunks = &world_ref.chunks;
            world_ref.mesh_queue.sort_unstable_by(|a, b| {
                let d_a = chunk_distance_sq_to_player(*a, player_pos, chunks.get(a));
                let d_b = chunk_distance_sq_to_player(*b, player_pos, chunks.get(b));
                d_b.total_cmp(&d_a)
            });
            world.mesh_queue_dirty = false;
        }

        let max_meshes_per_frame = if is_bench_initializing {
            48
        } else {
            MAX_MESHES_PER_FRAME
        };
        let max_in_flight_meshes = if is_bench_initializing { 64 } else { 24 };

        let mut meshed = 0;
        while meshed < max_meshes_per_frame
            && world.in_progress_meshes.len() < max_in_flight_meshes
            && !world.mesh_queue.is_empty()
        {
            let Some(coord) = world.mesh_queue.pop() else {
                break;
            };
            world.queued_for_mesh.remove(&coord);

            if let Some(chunk) = world.chunks.get(&coord) {
                let diff = coord - player_chunk;
                let dist_2d = diff.x.abs().max(diff.y.abs());
                if dist_2d <= view_dist {
                    let dist_sq = chunk_distance_sq_to_player(coord, player_pos, Some(chunk));
                    let (target_tier, chunk_greedy, chunk_lod) = determine_chunk_tier(
                        dist_sq,
                        distance_lod,
                        lod_threshold_sq,
                        greedy_meshing,
                        greedy_threshold,
                        greedy_threshold_sq,
                    );

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
                            let mesh = build_chunk_mesh_lod(
                                &chunk,
                                north.as_ref(),
                                south.as_ref(),
                                east.as_ref(),
                                west.as_ref(),
                                true, // max_y_skip
                                chunk_greedy,
                                chunk_lod,
                            );
                            let _ = tx.send((coord, mesh, target_tier));
                        })
                        .detach();
                    meshed += 1;
                }
            }
        }
    }

    // 5. Synchronize modified macro-chunk clusters with Bevy ECS and GPU assets
    crate::world::macro_lod::flush_dirty_macro_chunks(
        &mut commands,
        &mut world,
        &mut assets.meshes,
        &mut assets.materials,
    );
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGeneratorPool>()
            .init_resource::<ChunkMesherPool>()
            .init_resource::<crate::world::occlusion::SectionOcclusionCache>()
            .add_systems(
                Update,
                (
                    world_streaming_system.in_set(crate::stage::VoxelStage::WorldStreaming),
                    crate::world::occlusion::section_occlusion_system
                        .after(crate::stage::VoxelStage::WorldStreaming),
                ),
            );
    }
}
