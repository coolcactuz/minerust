//! Multi-algorithm chunk geometry mesher for MineRust.
//! Supports standard 1x1 voxel quads, greedy coplanar merging (LOD 0),
//! and sloped heightfield continuous meshing with ore simplification (LOD 1).

pub mod greedy;
pub mod helpers;
pub mod sloped_lod;
pub mod standard;

#[cfg(test)]
mod tests;

use crate::chunk::{CHUNK_HEIGHT, Chunk};
use bevy::prelude::Mesh;

pub use greedy::build_chunk_mesh_greedy;
pub use helpers::{
    add_quad, add_triangle, can_merge_blocks, should_render_face, simplify_block_for_lod,
    triangle_normal, MeshBuffers,
};
pub use sloped_lod::build_chunk_mesh_sloped_lod;
pub use standard::build_chunk_mesh_standard;

/// Output geometry meshes for a single chunk: separate solid terrain and semi-transparent water.
#[derive(Default, Debug)]
pub struct ChunkMeshes {
    pub solid: Option<Mesh>,
    pub water: Option<Mesh>,
}

impl ChunkMeshes {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.solid.is_none() && self.water.is_none()
    }

    #[must_use]
    pub fn total_vertices(&self) -> usize {
        self.solid.as_ref().map_or(0, Mesh::count_vertices)
            + self.water.as_ref().map_or(0, Mesh::count_vertices)
    }

    #[must_use]
    pub fn primary_mesh(self) -> Option<Mesh> {
        self.solid.or(self.water)
    }
}

/// Builds the geometry mesh for a chunk.
/// Defaults to LOD 0 (detailed greedy/standard voxel meshing).
#[must_use]
pub fn build_chunk_mesh(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y_skip: bool,
    greedy: bool,
) -> ChunkMeshes {
    build_chunk_mesh_lod(chunk, north, south, east, west, max_y_skip, greedy, 0)
}

/// Builds the geometry mesh for a chunk with level of detail awareness.
/// - `lod == 0`: Full detail 3D voxel mesh (greedy or standard).
/// - `lod >= 1`: Continuous sloped heightfield mesh with ore grouping.
#[must_use]
pub fn build_chunk_mesh_lod(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y_skip: bool,
    greedy: bool,
    lod: u8,
) -> ChunkMeshes {
    let max_y = if max_y_skip {
        chunk.max_y.min(CHUNK_HEIGHT - 1)
    } else {
        CHUNK_HEIGHT - 1
    };

    if lod >= 1 {
        build_chunk_mesh_sloped_lod(chunk, north, south, east, west, max_y)
    } else if greedy {
        build_chunk_mesh_greedy(chunk, north, south, east, west, max_y)
    } else {
        build_chunk_mesh_standard(chunk, north, south, east, west, max_y)
    }
}
