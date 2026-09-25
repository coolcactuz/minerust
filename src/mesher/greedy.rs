use super::helpers::{add_quad, can_merge_blocks, should_render_face, MeshBuffers};
use super::{
    compute_section_connectivity, ChunkMeshes, SectionConnectivity, SectionMeshes,
    CHUNK_SECTIONS, SECTION_HEIGHT,
};
use crate::block::{BlockFace, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH, Chunk};
use crate::texture::{block_texture, quad_uvs};

/// Builds the geometry mesh for a single 16x16x16 section using greedy coplanar quad merging.
pub fn build_section_mesh_greedy(
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
    let section_h = y_max - y_start + 1;

    let mut solid = MeshBuffers::with_capacity(512, 768);
    let mut water = MeshBuffers::default();

    // 1. TOP (+Y) Faces: horizontal slices (X = 0..16, Z = 0..16)
    let mut top_mask = [None; CHUNK_WIDTH * CHUNK_DEPTH];
    for ly in y_start..=y_max {
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
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && top_mask[lz * CHUNK_WIDTH + (lx + w)]
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_top: while lz + h < CHUNK_DEPTH {
                        for k in 0..w {
                            if !top_mask[(lz + h) * CHUNK_WIDTH + (lx + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
                            {
                                break 'outer_top;
                            }
                        }
                        h += 1;
                    }
                    for dz in 0..h {
                        for dx in 0..w {
                            top_mask[(lz + dz) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fz = lz as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let layer = block_texture(block, BlockFace::Top).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx, fy + 1.0, fz],
                            [fx, fy + 1.0, fz + fh],
                            [fx + fw, fy + 1.0, fz + fh],
                            [fx + fw, fy + 1.0, fz],
                        ],
                        [0.0, 1.0, 0.0],
                        face_uvs,
                        layer,
                        1.0,
                    );

                    // For surface water, also render downward-facing ceiling quad for underwater viewing
                    if block.is_water() {
                        add_quad(
                            &mut water,
                            [
                                [fx, fy + 1.0, fz],
                                [fx + fw, fy + 1.0, fz],
                                [fx + fw, fy + 1.0, fz + fh],
                                [fx, fy + 1.0, fz + fh],
                            ],
                            [0.0, -1.0, 0.0],
                            face_uvs,
                            layer,
                            0.7,
                        );
                    }
                }
            }
        }
    }

    // 2. BOTTOM (-Y) Faces: horizontal slices (X = 0..16, Z = 0..16)
    let mut bot_mask = [None; CHUNK_WIDTH * CHUNK_DEPTH];
    for ly in y_start.max(1)..=y_max {
        let fy = ly as f32;
        let mut any_face = false;
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
                    let bottom_neighbor = if ly > 0 {
                        chunk.get_fast(lx, ly - 1, lz)
                    } else {
                        BlockType::Air
                    };
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
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_bot: while lz + h < CHUNK_DEPTH {
                        for k in 0..w {
                            if !bot_mask[(lz + h) * CHUNK_WIDTH + (lx + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
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
                    let layer = block_texture(block, BlockFace::Bottom).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx, fy, fz + fh],
                            [fx, fy, fz],
                            [fx + fw, fy, fz],
                            [fx + fw, fy, fz + fh],
                        ],
                        [0.0, -1.0, 0.0],
                        face_uvs,
                        layer,
                        0.5,
                    );
                }
            }
        }
    }

    // Stack-allocated reusable mask for vertical side slices (dim = 16 x 16 = 256 elements)
    let mut side_mask = [None; CHUNK_WIDTH * SECTION_HEIGHT];

    // 3. NORTH (+Z) Faces: slice along Z (0..CHUNK_DEPTH), grid: X = 0..16, Y = y_start..=y_max
    for lz in 0..CHUNK_DEPTH {
        let fz = lz as f32;
        let mut any_face = false;
        for ly in y_start..=y_max {
            let rel_y = ly - y_start;
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
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
                        side_mask[rel_y * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[rel_y * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        for rel_y in 0..section_h {
            let ly = y_start + rel_y;
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = side_mask[rel_y * CHUNK_WIDTH + lx] {
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && side_mask[rel_y * CHUNK_WIDTH + (lx + w)]
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_north: while rel_y + h < section_h {
                        for k in 0..w {
                            if !side_mask[(rel_y + h) * CHUNK_WIDTH + (lx + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
                            {
                                break 'outer_north;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dx in 0..w {
                            side_mask[(rel_y + dy) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let layer = block_texture(block, BlockFace::North).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx, fy + fh, fz + 1.0],
                            [fx, fy, fz + 1.0],
                            [fx + fw, fy, fz + 1.0],
                            [fx + fw, fy + fh, fz + 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        face_uvs,
                        layer,
                        0.85,
                    );
                }
            }
        }
    }

    // 4. SOUTH (-Z) Faces: slice along Z (0..CHUNK_DEPTH), grid: X = 0..16, Y = y_start..=y_max
    for lz in 0..CHUNK_DEPTH {
        let fz = lz as f32;
        let mut any_face = false;
        for ly in y_start..=y_max {
            let rel_y = ly - y_start;
            for lx in 0..CHUNK_WIDTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
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
                        side_mask[rel_y * CHUNK_WIDTH + lx] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[rel_y * CHUNK_WIDTH + lx] = None;
            }
        }

        if !any_face {
            continue;
        }

        for rel_y in 0..section_h {
            let ly = y_start + rel_y;
            for lx in 0..CHUNK_WIDTH {
                if let Some(block) = side_mask[rel_y * CHUNK_WIDTH + lx] {
                    let mut w = 1;
                    while lx + w < CHUNK_WIDTH
                        && side_mask[rel_y * CHUNK_WIDTH + (lx + w)]
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_south: while rel_y + h < section_h {
                        for k in 0..w {
                            if !side_mask[(rel_y + h) * CHUNK_WIDTH + (lx + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
                            {
                                break 'outer_south;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dx in 0..w {
                            side_mask[(rel_y + dy) * CHUNK_WIDTH + (lx + dx)] = None;
                        }
                    }

                    let fx = lx as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let layer = block_texture(block, BlockFace::South).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx + fw, fy + fh, fz],
                            [fx + fw, fy, fz],
                            [fx, fy, fz],
                            [fx, fy + fh, fz],
                        ],
                        [0.0, 0.0, -1.0],
                        face_uvs,
                        layer,
                        0.85,
                    );
                }
            }
        }
    }

    // 5. EAST (+X) Faces: slice along X (0..CHUNK_WIDTH), grid: Z = 0..16, Y = y_start..=y_max
    for lx in 0..CHUNK_WIDTH {
        let fx = lx as f32;
        let mut any_face = false;
        for ly in y_start..=y_max {
            let rel_y = ly - y_start;
            for lz in 0..CHUNK_DEPTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
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
                        side_mask[rel_y * CHUNK_DEPTH + lz] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[rel_y * CHUNK_DEPTH + lz] = None;
            }
        }

        if !any_face {
            continue;
        }

        for rel_y in 0..section_h {
            let ly = y_start + rel_y;
            for lz in 0..CHUNK_DEPTH {
                if let Some(block) = side_mask[rel_y * CHUNK_DEPTH + lz] {
                    let mut w = 1;
                    while lz + w < CHUNK_DEPTH
                        && side_mask[rel_y * CHUNK_DEPTH + (lz + w)]
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_east: while rel_y + h < section_h {
                        for k in 0..w {
                            if !side_mask[(rel_y + h) * CHUNK_DEPTH + (lz + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
                            {
                                break 'outer_east;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dz in 0..w {
                            side_mask[(rel_y + dy) * CHUNK_DEPTH + (lz + dz)] = None;
                        }
                    }

                    let fz = lz as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let layer = block_texture(block, BlockFace::East).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx + 1.0, fy + fh, fz + fw],
                            [fx + 1.0, fy, fz + fw],
                            [fx + 1.0, fy, fz],
                            [fx + 1.0, fy + fh, fz],
                        ],
                        [1.0, 0.0, 0.0],
                        face_uvs,
                        layer,
                        0.7,
                    );
                }
            }
        }
    }

    // 6. WEST (-X) Faces: slice along X (0..CHUNK_WIDTH), grid: Z = 0..16, Y = y_start..=y_max
    for lx in 0..CHUNK_WIDTH {
        let fx = lx as f32;
        let mut any_face = false;
        for ly in y_start..=y_max {
            let rel_y = ly - y_start;
            for lz in 0..CHUNK_DEPTH {
                let block = chunk.get_fast(lx, ly, lz);
                if block != BlockType::Air {
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
                        side_mask[rel_y * CHUNK_DEPTH + lz] = Some(block);
                        any_face = true;
                        continue;
                    }
                }
                side_mask[rel_y * CHUNK_DEPTH + lz] = None;
            }
        }

        if !any_face {
            continue;
        }

        for rel_y in 0..section_h {
            let ly = y_start + rel_y;
            for lz in 0..CHUNK_DEPTH {
                if let Some(block) = side_mask[rel_y * CHUNK_DEPTH + lz] {
                    let mut w = 1;
                    while lz + w < CHUNK_DEPTH
                        && side_mask[rel_y * CHUNK_DEPTH + (lz + w)]
                            .is_some_and(|b| can_merge_blocks(block, b))
                    {
                        w += 1;
                    }
                    let mut h = 1;
                    'outer_west: while rel_y + h < section_h {
                        for k in 0..w {
                            if !side_mask[(rel_y + h) * CHUNK_DEPTH + (lz + k)]
                                .is_some_and(|b| can_merge_blocks(block, b))
                            {
                                break 'outer_west;
                            }
                        }
                        h += 1;
                    }
                    for dy in 0..h {
                        for dz in 0..w {
                            side_mask[(rel_y + dy) * CHUNK_DEPTH + (lz + dz)] = None;
                        }
                    }

                    let fz = lz as f32;
                    let fy = ly as f32;
                    let fw = w as f32;
                    let fh = h as f32;
                    let layer = block_texture(block, BlockFace::West).layer();
                    let face_uvs = quad_uvs(fw, fh);
                    let target = if block.is_water() {
                        &mut water
                    } else {
                        &mut solid
                    };
                    add_quad(
                        target,
                        [
                            [fx, fy + fh, fz],
                            [fx, fy, fz],
                            [fx, fy, fz + fw],
                            [fx, fy + fh, fz + fw],
                        ],
                        [-1.0, 0.0, 0.0],
                        face_uvs,
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

/// Builds the geometry mesh for an entire 16x128x16 chunk column by meshing all 8 sub-chunk sections.
pub fn build_chunk_mesh_greedy(
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
        *section = build_section_mesh_greedy(chunk, north, south, east, west, sy, max_y);
        connectivity[sy] = compute_section_connectivity(chunk, sy);
    }
    ChunkMeshes { sections, connectivity }
}
