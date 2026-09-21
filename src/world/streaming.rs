use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::camera::FpsCamera;
use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};
use crate::error::WorldError;
use crate::menu::GraphicsSettings;
use crate::mesher::build_chunk_mesh;
use crate::world::grid::WorldGrid;
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

        // Prune stale mesh queue entries that are outside the current visual distance
        world.mesh_queue.retain(|c| {
            let diff = *c - player_chunk;
            diff.x.abs() <= view_dist && diff.y.abs() <= view_dist
        });
        world.queued_for_mesh.retain(|c| {
            let diff = *c - player_chunk;
            diff.x.abs() <= view_dist && diff.y.abs() <= view_dist
        });

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
    let mut dispatched = 0;
    while dispatched < MAX_CHUNK_DISPATCH_PER_FRAME && !world.generation_queue.is_empty() {
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

            // Guard against race conditions / zombie entities:
            // If the player moved away and this chunk is now outside visual view distance,
            // discard the completed mesh immediately rather than spawning an off-screen entity.
            let diff = coord - player_chunk;
            if diff.x.abs() > view_dist || diff.y.abs() > view_dist {
                if let Some(entity) = world.chunk_entities.remove(&coord) {
                    commands.entity(entity).despawn();
                }
                if let Some(old_v) = world.chunk_vertices.remove(&coord) {
                    world.total_vertices = world.total_vertices.saturating_sub(old_v);
                }
                world.chunk_lod.remove(&coord);
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
