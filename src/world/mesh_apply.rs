//! GPU mesh entity allocation, material assignment, and shadow component initialization for chunks.

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoAutoAabb;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;

use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH};
use crate::mesher::{CHUNK_SECTIONS, ChunkMeshes};
use crate::voxel_material::{VoxelBlockMaterial, VoxelExtension};
use crate::world::grid::{ChunkSection, WaterSection, WorldGrid};
use crate::world::occlusion::SHADOW_MAX_CHUNK_RADIUS;
use crate::world::streaming::TIER_SLOPED_LOD;

/// Applies or despawns chunk meshes (solid terrain and water) on the GPU.
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
        if !macro_chunk.dirty {
            macro_chunk.dirty_age_frames = 0;
            macro_chunk.dirty = true;
        }
        world.dirty_macro_chunks.insert(macro_coord);
        return;
    }

    // LOD 0 (tier 0 or 1): remove from macro-chunk if transitioning from distant LOD 2
    let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
    if let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) {
        if macro_chunk.chunks[slot].take().is_some() {
            if !macro_chunk.dirty {
                macro_chunk.dirty_age_frames = 0;
                macro_chunk.dirty = true;
            }
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

    // Check if chunk is outside directional shadow cascade range relative to player camera
    let is_outside_shadows = if let Some(player_chunk) = world.last_player_chunk {
        let diff = coord - player_chunk;
        diff.x.abs() > SHADOW_MAX_CHUNK_RADIUS || diff.y.abs() > SHADOW_MAX_CHUNK_RADIUS
    } else {
        false
    };

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
        let should_not_cast = is_outside_shadows || section_y <= 1;
        let should_not_receive = is_outside_shadows || section_y <= 1;

        if let Some(entity) = solid_entities[sy] {
            if let Some(mesh) = section_mesh.solid {
                let mut entity_cmd = commands.entity(entity);
                entity_cmd.insert((
                    Mesh3d(meshes.add(mesh)),
                    aabb,
                    NoAutoAabb,
                    Visibility::Inherited,
                ));
                if should_not_cast {
                    entity_cmd.insert(NotShadowCaster);
                } else {
                    entity_cmd.remove::<NotShadowCaster>();
                }
                if should_not_receive {
                    entity_cmd.insert(NotShadowReceiver);
                } else {
                    entity_cmd.remove::<NotShadowReceiver>();
                }
            } else {
                commands.entity(entity).despawn();
                solid_entities[sy] = None;
            }
        } else if let Some(mesh) = section_mesh.solid {
            let mut entity_cmd = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(solid_material.clone()),
                Transform::from_translation(world_pos),
                aabb,
                NoAutoAabb,
                ChunkSection {
                    chunk: coord,
                    section_y,
                },
            ));
            if should_not_cast {
                entity_cmd.insert(NotShadowCaster);
            }
            if should_not_receive {
                entity_cmd.insert(NotShadowReceiver);
            }
            solid_entities[sy] = Some(entity_cmd.id());
        }

        // Water surface mesh entity (transparent water never casts or receives shadows)
        if let Some(entity) = water_entities[sy] {
            if let Some(mesh) = section_mesh.water {
                commands.entity(entity).insert((
                    Mesh3d(meshes.add(mesh)),
                    aabb,
                    NoAutoAabb,
                    Visibility::Inherited,
                    WaterSection,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
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
                    WaterSection,
                    NotShadowCaster,
                    NotShadowReceiver,
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

/// Cleans up GPU entities, macro-chunk slot, and cached metadata for a chunk that is de-meshed or unloaded.
pub fn cleanup_chunk_mesh_state(coord: IVec2, commands: &mut Commands, world: &mut WorldGrid) {
    let (macro_coord, slot) = crate::world::macro_lod::chunk_to_macro_coord(coord);
    let mut remove_macro = false;
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
            remove_macro = true;
        }
    }
    if remove_macro {
        world.macro_chunks.remove(&macro_coord);
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
