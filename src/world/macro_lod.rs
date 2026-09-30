use bevy::asset::RenderAssetUsages;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

use crate::mesher::SectionMeshes;
use crate::voxel_material::VoxelBlockMaterial;
use crate::world::grid::WorldGrid;

/// Dimension in chunks along each horizontal axis of a macro-chunk cluster (4x4 chunks = 64m x 64m).
pub const MACRO_CHUNK_SIZE: i32 = 4;

/// Total chunk capacity of an individual macro-chunk cluster (4 * 4 = 16 chunks).
pub const MACRO_CHUNK_AREA: usize = (MACRO_CHUNK_SIZE * MACRO_CHUNK_SIZE) as usize;

/// Maps a chunk coordinate `(cx, cz)` to its enclosing macro-chunk coordinate `(mx, mz)`
/// and its zero-indexed local slot within the macro-chunk cluster `(0..16)`.
#[inline]
#[must_use]
pub fn chunk_to_macro_coord(coord: IVec2) -> (IVec2, usize) {
    let mx = coord.x.div_euclid(MACRO_CHUNK_SIZE);
    let mz = coord.y.div_euclid(MACRO_CHUNK_SIZE);
    let lx = coord.x.rem_euclid(MACRO_CHUNK_SIZE) as usize;
    let lz = coord.y.rem_euclid(MACRO_CHUNK_SIZE) as usize;
    (IVec2::new(mx, mz), lx + lz * (MACRO_CHUNK_SIZE as usize))
}

/// Component attached to consolidated macro-chunk entities for identification and lifecycle tracking.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, Reflect)]
pub struct MacroChunkSection {
    pub macro_coord: IVec2,
}

/// In-memory state and GPU entity handles for an individual 4x4 macro-chunk cluster.
#[derive(Default, Debug)]
pub struct MacroChunk {
    pub solid_entity: Option<Entity>,
    pub water_entity: Option<Entity>,
    pub chunks: [Option<SectionMeshes>; MACRO_CHUNK_AREA],
    pub dirty: bool,
}

impl MacroChunk {
    #[inline]
    #[must_use]
    pub fn has_any_chunk(&self) -> bool {
        self.chunks.iter().any(Option::is_some)
    }

    #[inline]
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.iter().filter(|c| c.is_some()).count()
    }
}

/// Merges up to 16 chunk meshes into a single consolidated `Mesh` in macro-chunk local space `[0..64, 0..128, 0..64]`.
///
/// Automatically offsets vertex positions according to each chunk's local slot `(lx, lz)`
/// and uses compact `u16` indices if vertex count <= 65,535, gracefully falling back to `u32` if exceeded.
pub fn merge_chunk_meshes<'a>(
    chunk_meshes: impl IntoIterator<Item = (usize, &'a Mesh)>,
) -> Option<Mesh> {
    let mut total_verts = 0;
    let mut total_indices = 0;

    let items: Vec<(usize, &'a Mesh)> = chunk_meshes.into_iter().collect();
    for (_, mesh) in &items {
        total_verts += mesh.count_vertices();
        total_indices += mesh.indices().map_or(0, |idx| idx.len());
    }

    if total_verts == 0 {
        return None;
    }

    let mut merged_positions = Vec::with_capacity(total_verts);
    let mut merged_normals = Vec::with_capacity(total_verts);
    let mut merged_uvs = Vec::with_capacity(total_verts);
    let mut merged_uvs_1 = Vec::with_capacity(total_verts);
    let mut merged_indices = Vec::with_capacity(total_indices);

    for (slot, mesh) in items {
        let lx = (slot % (MACRO_CHUNK_SIZE as usize)) as f32;
        let lz = (slot / (MACRO_CHUNK_SIZE as usize)) as f32;
        let offset_x = lx * 16.0;
        let offset_z = lz * 16.0;

        let base_vertex = merged_positions.len() as u32;

        if let Some(VertexAttributeValues::Float32x3(pos)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            for p in pos {
                merged_positions.push([p[0] + offset_x, p[1], p[2] + offset_z]);
            }
        }

        if let Some(VertexAttributeValues::Float32x3(norm)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            merged_normals.extend_from_slice(norm);
        }

        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            merged_uvs.extend_from_slice(uvs);
        }

        if let Some(VertexAttributeValues::Float32x2(uvs_1)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
        {
            merged_uvs_1.extend_from_slice(uvs_1);
        }

        if let Some(indices) = mesh.indices() {
            match indices {
                Indices::U16(idx) => {
                    for i in idx {
                        merged_indices.push(base_vertex + (*i as u32));
                    }
                }
                Indices::U32(idx) => {
                    for i in idx {
                        merged_indices.push(base_vertex + *i);
                    }
                }
            }
        }
    }

    let mut merged_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    merged_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, merged_positions);
    merged_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, merged_normals);
    merged_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, merged_uvs);
    merged_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, merged_uvs_1);

    if u16::try_from(total_verts).is_ok() {
        let u16_indices: Vec<u16> = merged_indices.into_iter().map(|i| i as u16).collect();
        merged_mesh.insert_indices(Indices::U16(u16_indices));
    } else {
        merged_mesh.insert_indices(Indices::U32(merged_indices));
    }

    Some(merged_mesh)
}

/// Rebuilds and synchronizes all dirty macro-chunk clusters with the Bevy ECS and GPU assets.
pub fn flush_dirty_macro_chunks(
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelBlockMaterial>,
) {
    let solid_material = world.block_material.clone().unwrap_or_else(|| {
        materials.add(ExtendedMaterial {
            base: StandardMaterial {
                cull_mode: Some(bevy::render::render_resource::Face::Back),
                perceptual_roughness: 0.85,
                reflectance: 0.15,
                ..default()
            },
            extension: crate::voxel_material::VoxelExtension {
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
            extension: crate::voxel_material::VoxelExtension {
                array_texture: Handle::default(),
            },
        })
    });

    // Fast path: if no macro-chunks were marked dirty this frame, skip completely
    if world.dirty_macro_chunks.is_empty() {
        return;
    }

    let mut empty_macros = Vec::new();

    // Drain keys of dirty macro-chunks to avoid iterating all macro-chunks every frame
    let dirty_keys: Vec<IVec2> = world.dirty_macro_chunks.drain().collect();

    for macro_coord in dirty_keys {
        let Some(macro_chunk) = world.macro_chunks.get_mut(&macro_coord) else {
            continue;
        };
        macro_chunk.dirty = false;

        if !macro_chunk.has_any_chunk() {
            if let Some(e) = macro_chunk.solid_entity.take() {
                commands.entity(e).despawn();
            }
            if let Some(e) = macro_chunk.water_entity.take() {
                commands.entity(e).despawn();
            }
            empty_macros.push(macro_coord);
            continue;
        }

        let world_pos = Vec3::new(
            (macro_coord.x * MACRO_CHUNK_SIZE * 16) as f32,
            0.0,
            (macro_coord.y * MACRO_CHUNK_SIZE * 16) as f32,
        );

        // 1. Build and synchronize solid terrain mesh
        let solid_mesh_opt = merge_chunk_meshes(macro_chunk.chunks.iter().enumerate().filter_map(
            |(slot, s)| {
                s.as_ref()
                    .and_then(|sec| sec.solid.as_ref())
                    .map(|m| (slot, m))
            },
        ));

        if let Some(mesh) = solid_mesh_opt {
            if let Some(entity) = macro_chunk.solid_entity {
                commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
            } else {
                let entity = commands
                    .spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(solid_material.clone()),
                        Transform::from_translation(world_pos),
                        bevy::light::NotShadowCaster,
                        MacroChunkSection { macro_coord },
                    ))
                    .id();
                macro_chunk.solid_entity = Some(entity);
            }
        } else if let Some(entity) = macro_chunk.solid_entity.take() {
            commands.entity(entity).despawn();
        }

        // 2. Build and synchronize water surface mesh
        let water_mesh_opt = merge_chunk_meshes(macro_chunk.chunks.iter().enumerate().filter_map(
            |(slot, s)| {
                s.as_ref()
                    .and_then(|sec| sec.water.as_ref())
                    .map(|m| (slot, m))
            },
        ));

        if let Some(mesh) = water_mesh_opt {
            if let Some(entity) = macro_chunk.water_entity {
                commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
            } else {
                let entity = commands
                    .spawn((
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(water_material.clone()),
                        Transform::from_translation(world_pos),
                        bevy::light::NotShadowCaster,
                        MacroChunkSection { macro_coord },
                    ))
                    .id();
                macro_chunk.water_entity = Some(entity);
            }
        } else if let Some(entity) = macro_chunk.water_entity.take() {
            commands.entity(entity).despawn();
        }
    }

    for empty_coord in empty_macros {
        world.macro_chunks.remove(&empty_coord);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::Indices;

    fn create_test_mesh(pos_offset: f32) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [pos_offset, 0.0, 0.0],
                [pos_offset + 1.0, 0.0, 0.0],
                [pos_offset, 1.0, 0.0],
            ],
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_1,
            vec![[1.0, 1.0], [1.0, 1.0], [1.0, 1.0]],
        );
        mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
        mesh
    }

    #[test]
    fn test_chunk_to_macro_coord_mapping() {
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(0, 0)),
            (IVec2::new(0, 0), 0)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(3, 0)),
            (IVec2::new(0, 0), 3)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(0, 3)),
            (IVec2::new(0, 0), 12)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(3, 3)),
            (IVec2::new(0, 0), 15)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(4, 0)),
            (IVec2::new(1, 0), 0)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(-1, -1)),
            (IVec2::new(-1, -1), 15)
        );
        assert_eq!(
            chunk_to_macro_coord(IVec2::new(-4, -4)),
            (IVec2::new(-1, -1), 0)
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_merge_chunk_meshes_offsets_and_indices() {
        let m0 = create_test_mesh(0.0);
        let m1 = create_test_mesh(2.0);

        // slot 0: lx=0, lz=0 -> offset (0, 0)
        // slot 5: lx=1, lz=1 -> offset (16, 16)
        let merged = merge_chunk_meshes(vec![(0, &m0), (5, &m1)]).expect("Must return merged mesh");

        assert_eq!(merged.count_vertices(), 6);
        let pos = match merged.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(p) => p.clone(),
            _ => panic!("Expected Float32x3"),
        };

        // First mesh positions (no offset)
        assert_eq!(pos[0], [0.0, 0.0, 0.0]);
        assert_eq!(pos[1], [1.0, 0.0, 0.0]);
        assert_eq!(pos[2], [0.0, 1.0, 0.0]);

        // Second mesh positions (offset by (16, 16))
        assert_eq!(pos[3], [18.0, 0.0, 16.0]);
        assert_eq!(pos[4], [19.0, 0.0, 16.0]);
        assert_eq!(pos[5], [18.0, 1.0, 16.0]);

        // Indices: second triangle must be offset by 3
        match merged.indices().unwrap() {
            Indices::U16(idx) => {
                assert_eq!(idx, &[0, 1, 2, 3, 4, 5]);
            }
            Indices::U32(_) => panic!("Expected U16 indices"),
        }
    }

    #[test]
    fn test_merge_empty_chunks_returns_none() {
        let merged = merge_chunk_meshes(Vec::<(usize, &Mesh)>::new());
        assert!(merged.is_none());
    }

    #[test]
    fn test_macro_chunk_helpers() {
        let mut mc = MacroChunk::default();
        assert!(!mc.has_any_chunk());
        assert_eq!(mc.chunk_count(), 0);

        mc.chunks[0] = Some(SectionMeshes {
            solid: Some(create_test_mesh(0.0)),
            water: None,
        });

        assert!(mc.has_any_chunk());
        assert_eq!(mc.chunk_count(), 1);
    }
}
