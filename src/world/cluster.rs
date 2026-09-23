//! Regional Chunk Clustering (Chunk Merging) for Distant Level of Detail (LOD).
//!
//! Merges multiple distant 16x16 chunk meshes into composite regional meshes (e.g. 2x2 or 4x4 chunks)
//! to dramatically reduce the active Entity count in Bevy's ECS and eliminate CPU frustum culling overhead.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

/// Converts a 16x16 chunk coordinate into a regional cluster coordinate.
#[inline]
pub const fn chunk_to_region_coord(chunk_coord: IVec2, cluster_size: i32) -> IVec2 {
    IVec2::new(
        chunk_coord.x.div_euclid(cluster_size),
        chunk_coord.y.div_euclid(cluster_size),
    )
}

/// Calculates the 3D world origin position for a regional cluster entity.
#[inline]
pub const fn region_to_world_pos(region_coord: IVec2, cluster_size: i32) -> Vec3 {
    Vec3::new(
        (region_coord.x * cluster_size * 16) as f32,
        0.0,
        (region_coord.y * cluster_size * 16) as f32,
    )
}

/// Calculates the local relative offset of a chunk within its regional cluster.
#[inline]
pub fn chunk_offset_in_region(chunk_coord: IVec2, cluster_size: i32) -> Vec3 {
    let ox = chunk_coord.x.rem_euclid(cluster_size);
    let oz = chunk_coord.y.rem_euclid(cluster_size);
    Vec3::new((ox * 16) as f32, 0.0, (oz * 16) as f32)
}

/// Merges multiple chunk meshes with their relative translation offsets into a single composite Mesh.
pub fn merge_chunk_meshes<'a>(
    meshes_with_offsets: impl IntoIterator<Item = (&'a Mesh, Vec3)>,
) -> Option<Mesh> {
    let mut total_positions = Vec::new();
    let mut total_normals = Vec::new();
    let mut total_uvs = Vec::new();
    let mut total_uvs_1 = Vec::new();
    let mut total_colors = Vec::new();
    let mut total_indices = Vec::new();

    for (mesh, offset) in meshes_with_offsets {
        let base_vertex = total_positions.len() as u32;

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };

        total_positions.reserve(positions.len());
        for &p in positions {
            total_positions.push([p[0] + offset.x, p[1] + offset.y, p[2] + offset.z]);
        }

        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            total_normals.extend_from_slice(normals);
        }

        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        {
            total_uvs.extend_from_slice(uvs);
        }

        if let Some(VertexAttributeValues::Float32x2(uvs_1)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_1)
        {
            total_uvs_1.extend_from_slice(uvs_1);
        }

        if let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        {
            total_colors.extend_from_slice(colors);
        }

        if let Some(indices) = mesh.indices() {
            match indices {
                Indices::U32(inds) => {
                    total_indices.reserve(inds.len());
                    for &idx in inds {
                        total_indices.push(idx + base_vertex);
                    }
                }
                Indices::U16(inds) => {
                    total_indices.reserve(inds.len());
                    for &idx in inds {
                        total_indices.push(u32::from(idx) + base_vertex);
                    }
                }
            }
        }
    }

    if total_positions.is_empty() {
        return None;
    }

    let mut merged = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    merged.insert_attribute(Mesh::ATTRIBUTE_POSITION, total_positions);
    merged.insert_attribute(Mesh::ATTRIBUTE_NORMAL, total_normals);
    merged.insert_attribute(Mesh::ATTRIBUTE_UV_0, total_uvs);
    merged.insert_attribute(Mesh::ATTRIBUTE_UV_1, total_uvs_1);
    merged.insert_attribute(Mesh::ATTRIBUTE_COLOR, total_colors);
    merged.insert_indices(Indices::U32(total_indices));

    Some(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_to_region_coord_positive_and_negative() {
        // With cluster_size = 2
        assert_eq!(chunk_to_region_coord(IVec2::new(0, 0), 2), IVec2::new(0, 0));
        assert_eq!(chunk_to_region_coord(IVec2::new(1, 1), 2), IVec2::new(0, 0));
        assert_eq!(chunk_to_region_coord(IVec2::new(2, 3), 2), IVec2::new(1, 1));
        assert_eq!(chunk_to_region_coord(IVec2::new(-1, -1), 2), IVec2::new(-1, -1));
        assert_eq!(chunk_to_region_coord(IVec2::new(-2, -2), 2), IVec2::new(-1, -1));
        assert_eq!(chunk_to_region_coord(IVec2::new(-3, -4), 2), IVec2::new(-2, -2));

        // With cluster_size = 4
        assert_eq!(chunk_to_region_coord(IVec2::new(3, 3), 4), IVec2::new(0, 0));
        assert_eq!(chunk_to_region_coord(IVec2::new(4, 7), 4), IVec2::new(1, 1));
        assert_eq!(chunk_to_region_coord(IVec2::new(-1, -4), 4), IVec2::new(-1, -1));
    }

    #[test]
    fn test_chunk_offset_in_region() {
        assert_eq!(chunk_offset_in_region(IVec2::new(0, 0), 2), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(chunk_offset_in_region(IVec2::new(1, 0), 2), Vec3::new(16.0, 0.0, 0.0));
        assert_eq!(chunk_offset_in_region(IVec2::new(0, 1), 2), Vec3::new(0.0, 0.0, 16.0));
        assert_eq!(chunk_offset_in_region(IVec2::new(1, 1), 2), Vec3::new(16.0, 0.0, 16.0));

        // Negative coords with rem_euclid
        assert_eq!(chunk_offset_in_region(IVec2::new(-1, -1), 2), Vec3::new(16.0, 0.0, 16.0));
        assert_eq!(chunk_offset_in_region(IVec2::new(-2, -2), 2), Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_region_to_world_pos() {
        assert_eq!(region_to_world_pos(IVec2::new(0, 0), 2), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(region_to_world_pos(IVec2::new(1, 2), 2), Vec3::new(32.0, 0.0, 64.0));
        assert_eq!(region_to_world_pos(IVec2::new(-1, -2), 2), Vec3::new(-32.0, 0.0, -64.0));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_merge_chunk_meshes() {
        // Create 2 mock meshes
        let mut m1 = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        m1.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 0.0]]);
        m1.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
        m1.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
        m1.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[1.0, 0.0]; 3]);
        m1.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; 3]);
        m1.insert_indices(Indices::U32(vec![0, 1, 2]));

        let mut m2 = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        m2.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 2.0, 0.0], [1.0, 2.0, 0.0], [0.0, 0.0, 0.0]]);
        m2.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 3]);
        m2.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
        m2.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[2.0, 0.0]; 3]);
        m2.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.8, 0.8, 0.8, 1.0]; 3]);
        m2.insert_indices(Indices::U32(vec![0, 1, 2]));

        let offset2 = Vec3::new(16.0, 0.0, 0.0);
        let merged = merge_chunk_meshes([(&m1, Vec3::ZERO), (&m2, offset2)]).expect("merge should succeed");

        assert_eq!(merged.count_vertices(), 6);

        if let Some(VertexAttributeValues::Float32x3(pos)) = merged.attribute(Mesh::ATTRIBUTE_POSITION) {
            assert_eq!(pos[0], [0.0, 1.0, 0.0]);
            assert_eq!(pos[3], [16.0, 2.0, 0.0]); // Offset applied!
        } else {
            panic!("Expected Float32x3 positions");
        }

        if let Some(Indices::U32(inds)) = merged.indices() {
            assert_eq!(inds, &[0, 1, 2, 3, 4, 5]); // Base vertex index offset applied!
        } else {
            panic!("Expected U32 indices");
        }
    }
}
