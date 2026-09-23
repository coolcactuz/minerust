use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use super::helpers::{add_quad, should_render_face};
use crate::block::{BlockFace, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::texture::{block_texture, quad_uvs};

/// Standard 1x1 voxel face mesher
pub fn build_chunk_mesh_standard(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y: usize,
) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(2048);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(2048);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(2048);
    let mut uvs_1: Vec<[f32; 2]> = Vec::with_capacity(2048);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(2048);
    let mut indices: Vec<u16> = Vec::with_capacity(3072);

    let unit_uvs = quad_uvs(1.0, 1.0);

    for ly in 0..=max_y {
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
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut uvs_1,
                            &mut colors,
                            &mut indices,
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
                let bottom_neighbor = if ly > 0 {
                    chunk.get_fast(lx, ly - 1, lz)
                } else {
                    BlockType::Air
                };
                if should_render_face(block, bottom_neighbor, BlockFace::Bottom) {
                    let layer = block_texture(block, BlockFace::Bottom).layer();
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uvs_1,
                        &mut colors,
                        &mut indices,
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

    if positions.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uvs_1);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U16(indices));

    Some(mesh)
}
