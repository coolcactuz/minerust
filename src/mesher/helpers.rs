use crate::block::{BlockFace, BlockType};

#[inline(always)]
pub const fn should_render_face(block: BlockType, neighbor: BlockType, _face: BlockFace) -> bool {
    if block as u8 == neighbor as u8 {
        return false;
    }

    if block.is_water() {
        matches!(neighbor, BlockType::Air | BlockType::Glass)
    } else if block.is_solid() {
        neighbor.is_transparent()
    } else {
        false
    }
}

#[inline(always)]
pub const fn can_merge_blocks(b1: BlockType, b2: BlockType) -> bool {
    b1 as u8 == b2 as u8
        || (matches!(b1, BlockType::Sand | BlockType::Gravel)
            && matches!(b2, BlockType::Sand | BlockType::Gravel))
}

/// Simplifies ore and block types into base rock/sand for distant LOD rendering.
/// At distance, ore veins (coal, iron, gold, diamond) merge into stone to reduce visual noise and quad fragmentation.
#[inline(always)]
pub const fn simplify_block_for_lod(block: BlockType) -> BlockType {
    match block {
        BlockType::CoalOre
        | BlockType::IronOre
        | BlockType::GoldOre
        | BlockType::DiamondOre
        | BlockType::Cobblestone => BlockType::Stone,
        BlockType::Gravel => BlockType::Sand,
        other => other,
    }
}

#[inline(always)]
pub fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    uvs_1: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    verts: [[f32; 3]; 4],
    norm: [f32; 3],
    quad_uvs: [[f32; 2]; 4],
    layer: f32,
    shade: f32,
) {
    let start_idx = positions.len() as u32;

    positions.extend_from_slice(&verts);
    normals.extend_from_slice(&[norm; 4]);
    colors.extend_from_slice(&[[shade, shade, shade, 1.0]; 4]);
    uvs_1.extend_from_slice(&[[layer, 0.0]; 4]);
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

#[inline(always)]
pub fn add_triangle(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    uvs_1: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    verts: [[f32; 3]; 3],
    norm: [f32; 3],
    tri_uvs: [[f32; 2]; 3],
    layer: f32,
    shade: f32,
) {
    let start_idx = positions.len() as u32;

    positions.extend_from_slice(&verts);
    normals.extend_from_slice(&[norm; 3]);
    colors.extend_from_slice(&[[shade, shade, shade, 1.0]; 3]);
    uvs_1.extend_from_slice(&[[layer, 0.0]; 3]);
    uvs.extend_from_slice(&tri_uvs);

    indices.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2]);
}

#[inline(always)]
pub fn triangle_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let nx = u[1] * v[2] - u[2] * v[1];
    let ny = u[2] * v[0] - u[0] * v[2];
    let nz = u[0] * v[1] - u[1] * v[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-5 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}
