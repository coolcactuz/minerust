use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::{BlockFace, BlockType};
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::texture::{block_texture, get_tile_uvs};

pub fn build_chunk_mesh(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(2048);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(2048);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(2048);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(2048);
    let mut indices: Vec<u32> = Vec::with_capacity(3072);

    let max_y = chunk.max_y.min(CHUNK_HEIGHT - 1);
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

                let should_render_face = |neighbor: BlockType| -> bool {
                    if block.is_solid() {
                        !neighbor.is_solid()
                    } else if block.is_water() {
                        neighbor == BlockType::Air
                    } else {
                        false
                    }
                };

                // Top (+Y)
                let top_neighbor = if ly + 1 < CHUNK_HEIGHT {
                    chunk.get_fast(lx, ly + 1, lz)
                } else {
                    BlockType::Air
                };
                if should_render_face(top_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::Top));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz],
                        ],
                        [0.0, 1.0, 0.0],
                        tile_uvs,
                        1.0,
                    );
                }

                // Bottom (-Y)
                if ly > 0 {
                    let bottom_neighbor = chunk.get_fast(lx, ly - 1, lz);
                    if should_render_face(bottom_neighbor) {
                        let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::Bottom));
                        add_quad(
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut colors,
                            &mut indices,
                            [
                                [fx, fy, fz + 1.0],
                                [fx, fy, fz],
                                [fx + 1.0, fy, fz],
                                [fx + 1.0, fy, fz + 1.0],
                            ],
                            [0.0, -1.0, 0.0],
                            tile_uvs,
                            0.5,
                        );
                    }
                }

                // North / Front (+Z)
                let north_neighbor = if lz + 1 < CHUNK_DEPTH {
                    chunk.get_fast(lx, ly, lz + 1)
                } else if let Some(n) = north {
                    n.get_fast(lx, ly, 0)
                } else {
                    BlockType::Air
                };
                if should_render_face(north_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::North));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + 1.0, fz + 1.0],
                            [fx, fy, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        tile_uvs,
                        0.85,
                    );
                }

                // South / Back (-Z)
                let south_neighbor = if lz > 0 {
                    chunk.get_fast(lx, ly, lz - 1)
                } else if let Some(s) = south {
                    s.get_fast(lx, ly, CHUNK_DEPTH - 1)
                } else {
                    BlockType::Air
                };
                if should_render_face(south_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::South));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy, fz],
                            [fx, fy, fz],
                            [fx, fy + 1.0, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        tile_uvs,
                        0.85,
                    );
                }

                // East / Right (+X)
                let east_neighbor = if lx + 1 < CHUNK_WIDTH {
                    chunk.get_fast(lx + 1, ly, lz)
                } else if let Some(e) = east {
                    e.get_fast(0, ly, lz)
                } else {
                    BlockType::Air
                };
                if should_render_face(east_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::East));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + 1.0, fz],
                        ],
                        [1.0, 0.0, 0.0],
                        tile_uvs,
                        0.7,
                    );
                }

                // West / Left (-X)
                let west_neighbor = if lx > 0 {
                    chunk.get_fast(lx - 1, ly, lz)
                } else if let Some(w) = west {
                    w.get_fast(CHUNK_WIDTH - 1, ly, lz)
                } else {
                    BlockType::Air
                };
                if should_render_face(west_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::West));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy, fz],
                            [fx, fy, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                        ],
                        [-1.0, 0.0, 0.0],
                        tile_uvs,
                        0.7,
                    );
                }
            }
        }
    }

    if positions.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    Some(mesh)
}

#[inline(always)]
fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    verts: [[f32; 3]; 4],
    norm: [f32; 3],
    quad_uvs: [[f32; 2]; 4],
    shade: f32,
) {
    let start_idx = positions.len() as u32;

    for v in &verts {
        positions.push(*v);
        normals.push(norm);
        colors.push([shade, shade, shade, 1.0]);
    }

    uvs.extend_from_slice(&quad_uvs);

    // Standard Bevy Cuboid CCW winding: 0, 1, 2, 2, 3, 0
    indices.extend_from_slice(&[
        start_idx,
        start_idx + 1,
        start_idx + 2,
        start_idx + 2,
        start_idx + 3,
        start_idx,
    ]);
}
