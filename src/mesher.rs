use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::{BlockFace, BlockType};
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::texture::{block_texture, get_tile_uvs};

pub fn build_chunk_mesh(
    chunk: &Chunk,
    chunk_x: i32,
    chunk_z: i32,
    get_block_at: impl Fn(i32, i32, i32) -> BlockType,
) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let world_offset_x = chunk_x * CHUNK_WIDTH as i32;
    let world_offset_z = chunk_z * CHUNK_DEPTH as i32;

    for ly in 0..CHUNK_HEIGHT {
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get(lx as i32, ly as i32, lz as i32);
                if block == BlockType::Air {
                    continue;
                }

                let wx = world_offset_x + lx as i32;
                let wy = ly as i32;
                let wz = world_offset_z + lz as i32;

                let fx = lx as f32;
                let fy = ly as f32;
                let fz = lz as f32;

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
                let top_neighbor = if wy + 1 < CHUNK_HEIGHT as i32 {
                    get_block_at(wx, wy + 1, wz)
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
                            [fx + 1.0, fy + 1.0, fz],
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                        ],
                        [0.0, 1.0, 0.0],
                        tile_uvs,
                        1.0,
                    );
                }

                // Bottom (-Y)
                if wy > 0 {
                    let bottom_neighbor = get_block_at(wx, wy - 1, wz);
                    if should_render_face(bottom_neighbor) {
                        let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::Bottom));
                        add_quad(
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut colors,
                            &mut indices,
                            [
                                [fx + 1.0, fy, fz + 1.0],
                                [fx, fy, fz + 1.0],
                                [fx, fy, fz],
                                [fx + 1.0, fy, fz],
                            ],
                            [0.0, -1.0, 0.0],
                            tile_uvs,
                            0.5,
                        );
                    }
                }

                // North / Front (+Z)
                let north_neighbor = get_block_at(wx, wy, wz + 1);
                if should_render_face(north_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::North));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        tile_uvs,
                        0.85,
                    );
                }

                // South / Back (-Z)
                let south_neighbor = get_block_at(wx, wy, wz - 1);
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
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy, fz],
                            [fx, fy, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        tile_uvs,
                        0.85,
                    );
                }

                // East / Right (+X)
                let east_neighbor = get_block_at(wx + 1, wy, wz);
                if should_render_face(east_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::East));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                        ],
                        [1.0, 0.0, 0.0],
                        tile_uvs,
                        0.7,
                    );
                }

                // West / Left (-X)
                let west_neighbor = get_block_at(wx - 1, wy, wz);
                if should_render_face(west_neighbor) {
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::West));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx, fy + 1.0, fz],
                            [fx, fy, fz],
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

    indices.extend_from_slice(&[
        start_idx,
        start_idx + 1,
        start_idx + 2,
        start_idx + 2,
        start_idx + 3,
        start_idx,
    ]);
}
