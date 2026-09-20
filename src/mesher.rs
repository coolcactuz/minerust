use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::BlockFace;
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};

pub fn build_chunk_mesh(
    chunk: &Chunk,
    chunk_x: i32,
    chunk_z: i32,
    is_solid_at: impl Fn(i32, i32, i32) -> bool,
) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let world_offset_x = chunk_x * CHUNK_WIDTH as i32;
    let world_offset_z = chunk_z * CHUNK_DEPTH as i32;

    for ly in 0..CHUNK_HEIGHT {
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get(lx as i32, ly as i32, lz as i32);
                if !block.is_solid() {
                    continue;
                }

                let wx = world_offset_x + lx as i32;
                let wy = ly as i32;
                let wz = world_offset_z + lz as i32;

                let fx = lx as f32;
                let fy = ly as f32;
                let fz = lz as f32;

                // Top (+Y)
                if !is_solid_at(wx, wy + 1, wz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx, fy + 1.0, fz],
                        ],
                        [0.0, 1.0, 0.0],
                        block.color(BlockFace::Top),
                    );
                }

                // Bottom (-Y)
                if wy > 0 && !is_solid_at(wx, wy - 1, wz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx, fy, fz + 1.0],
                        ],
                        [0.0, -1.0, 0.0],
                        block.color(BlockFace::Bottom),
                    );
                }

                // North (+Z)
                if !is_solid_at(wx, wy, wz + 1) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz + 1.0],
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        block.color(BlockFace::North),
                    );
                }

                // South (-Z)
                if !is_solid_at(wx, wy, wz - 1) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy, fz],
                            [fx, fy, fz],
                            [fx, fy + 1.0, fz],
                            [fx + 1.0, fy + 1.0, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        block.color(BlockFace::South),
                    );
                }

                // East (+X)
                if !is_solid_at(wx + 1, wy, wz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy, fz + 1.0],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + 1.0, fz],
                            [fx + 1.0, fy + 1.0, fz + 1.0],
                        ],
                        [1.0, 0.0, 0.0],
                        block.color(BlockFace::East),
                    );
                }

                // West (-X)
                if !is_solid_at(wx - 1, wy, wz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz],
                            [fx, fy, fz + 1.0],
                            [fx, fy + 1.0, fz + 1.0],
                            [fx, fy + 1.0, fz],
                        ],
                        [-1.0, 0.0, 0.0],
                        block.color(BlockFace::West),
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    Some(mesh)
}

fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    verts: [[f32; 3]; 4],
    norm: [f32; 3],
    color: [f32; 4],
) {
    let start_idx = positions.len() as u32;

    for v in &verts {
        positions.push(*v);
        normals.push(norm);
        colors.push(color);
    }

    indices.extend_from_slice(&[
        start_idx,
        start_idx + 1,
        start_idx + 2,
        start_idx,
        start_idx + 2,
        start_idx + 3,
    ]);
}
