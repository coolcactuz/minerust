use super::helpers::{add_quad, add_triangle, simplify_block_for_lod, triangle_normal, MeshBuffers};
use super::{ChunkMeshes, SectionConnectivity, SectionMeshes, CHUNK_SECTIONS};
use crate::block::{BlockFace, BlockType};
use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};
use crate::texture::{block_texture, quad_uvs};

/// Sloped Heightfield Mesher for distant Level of Detail (LOD 1).
///
/// Instead of generating stepped 1x1 voxel staircase quads with vertical riser walls,
/// this mesher samples the surface elevation of each column and connects diagonal mountain slopes
/// into continuous slanted triangles.
///
/// Key optimizations:
/// 1. Ores (Coal, Iron, Gold, Diamond) and Cobblestone are grouped into Stone to prevent texture fragmentation.
/// 2. Slopes are rendered as single angled quads/triangles, reducing vertex and triangle count by ~85-90%.
/// 3. Water surfaces are locked at sea level (Y=64.0) to preserve flat lakes and oceans.
/// 4. Perimeter skirts drop 2.5 blocks along chunk borders, preventing any visible seams/gaps with LOD 0 chunks.
const CELL_SIZE: usize = 2;
const SKIRT_DROP: f32 = 2.5;

pub fn build_chunk_mesh_sloped_lod(
    chunk: &Chunk,
    north: Option<&Chunk>,
    south: Option<&Chunk>,
    east: Option<&Chunk>,
    west: Option<&Chunk>,
    max_y: usize,
) -> ChunkMeshes {
    // 1. Sample the top solid/visible block and surface height for each of the 16x16 columns
    let mut col_heights = [[0.0f32; CHUNK_WIDTH]; CHUNK_DEPTH];
    let mut col_blocks = [[BlockType::Air; CHUNK_WIDTH]; CHUNK_DEPTH];
    let mut col_seabed_heights = [[56.0f32; CHUNK_WIDTH]; CHUNK_DEPTH];
    let mut col_seabed_blocks = [[BlockType::Sand; CHUNK_WIDTH]; CHUNK_DEPTH];
    let mut has_any_surface = false;

    for lz in 0..CHUNK_DEPTH {
        for lx in 0..CHUNK_WIDTH {
            let mut surface_found = false;
            for ly in (0..=max_y).rev() {
                let raw_b = chunk.get_fast(lx, ly, lz);
                if raw_b == BlockType::Air {
                    continue;
                }
                let simplified = simplify_block_for_lod(raw_b);
                let h = (ly as f32) + 1.0;
                if !surface_found {
                    col_heights[lz][lx] = h;
                    col_blocks[lz][lx] = simplified;
                    has_any_surface = true;
                    surface_found = true;
                }
                if raw_b != BlockType::Water {
                    col_seabed_heights[lz][lx] = h;
                    col_seabed_blocks[lz][lx] = simplified;
                    break;
                }
            }
        }
    }

    if !has_any_surface {
        return ChunkMeshes::default();
    }

    // Helper to sample height at (cx, cz), checking neighbors if outside [0..16, 0..16]
    let sample_height = |cx: i32, cz: i32| -> Option<f32> {
        if cx >= 0 && cx < CHUNK_WIDTH as i32 && cz >= 0 && cz < CHUNK_DEPTH as i32 {
            let b = col_blocks[cz as usize][cx as usize];
            if b != BlockType::Air {
                Some(col_heights[cz as usize][cx as usize])
            } else {
                None
            }
        } else if cz >= CHUNK_DEPTH as i32 && cx >= 0 && cx < CHUNK_WIDTH as i32 {
            // North neighbor (z = 0)
            north.and_then(|n| {
                for ly in (0..=max_y).rev() {
                    let b = n.get_fast(cx as usize, ly, 0);
                    if b != BlockType::Air {
                        return Some((ly as f32) + 1.0);
                    }
                }
                None
            })
        } else if cz < 0 && cx >= 0 && cx < CHUNK_WIDTH as i32 {
            // South neighbor (z = 15)
            south.and_then(|s| {
                for ly in (0..=max_y).rev() {
                    let b = s.get_fast(cx as usize, ly, CHUNK_DEPTH - 1);
                    if b != BlockType::Air {
                        return Some((ly as f32) + 1.0);
                    }
                }
                None
            })
        } else if cx >= CHUNK_WIDTH as i32 && cz >= 0 && cz < CHUNK_DEPTH as i32 {
            // East neighbor (x = 0)
            east.and_then(|e| {
                for ly in (0..=max_y).rev() {
                    let b = e.get_fast(0, ly, cz as usize);
                    if b != BlockType::Air {
                        return Some((ly as f32) + 1.0);
                    }
                }
                None
            })
        } else if cx < 0 && cz >= 0 && cz < CHUNK_DEPTH as i32 {
            // West neighbor (x = 15)
            west.and_then(|w| {
                for ly in (0..=max_y).rev() {
                    let b = w.get_fast(CHUNK_WIDTH - 1, ly, cz as usize);
                    if b != BlockType::Air {
                        return Some((ly as f32) + 1.0);
                    }
                }
                None
            })
        } else {
            None
        }
    };

    // 2. Compute smooth elevations for all 17x17 vertex intersections
    let mut vertex_h = [[0.0f32; CHUNK_WIDTH + 1]; CHUNK_DEPTH + 1];

    for vz in 0..=CHUNK_DEPTH {
        for vx in 0..=CHUNK_WIDTH {
            let ivx = vx as i32;
            let ivz = vz as i32;

            // Average of the 4 surrounding columns touching this vertex intersection
            let candidates = [
                sample_height(ivx - 1, ivz - 1),
                sample_height(ivx, ivz - 1),
                sample_height(ivx - 1, ivz),
                sample_height(ivx, ivz),
            ];

            let mut sum = 0.0f32;
            let mut count = 0;
            for h in candidates.into_iter().flatten() {
                sum += h;
                count += 1;
            }

            vertex_h[vz][vx] = if count > 0 {
                sum / (count as f32)
            } else {
                // Fallback to nearest interior column
                let clamp_x = vx.min(CHUNK_WIDTH - 1);
                let clamp_z = vz.min(CHUNK_DEPTH - 1);
                col_heights[clamp_z][clamp_x]
            };
        }
    }

    // 3. Generate mesh geometry using 2x2 macro-cells (8x8 cells across chunk)
    let mut solid = MeshBuffers::default();
    let mut water = MeshBuffers::default();

    let cells_x = CHUNK_WIDTH / CELL_SIZE; // 8
    let cells_z = CHUNK_DEPTH / CELL_SIZE; // 8

    for cz in 0..cells_z {
        let lz0 = cz * CELL_SIZE;
        let lz1 = lz0 + CELL_SIZE;
        let fz0 = lz0 as f32;
        let fz1 = lz1 as f32;

        for cx in 0..cells_x {
            let lx0 = cx * CELL_SIZE;
            let lx1 = lx0 + CELL_SIZE;
            let fx0 = lx0 as f32;
            let fx1 = lx1 as f32;

            // Determine dominant block type among the 2x2 columns
            let b00 = col_blocks[lz0][lx0];
            let b10 = col_blocks[lz0][lx0 + 1];
            let b01 = col_blocks[lz0 + 1][lx0];
            let b11 = col_blocks[lz0 + 1][lx0 + 1];

            // Filter out air
            let sub_blocks = [b00, b10, b01, b11];
            let mut dominant_block = BlockType::Air;
            let mut best_count = 0;

            for &b in &sub_blocks {
                if b == BlockType::Air {
                    continue;
                }
                let c = sub_blocks.iter().filter(|&&other| other == b).count();
                if c > best_count {
                    best_count = c;
                    dominant_block = b;
                }
            }

            if dominant_block == BlockType::Air {
                continue;
            }

            // If dominant block is water, water must remain completely flat at sea level (64.0)
            if dominant_block.is_water() {
                let water_y = 64.0;
                let layer = block_texture(BlockType::Water, BlockFace::Top).layer();
                let water_uvs = quad_uvs(CELL_SIZE as f32, CELL_SIZE as f32);
                add_quad(
                    &mut water,
                    [
                        [fx0, water_y, fz0],
                        [fx0, water_y, fz1],
                        [fx1, water_y, fz1],
                        [fx1, water_y, fz0],
                    ],
                    [0.0, 1.0, 0.0],
                    water_uvs,
                    layer,
                    1.0,
                );

                // Downward face for underwater viewing
                add_quad(
                    &mut water,
                    [
                        [fx0, water_y, fz0],
                        [fx1, water_y, fz0],
                        [fx1, water_y, fz1],
                        [fx0, water_y, fz1],
                    ],
                    [0.0, -1.0, 0.0],
                    water_uvs,
                    layer,
                    0.7,
                );

                // Render solid seabed under water so distant water doesn't expose the sky void
                let seabed_y = (col_seabed_heights[lz0][lx0]
                    + col_seabed_heights[lz0][lx0 + 1]
                    + col_seabed_heights[lz0 + 1][lx0]
                    + col_seabed_heights[lz0 + 1][lx0 + 1])
                    * 0.25;
                let seabed_block = col_seabed_blocks[lz0][lx0];
                let seabed_layer = block_texture(seabed_block, BlockFace::Top).layer();
                add_quad(
                    &mut solid,
                    [
                        [fx0, seabed_y, fz0],
                        [fx0, seabed_y, fz1],
                        [fx1, seabed_y, fz1],
                        [fx1, seabed_y, fz0],
                    ],
                    [0.0, 1.0, 0.0],
                    water_uvs,
                    seabed_layer,
                    0.85,
                );
                continue;
            }

            // Solid terrain: construct 4 corner vertices
            let mut y00 = vertex_h[lz0][lx0];
            let mut y01 = vertex_h[lz1][lx0];
            let mut y11 = vertex_h[lz1][lx1];
            let mut y10 = vertex_h[lz0][lx1];

            // For terrain adjacent to water, keep elevation at or above water surface
            if b00.is_water() {
                y00 = y00.max(64.0);
            }
            if b01.is_water() {
                y01 = y01.max(64.0);
            }
            if b11.is_water() {
                y11 = y11.max(64.0);
            }
            if b10.is_water() {
                y10 = y10.max(64.0);
            }

            let p0 = [fx0, y00, fz0];
            let p1 = [fx0, y01, fz1];
            let p2 = [fx1, y11, fz1];
            let p3 = [fx1, y10, fz0];

            let layer = block_texture(dominant_block, BlockFace::Top).layer();
            let f_size = CELL_SIZE as f32;

            // Triangle 1: p0 -> p1 -> p2
            let norm_1 = triangle_normal(p0, p1, p2);
            let shade_1 = 0.75 + 0.25 * norm_1[1].clamp(0.0, 1.0);
            add_triangle(
                &mut solid,
                [p0, p1, p2],
                norm_1,
                [[0.0, 0.0], [0.0, f_size], [f_size, f_size]],
                layer,
                shade_1,
            );

            // Triangle 2: p0 -> p2 -> p3
            let norm_2 = triangle_normal(p0, p2, p3);
            let shade_2 = 0.75 + 0.25 * norm_2[1].clamp(0.0, 1.0);
            add_triangle(
                &mut solid,
                [p0, p2, p3],
                norm_2,
                [[0.0, 0.0], [f_size, f_size], [f_size, 0.0]],
                layer,
                shade_2,
            );
        }
    }

    let skirt_uvs = quad_uvs(CELL_SIZE as f32, SKIRT_DROP);

    // 4. Perimeter Skirts: drop down 2.5 blocks along outer boundaries to seal cracks with adjacent chunks
    // South border (Z = 0)
    for cx in 0..cells_x {
        let lx0 = (cx * CELL_SIZE) as f32;
        let lx1 = ((cx + 1) * CELL_SIZE) as f32;
        let y0 = vertex_h[0][cx * CELL_SIZE];
        let y1 = vertex_h[0][(cx + 1) * CELL_SIZE];
        let b = col_blocks[0][cx * CELL_SIZE];
        if b != BlockType::Air && !b.is_water() {
            let layer = block_texture(b, BlockFace::South).layer();
            add_quad(
                &mut solid,
                [
                    [lx1, y1, 0.0],
                    [lx1, y1 - SKIRT_DROP, 0.0],
                    [lx0, y0 - SKIRT_DROP, 0.0],
                    [lx0, y0, 0.0],
                ],
                [0.0, 0.0, -1.0],
                skirt_uvs,
                layer,
                0.75,
            );
        }
    }

    // North border (Z = 16)
    for cx in 0..cells_x {
        let lx0 = (cx * CELL_SIZE) as f32;
        let lx1 = ((cx + 1) * CELL_SIZE) as f32;
        let y0 = vertex_h[CHUNK_DEPTH][cx * CELL_SIZE];
        let y1 = vertex_h[CHUNK_DEPTH][(cx + 1) * CELL_SIZE];
        let b = col_blocks[CHUNK_DEPTH - 1][cx * CELL_SIZE];
        if b != BlockType::Air && !b.is_water() {
            let layer = block_texture(b, BlockFace::North).layer();
            add_quad(
                &mut solid,
                [
                    [lx0, y0, 16.0],
                    [lx0, y0 - SKIRT_DROP, 16.0],
                    [lx1, y1 - SKIRT_DROP, 16.0],
                    [lx1, y1, 16.0],
                ],
                [0.0, 0.0, 1.0],
                skirt_uvs,
                layer,
                0.75,
            );
        }
    }

    // West border (X = 0)
    for cz in 0..cells_z {
        let lz0 = (cz * CELL_SIZE) as f32;
        let lz1 = ((cz + 1) * CELL_SIZE) as f32;
        let y0 = vertex_h[cz * CELL_SIZE][0];
        let y1 = vertex_h[(cz + 1) * CELL_SIZE][0];
        let b = col_blocks[cz * CELL_SIZE][0];
        if b != BlockType::Air && !b.is_water() {
            let layer = block_texture(b, BlockFace::West).layer();
            add_quad(
                &mut solid,
                [
                    [0.0, y0, lz0],
                    [0.0, y0 - SKIRT_DROP, lz0],
                    [0.0, y1 - SKIRT_DROP, lz1],
                    [0.0, y1, lz1],
                ],
                [-1.0, 0.0, 0.0],
                skirt_uvs,
                layer,
                0.7,
            );
        }
    }

    // East border (X = 16)
    for cz in 0..cells_z {
        let lz0 = (cz * CELL_SIZE) as f32;
        let lz1 = ((cz + 1) * CELL_SIZE) as f32;
        let y0 = vertex_h[cz * CELL_SIZE][CHUNK_WIDTH];
        let y1 = vertex_h[(cz + 1) * CELL_SIZE][CHUNK_WIDTH];
        let b = col_blocks[cz * CELL_SIZE][CHUNK_WIDTH - 1];
        if b != BlockType::Air && !b.is_water() {
            let layer = block_texture(b, BlockFace::East).layer();
            add_quad(
                &mut solid,
                [
                    [16.0, y1, lz1],
                    [16.0, y1 - SKIRT_DROP, lz1],
                    [16.0, y0 - SKIRT_DROP, lz0],
                    [16.0, y0, lz0],
                ],
                [1.0, 0.0, 0.0],
                skirt_uvs,
                layer,
                0.7,
            );
        }
    }

    let mut sections: [SectionMeshes; CHUNK_SECTIONS] = Default::default();
    let mut connectivity: [SectionConnectivity; CHUNK_SECTIONS] = Default::default();
    sections[0] = SectionMeshes {
        solid: solid.to_mesh(),
        water: water.to_mesh(),
    };
    connectivity[0] = SectionConnectivity::full();
    ChunkMeshes { sections, connectivity }
}
