use bevy::asset::RenderAssetUsages;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::{BlockFace, BlockType};

/// In-memory geometry buffer builder for chunk meshes.
#[derive(Default, Debug)]
pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub uvs_1: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
}

impl MeshBuffers {
    #[must_use]
    pub fn with_capacity(vert_cap: usize, idx_cap: usize) -> Self {
        Self {
            positions: Vec::with_capacity(vert_cap),
            normals: Vec::with_capacity(vert_cap),
            uvs: Vec::with_capacity(vert_cap),
            uvs_1: Vec::with_capacity(vert_cap),
            indices: Vec::with_capacity(idx_cap),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    #[must_use]
    pub fn to_mesh(self) -> Option<Mesh> {
        if self.positions.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.uvs_1);
        mesh.insert_indices(Indices::U16(self.indices));
        Some(mesh)
    }
}

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
}

/// Simplifies ore and block types into base rock for distant LOD rendering.
/// At distance, ore veins (coal, iron, gold, diamond) merge into stone to reduce visual noise and quad fragmentation.
#[inline(always)]
pub const fn simplify_block_for_lod(block: BlockType) -> BlockType {
    match block {
        BlockType::CoalOre
        | BlockType::IronOre
        | BlockType::GoldOre
        | BlockType::DiamondOre
        | BlockType::Cobblestone => BlockType::Stone,
        other => other,
    }
}

#[inline(always)]
pub fn add_quad(
    buffers: &mut MeshBuffers,
    verts: [[f32; 3]; 4],
    norm: [f32; 3],
    quad_uvs: [[f32; 2]; 4],
    layer: f32,
    shade: f32,
) {
    debug_assert!(buffers.positions.len() <= (u16::MAX - 4) as usize, "Chunk vertex count exceeds u16::MAX");
    let start_idx = buffers.positions.len() as u16;

    buffers.positions.extend_from_slice(&verts);
    buffers.normals.extend_from_slice(&[norm; 4]);
    buffers.uvs_1.extend_from_slice(&[[layer, shade]; 4]);
    buffers.uvs.extend_from_slice(&quad_uvs);

    // Standard Bevy Cuboid CCW winding: 0, 1, 2, 2, 3, 0
    buffers.indices.extend_from_slice(&[
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
    buffers: &mut MeshBuffers,
    verts: [[f32; 3]; 3],
    norm: [f32; 3],
    tri_uvs: [[f32; 2]; 3],
    layer: f32,
    shade: f32,
) {
    debug_assert!(buffers.positions.len() <= (u16::MAX - 3) as usize, "Chunk vertex count exceeds u16::MAX");
    let start_idx = buffers.positions.len() as u16;

    buffers.positions.extend_from_slice(&verts);
    buffers.normals.extend_from_slice(&[norm; 3]);
    buffers.uvs_1.extend_from_slice(&[[layer, shade]; 3]);
    buffers.uvs.extend_from_slice(&tri_uvs);

    buffers.indices.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2]);
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
