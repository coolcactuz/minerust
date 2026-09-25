use super::helpers::{add_quad, should_render_face, MeshBuffers};
use super::{
    compute_section_connectivity, ChunkMeshes, SectionConnectivity, SectionMeshes,
    CHUNK_SECTIONS, SECTION_HEIGHT,
};
use crate::block::{BlockFace, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::texture::{block_texture, quad_uvs};

/// Builds the geometry mesh for a single 16x16x16 section using standard 1x1 voxel quads.
pub fn build_section_mesh_standard(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    sy: usize,
    max_y: usize,
) -> SectionMeshes {
    let y_start = sy * SECTION_HEIGHT;
    if y_start > max_y {
        return SectionMeshes::default();
    }
    let y_max = (y_start + SECTION_HEIGHT - 1).min(max_y);
    if y_max < y_start {
        return SectionMeshes::default();
    }

    let mut solid = MeshBuffers::with_capacity(512, 768);
    let mut water = MeshBuffers::default();

    let unit_uvs = quad_uvs(1.0, 1.0);

    for ly in y_start..=y_max {
        let fy = ly as f32;
        for lz in 0..CHUNK_DEPTH {
            let fz = lz as f32;
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block == BlockType::Air {
                    continue;
                }

                let fx = lx as f32;

                // Top (+Y)
                let top_neighbor = if ly + 1 < CHUNK_HEIGHT {
                    chunk.get_fast(lx, ly + 1, lz)
                } else {
                    BlockType::Air
                };
                if should_render_face(block, top_neighbor, BlockFace::Top) {
                    let layer = block_texture(block, BlockFace::Top).layer();
                    let target = if block.is_water() { &mut water } else { &mut solid };
                    add_quad(
                        target,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz],
                        ],
                        [0.0, 1.0, 0.0],
                        unit_uvs,
                        layer,
                        1.0,
                    );

                    // For surface water, also render downward-facing ceiling quad for underwater viewing
                    if block.is_water() {
                        add_quad(
                            &mut water,
                            [
                                [fx, fy + 1.0, fz],
                                [fx + 1.0, fy + 1.0, fz],
                                [fx + 1.0, fy + 1.0, fz + 1.0],
                                [fx, fy + 1.0, fz + 1.0],
                            ],
                            [0.0, -1.0, 0.0],
                            unit_uvs,
                            layer,
                            0.7,
                        );
                    }
                }

                // Bottom (-Y)
                if ly > 0 {
                    let bottom_neighbor = chunk.get_fast(lx, ly - 1, lz);
                    if should_render_face(block, bottom_neighbor, BlockFace::Bottom) {
                        let layer = block_texture(block, BlockFace::Bottom).layer();
                        let target = if block.is_water() { &mut water } else { &mut solid };
                        add_quad(
                            target,
                            [
                                [fx, fy, fz + 1.0],
                                [fx, fy, fz],
                                [fx + 1.0, fy, fz],
                                [fx + 1.0, fy, fz + 1.0],
                            ],
                            [0.0, -1.0, 0.0],
                            unit_uvs,
                            layer,
                            0.5,
                        );
                    }
                }

                // North / Front (+Z)
                let north_neighbor = if lz + 1 < CHUNK_DEPTH {
                    chunk.get_fast(lx, ly, lz + 1)
                } else if let Some(n) = north {
                    n.get_fast(lx, ly, 0)
                } else if block.is_water() {
                    BlockType::Water
                } else {
                    BlockType::Air
                };
                if should_render_face(block, north_neighbor, BlockFace::North) {
                    let layer = block_texture(block, BlockFace::North).layer();
                    let target = if block.is_water() { &mut water } else { &mut solid };
                    add_quad(
                        target,
                        [
                            [fx, fy + 1.0, fz + 1.0],
                            [fx, fy, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        unit_uvs,
                        layer,
                        0.85,
                    );
                }

                // South / Back (-Z)
                let south_neighbor = if lz > 0 {
                    chunk.get_fast(lx, ly, lz - 1)
                } else if let Some(s) = south {
                    s.get_fast(lx, ly, CHUNK_DEPTH - 1)
                } else if block.is_water() {
                    BlockType::Water
                } else {
                    BlockType::Air
                };
                if should_render_face(block, south_neighbor, BlockFace::South) {
                    let layer = block_texture(block, BlockFace::South).layer();
                    let target = if block.is_water() { &mut water } else { &mut solid };
                    add_quad(
                        target,
                        [
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy, fz],
                            [fx, fy, fz],
                            [fx, fy + 1.0, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        unit_uvs,
                        layer,
                        0.85,
                    );
                }

                // East / Right (+X)
                let east_neighbor = if lx + 1 < CHUNK_WIDTH {
                    chunk.get_fast(lx + 1, ly, lz)
                } else if let Some(e) = east {
                    e.get_fast(0, ly, lz)
                } else if block.is_water() {
                    BlockType::Water
                } else {
                    BlockType::Air
                };
                if should_render_face(block, east_neighbor, BlockFace::East) {
                    let layer = block_texture(block, BlockFace::East).layer();
                    let target = if block.is_water() { &mut water } else { &mut solid };
                    add_quad(
                        target,
                        [
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + 1.0, fz],
                        ],
                        [1.0, 0.0, 0.0],
                        unit_uvs,
                        layer,
                        0.7,
                    );
                }

                // West / Left (-X)
                let west_neighbor = if lx > 0 {
                    chunk.get_fast(lx - 1, ly, lz)
                } else if let Some(w) = west {
                    w.get_fast(CHUNK_WIDTH - 1, ly, lz)
                } else if block.is_water() {
                    BlockType::Water
                } else {
                    BlockType::Air
                };
                if should_render_face(block, west_neighbor, BlockFace::West) {
                    let layer = block_texture(block, BlockFace::West).layer();
                    let target = if block.is_water() { &mut water } else { &mut solid };
                    add_quad(
                        target,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy, fz],
                            [fx, fy, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                        ],
                        [-1.0, 0.0, 0.0],
                        unit_uvs,
                        layer,
                        0.7,
                    );
                }
            }
        }
    }

    SectionMeshes {
        solid: solid.to_mesh(),
        water: water.to_mesh(),
    }
}

/// Builds the geometry mesh for an entire 16x128x16 chunk column using standard 1x1 voxel quads.
pub fn build_chunk_mesh_standard(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y: usize,
) -> ChunkMeshes {
    let mut sections: [SectionMeshes; CHUNK_SECTIONS] = Default::default();
    let mut connectivity: [SectionConnectivity; CHUNK_SECTIONS] = Default::default();
    for (sy, section) in sections.iter_mut().enumerate() {
        *section = build_section_mesh_standard(chunk, north, south, east, west, sy, max_y);
        connectivity[sy] = compute_section_connectivity(chunk, sy);
    }
    ChunkMeshes { sections, connectivity }
}
