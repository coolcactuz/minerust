//! Multi-algorithm chunk geometry mesher for MineRust.
//! Supports standard 1x1 voxel quads, greedy coplanar merging (LOD 0),
//! and sloped heightfield continuous meshing with ore simplification (LOD 1).
//! All meshers partition the 16x128x16 chunk column into 8 independent 16x16x16 sub-chunk sections.

pub mod greedy;
pub mod helpers;
pub mod sloped_lod;
pub mod standard;

#[cfg(test)]
mod tests;

use crate::chunk::{CHUNK_HEIGHT, Chunk};
use bevy::prelude::Mesh;

pub use greedy::{build_chunk_mesh_greedy, build_section_mesh_greedy};
pub use helpers::{
    add_quad, add_triangle, can_merge_blocks, should_render_face, simplify_block_for_lod,
    triangle_normal, MeshBuffers,
};
pub use sloped_lod::build_chunk_mesh_sloped_lod;
pub use standard::{build_chunk_mesh_standard, build_section_mesh_standard};

/// Number of vertical sub-chunk sections per chunk column (128 / 16 = 8).
pub const CHUNK_SECTIONS: usize = 8;
/// Height in blocks of each cubic sub-chunk section.
pub const SECTION_HEIGHT: usize = 16;

/// Output geometry meshes for a single 16x16x16 sub-chunk section: separate solid terrain and semi-transparent water.
#[derive(Default, Debug, Clone)]
pub struct SectionMeshes {
    pub solid: Option<Mesh>,
    pub water: Option<Mesh>,
}

impl SectionMeshes {
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

/// Output geometry meshes for an entire 16x128x16 chunk column, divided into 8 cubic sub-chunk sections.
#[derive(Default, Debug)]
pub struct ChunkMeshes {
    pub sections: [SectionMeshes; CHUNK_SECTIONS],
}

impl ChunkMeshes {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(SectionMeshes::is_empty)
    }

    #[must_use]
    pub fn total_vertices(&self) -> usize {
        self.sections.iter().map(SectionMeshes::total_vertices).sum()
    }

    #[must_use]
    pub fn primary_mesh(mut self) -> Option<Mesh> {
        for section in &mut self.sections {
            if let Some(mesh) = section.solid.take().or_else(|| section.water.take()) {
                return Some(mesh);
            }
        }
        None
    }

    #[must_use]
    pub fn first_solid(&self) -> Option<&Mesh> {
        self.sections.iter().find_map(|s| s.solid.as_ref())
    }

    #[must_use]
    pub fn first_solid_mesh(mut self) -> Option<Mesh> {
        for section in &mut self.sections {
            if let Some(m) = section.solid.take() {
                return Some(m);
            }
        }
        None
    }

    #[must_use]
    pub fn first_water(&self) -> Option<&Mesh> {
        self.sections.iter().find_map(|s| s.water.as_ref())
    }

    #[must_use]
    pub fn first_water_mesh(mut self) -> Option<Mesh> {
        for section in &mut self.sections {
            if let Some(m) = section.water.take() {
                return Some(m);
            }
        }
        None
    }

    #[must_use]
    pub fn has_solid(&self) -> bool {
        self.sections.iter().any(|s| s.solid.is_some())
    }

    #[must_use]
    pub fn has_water(&self) -> bool {
        self.sections.iter().any(|s| s.water.is_some())
    }

    #[must_use]
    pub fn from_single_section(section_y: usize, solid: Option<Mesh>, water: Option<Mesh>) -> Self {
        let mut res = Self::default();
        if section_y < CHUNK_SECTIONS {
            res.sections[section_y] = SectionMeshes { solid, water };
        }
        res
    }

    #[must_use]
    pub fn from_solid(mesh: Mesh) -> Self {
        Self::from_single_section(0, Some(mesh), None)
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
