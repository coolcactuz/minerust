use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::{BlockFace, BlockType};
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use crate::texture::{block_texture, get_tile_uvs};

#[inline(always)]
fn should_render_face(block: BlockType, neighbor: BlockType, _face: BlockFace) -> bool {
    if block == neighbor {
        return false;
    }

    if block.is_water() {
        neighbor == BlockType::Air || neighbor == BlockType::Glass
    } else if block.is_solid() {
        neighbor.is_transparent()
    } else {
        false
    }
}

#[inline(always)]
fn can_merge_blocks(b1: BlockType, b2: BlockType) -> bool {
    b1 == b2
        || (matches!(b1, BlockType::Sand | BlockType::Gravel)
            && matches!(b2, BlockType::Sand | BlockType::Gravel))
}

pub fn build_chunk_mesh(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y_skip: bool,
    greedy: bool,
) -> Option<Mesh> {
    let max_y = if max_y_skip {
        chunk.max_y.min(CHUNK_HEIGHT - 1)
    } else {
        CHUNK_HEIGHT - 1
    };

    if greedy {
        build_chunk_mesh_greedy(chunk, north, south, east, west, max_y)
    } else {
        build_chunk_mesh_standard(chunk, north, south, east, west, max_y)
    }
}

/// Standard 1x1 voxel face mesher
fn build_chunk_mesh_standard(
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
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(2048);
    let mut indices: Vec<u32> = Vec::with_capacity(3072);

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

                    // For surface water, also render downward-facing ceiling quad for underwater viewing
                    if block.is_water() {
                        add_quad(
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut colors,
                            &mut indices,
                            [
                                [fx, fy + 1.0, fz],
                                [fx + 1.0, fy + 1.0, fz],
                                [fx + 1.0, fy + 1.0, fz + 1.0],
                                [fx, fy + 1.0, fz + 1.0],
                            ],
                            [0.0, -1.0, 0.0],
                            tile_uvs,
                            0.7,
                        );
                    }
                }

                // Bottom (-Y)
                if ly > 0 {
                    let bottom_neighbor = chunk.get_fast(lx, ly - 1, lz);
                    if should_render_face(block, bottom_neighbor, BlockFace::Bottom) {
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
                // North / Front (+Z)
                let north_neighbor = if lz + 1 < CHUNK_DEPTH {
                    chunk.get_fast(lx, ly, lz + 1)
                } else if let Some(n) = north {
                    n.get_fast(lx, ly, 0)
                } else {
                    if block.is_water() { BlockType::Water } else { BlockType::Air }
                };
                if should_render_face(block, north_neighbor, BlockFace::North) {
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
                    if block.is_water() { BlockType::Water } else { BlockType::Air }
                };
                if should_render_face(block, south_neighbor, BlockFace::South) {
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
                    if block.is_water() { BlockType::Water } else { BlockType::Air }
                };
                if should_render_face(block, east_neighbor, BlockFace::East) {
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
                    if block.is_water() { BlockType::Water } else { BlockType::Air }
                };
                if should_render_face(block, west_neighbor, BlockFace::West) {
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

/// Greedy voxel face mesher that merges coplanar adjacent faces into single quads
fn build_chunk_mesh_greedy(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y: usize,
) -> Option<Mesh> {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(1024);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(1024);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(1024);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(1024);
    let mut indices: Vec<u32> = Vec::with_capacity(1536);

    // 1. TOP (+Y) Faces: horizontal slices (X = 0..16, Z = 0..16)
    let mut top_mask = [None; CHUNK_WIDTH * CHUNK_DEPTH];
    for ly in 0..=max_y {
        let fy = ly as f32;
        let mut any_face = false;
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let top_neighbor = if ly + 1 < CHUNK_HEIGHT {
                        chunk.get_fast(lx, ly + 1, lz)
                    } else {
                        BlockType::Air
                    };
                    if should_render_face(block, top_neighbor, BlockFace::Top) {
                        top_mask[lz * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                top_mask[lz * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        // Greedy merge on X (width) and Z (depth)
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = top_mask[lz * CHUNK_WIDTH + lx] {
                    // Find width along X
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && top_mask[lz * CHUNK_WIDTH + (lx + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    // Find depth along Z
                    let mut h = 1;
                    'outer_top: while lz + h < CHUNK_DEPTH {
                        for k in 0..w {
                            if !top_mask[(lz + h) * CHUNK_WIDTH + (lx + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_top;
                            }
                        }
                        h += 1;
                    }
                    // Clear mask
                    for dz in 0..h {
                        for dx in 0..w {
                            top_mask[(lz + dz) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fz = lz as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::Top));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + fh],
                            [fx + fw, fy + 1.0, fz + fh],
                            [fx + fw, fy + 1.0, fz],
                        ],
                        [0.0, 1.0, 0.0],
                        tile_uvs,
                        1.0,
                    );

                    // For surface water, also render downward-facing ceiling quad for underwater viewing
                    if block.is_water() {
                        add_quad(
                            &mut positions,
                            &mut normals,
                            &mut uvs,
                            &mut colors,
                            &mut indices,
                            [
                                [fx, fy + 1.0, fz],
                                [fx + fw, fy + 1.0, fz],
                                [fx + fw, fy + 1.0, fz + fh],
                                [fx, fy + 1.0, fz + fh],
                            ],
                            [0.0, -1.0, 0.0],
                            tile_uvs,
                            0.7,
                        );
                    }
                }
            }
        }
    }

    // 2. BOTTOM (-Y) Faces: horizontal slices (X = 0..16, Z = 0..16)
    let mut bot_mask = [None; CHUNK_WIDTH * CHUNK_DEPTH];
    for ly in 1..=max_y {
        let fy = ly as f32;
        let mut any_face = false;
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let bottom_neighbor = chunk.get_fast(lx, ly - 1, lz);
                    if should_render_face(block, bottom_neighbor, BlockFace::Bottom) {
                        bot_mask[lz * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                bot_mask[lz * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = bot_mask[lz * CHUNK_WIDTH + lx] {
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && bot_mask[lz * CHUNK_WIDTH + (lx + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_bot: while lz + h < CHUNK_DEPTH {
                        for k in 0..w {
                            if !bot_mask[(lz + h) * CHUNK_WIDTH + (lx + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_bot;
                            }
                        }
                        h += 1;
                    }
                    for dz in 0..h {
                        for dx in 0..w {
                            bot_mask[(lz + dz) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fz = lz as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::Bottom));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy, fz + fh],
                            [fx, fy, fz],
                            [fx + fw, fy, fz],
                            [fx + fw, fy, fz + fh],
                        ],
                        [0.0, -1.0, 0.0],
                        tile_uvs,
                        0.5,
                    );
                }
            }
        }
    }

    // Allocate reusable mask for vertical side slices (dim_u = 16, dim_v = max_y + 1)
    let slice_height = max_y + 1;
    let mut side_mask: Vec<Option<BlockType>> = vec![None; CHUNK_WIDTH * slice_height];

    // 3. NORTH (+Z) Faces: slice along Z (0..CHUNK_DEPTH), grid: X = 0..16, Y = 0..=max_y
    for lz in 0..CHUNK_DEPTH {
        let fz = lz as f32;
        let mut any_face = false;
        for ly in 0..=max_y {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let north_neighbor = if lz + 1 < CHUNK_DEPTH {
                        chunk.get_fast(lx, ly, lz + 1)
                    } else if let Some(n) = north {
                        n.get_fast(lx, ly, 0)
                    } else {
                        if block.is_water() { BlockType::Water } else { BlockType::Air }
                    };
                    if should_render_face(block, north_neighbor, BlockFace::North) {
                        side_mask[ly * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[ly * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        for ly in 0..=max_y {
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = side_mask[ly * CHUNK_WIDTH + lx] {
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && side_mask[ly * CHUNK_WIDTH + (lx + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_north: while ly + h <= max_y {
                        for k in 0..w {
                            if !side_mask[(ly + h) * CHUNK_WIDTH + (lx + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_north;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dx in 0..w {
                            side_mask[(ly + dy) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::North));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + fh, fz + 1.0],
                            [fx, fy, fz + 1.0],
                            [fx + fw, fy, fz + 1.0],
                            [fx + fw, fy + fh, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        tile_uvs,
                        0.85,
                    );
                }
            }
        }
    }

    // 4. SOUTH (-Z) Faces: slice along Z (0..CHUNK_DEPTH), grid: X = 0..16, Y = 0..=max_y
    for lz in 0..CHUNK_DEPTH {
        let fz = lz as f32;
        let mut any_face = false;
        for ly in 0..=max_y {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let south_neighbor = if lz > 0 {
                        chunk.get_fast(lx, ly, lz - 1)
                    } else if let Some(s) = south {
                        s.get_fast(lx, ly, CHUNK_DEPTH - 1)
                    } else {
                        if block.is_water() { BlockType::Water } else { BlockType::Air }
                    };
                    if should_render_face(block, south_neighbor, BlockFace::South) {
                        side_mask[ly * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[ly * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        for ly in 0..=max_y {
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = side_mask[ly * CHUNK_WIDTH + lx] {
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && side_mask[ly * CHUNK_WIDTH + (lx + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_south: while ly + h <= max_y {
                        for k in 0..w {
                            if !side_mask[(ly + h) * CHUNK_WIDTH + (lx + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_south;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dx in 0..w {
                            side_mask[(ly + dy) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::South));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + fw, fy + fh, fz],
                            [fx + fw, fy, fz],
                            [fx, fy, fz],
                            [fx, fy + fh, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        tile_uvs,
                        0.85,
                    );
                }
            }
        }
    }

    // 5. EAST (+X) Faces: slice along X (0..CHUNK_WIDTH), grid: Z = 0..16, Y = 0..=max_y
    for lx in 0..CHUNK_WIDTH {
        let fx = lx as f32;
        let mut any_face = false;
        for ly in 0..=max_y {
            for lz in 0..CHUNK_DEPTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let east_neighbor = if lx + 1 < CHUNK_WIDTH {
                        chunk.get_fast(lx + 1, ly, lz)
                    } else if let Some(e) = east {
                        e.get_fast(0, ly, lz)
                    } else {
                        if block.is_water() { BlockType::Water } else { BlockType::Air }
                    };
                    if should_render_face(block, east_neighbor, BlockFace::East) {
                        side_mask[ly * CHUNK_DEPTH + lz] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[ly * CHUNK_DEPTH + lz] = None;
            }
        }

        if !any_face {
            continue;
        }

        for ly in 0..=max_y {
            for lz in 0..CHUNK_DEPTH {
                if let Some(block) = side_mask[ly * CHUNK_DEPTH + lz] {
                    let mut w = 1;
                    while lz + w < CHUNK_DEPTH
                        && side_mask[ly * CHUNK_DEPTH + (lz + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_east: while ly + h <= max_y {
                        for k in 0..w {
                            if !side_mask[(ly + h) * CHUNK_DEPTH + (lz + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_east;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dz in 0..w {
                            side_mask[(ly + dy) * CHUNK_DEPTH + (lz + dz)] = None;
                        }
                    }

                    let fz = lz as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::East));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx + 1.0, fy + fh, fz + fw],
                            [fx + 1.0, fy, fz + fw],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + fh, fz],
                        ],
                        [1.0, 0.0, 0.0],
                        tile_uvs,
                        0.7,
                    );
                }
            }
        }
    }

    // 6. WEST (-X) Faces: slice along X (0..CHUNK_WIDTH), grid: Z = 0..16, Y = 0..=max_y
    for lx in 0..CHUNK_WIDTH {
        let fx = lx as f32;
        let mut any_face = false;
        for ly in 0..=max_y {
            for lz in 0..CHUNK_DEPTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let west_neighbor = if lx > 0 {
                        chunk.get_fast(lx - 1, ly, lz)
                    } else if let Some(w) = west {
                        w.get_fast(CHUNK_WIDTH - 1, ly, lz)
                    } else {
                        if block.is_water() { BlockType::Water } else { BlockType::Air }
                    };
                    if should_render_face(block, west_neighbor, BlockFace::West) {
                        side_mask[ly * CHUNK_DEPTH + lz] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[ly * CHUNK_DEPTH + lz] = None;
            }
        }

        if !any_face {
            continue;
        }

        for ly in 0..=max_y {
            for lz in 0..CHUNK_DEPTH {
                if let Some(block) = side_mask[ly * CHUNK_DEPTH + lz] {
                    let mut w = 1;
                    while lz + w < CHUNK_DEPTH
                        && side_mask[ly * CHUNK_DEPTH + (lz + w)]
                            .map_or(false, |b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_west: while ly + h <= max_y {
                        for k in 0..w {
                            if !side_mask[(ly + h) * CHUNK_DEPTH + (lz + k)]
                                .map_or(false, |b| can_merge_blocks(block, b))
                            {
                                break 'outer_west;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dz in 0..w {
                            side_mask[(ly + dy) * CHUNK_DEPTH + (lz + dz)] = None;
                        }
                    }

                    let fz = lz as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let tile_uvs = get_tile_uvs(block_texture(block, BlockFace::West));
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        [
                            [fx, fy + fh, fz],
                            [fx, fy, fz],
                            [fx, fy, fz + fw],
                            [fx, fy + fh, fz + fw],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy_meshing_reduces_vertices() {
        let mut chunk = Chunk::new();
        // Fill a 4x4 area of Stone blocks at y = 10
        for lx in 0..4 {
            for lz in 0..4 {
                chunk.set(lx, 10, lz, BlockType::Stone);
            }
        }

        let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
        let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();

        let std_verts = mesh_standard.count_vertices();
        let greedy_verts = mesh_greedy.count_vertices();

        assert_eq!(std_verts, 192);
        assert_eq!(greedy_verts, 24);
        assert!(greedy_verts < std_verts);
    }

    #[test]
    fn test_water_ocean_renders_only_on_surface() {
        let mut chunk = Chunk::new();
        // Ocean seabed: fill y = 0..=9 with Bedrock
        for lx in 0..CHUNK_WIDTH {
            for lz in 0..CHUNK_DEPTH {
                for ly in 0..=9 {
                    chunk.set(lx as i32, ly as i32, lz as i32, BlockType::Bedrock);
                }
            }
        }
        // Ocean water: fill y = 10..=20 with Water
        for lx in 0..CHUNK_WIDTH {
            for lz in 0..CHUNK_DEPTH {
                for ly in 10..=20 {
                    chunk.set(lx as i32, ly as i32, lz as i32, BlockType::Water);
                }
            }
        }

        // Greedy meshing of this ocean chunk
        let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
        // The ocean water only renders at the surface (y = 20): 1 top quad (4 verts) + 1 underside quad (4 verts) = 8 water verts!
        // Bedrock renders 1 top quad (4 verts) against water, 1 bottom quad (4 verts) at y=0, and 4 greedy side quads (16 verts) on chunk edges.
        // Total verts for entire 16x16 chunk with 20 layers: only 28 vertices (7 quads)!
        assert_eq!(mesh.count_vertices(), 28);
    }

    #[test]
    fn test_waterfall_renders_sides_in_air() {
        let mut chunk = Chunk::new();
        // 1x1 vertical column of water (waterfall) at (5, y, 5) from y = 10 to y = 12 surrounded by Air
        for ly in 10..=12 {
            chunk.set(5, ly, 5, BlockType::Water);
        }
        let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();
        // The waterfall column is surrounded by Air, so it renders Top + ceiling + bottom + 4 sides!
        assert!(mesh.count_vertices() > 8);
    }

    #[test]
    fn test_seabed_sand_gravel_greedy_merging() {
        let mut chunk = Chunk::new();
        // Create an alternating checkerboard of Sand and Gravel on the seabed at y = 10, covered with Water at y = 11
        for lx in 0..4 {
            for lz in 0..4 {
                let block = if (lx + lz) % 2 == 0 {
                    BlockType::Sand
                } else {
                    BlockType::Gravel
                };
                chunk.set(lx, 10, lz, block);
                chunk.set(lx, 11, lz, BlockType::Water);
            }
        }

        let mesh_standard = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
        let mesh_greedy = build_chunk_mesh(&chunk, None, None, None, None, true, true).unwrap();

        let std_verts = mesh_standard.count_vertices();
        let greedy_verts = mesh_greedy.count_vertices();

        // In greedy meshing, the alternating sand and gravel seabed merges into a single 4x4 quad!
        assert!(greedy_verts < std_verts);
    }

    #[test]
    fn test_submerged_terrain_renders_against_water_no_holes() {
        let mut chunk = Chunk::new();
        // Sand block at y = 10, Water above it at y = 11
        chunk.set(0, 10, 0, BlockType::Sand);
        chunk.set(0, 11, 0, BlockType::Water);

        let mesh = build_chunk_mesh(&chunk, None, None, None, None, true, false).unwrap();
        // Sand must render its top face at y = 10 against the water at y = 11!
        // All 6 faces of the Sand block are rendered (no holes into the void).
        assert_eq!(mesh.count_vertices(), 40);
    }
}

