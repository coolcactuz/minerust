use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::chunk::{CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};

/// Absolute 3D voxel coordinate in the world grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos(pub IVec3);

/// 2D chunk coordinate index in the horizontal world grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos(pub IVec2);

/// Chunk-local voxel offset within a single chunk (x: 0..16, y: 0..128, z: 0..16).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalBlockPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

/// Continuous world translation position.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldPos(pub Vec3);

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    #[inline]
    pub fn to_chunk_and_local(self) -> (ChunkPos, LocalBlockPos) {
        let cx = self.0.x.div_euclid(CHUNK_WIDTH as i32);
        let cz = self.0.z.div_euclid(CHUNK_DEPTH as i32);
        let lx = self.0.x.rem_euclid(CHUNK_WIDTH as i32) as u8;
        let ly = self.0.y.clamp(0, (CHUNK_HEIGHT - 1) as i32) as u8;
        let lz = self.0.z.rem_euclid(CHUNK_DEPTH as i32) as u8;

        (
            ChunkPos(IVec2::new(cx, cz)),
            LocalBlockPos {
                x: lx,
                y: ly,
                z: lz,
            },
        )
    }

    #[inline]
    pub fn to_vec3(self) -> Vec3 {
        self.0.as_vec3()
    }

    #[inline]
    pub fn from_vec3(v: Vec3) -> Self {
        Self(IVec3::new(
            v.x.floor() as i32,
            v.y.floor() as i32,
            v.z.floor() as i32,
        ))
    }
    #[inline]
    pub const fn x(self) -> i32 {
        self.0.x
    }

    #[inline]
    pub const fn y(self) -> i32 {
        self.0.y
    }

    #[inline]
    pub const fn z(self) -> i32 {
        self.0.z
    }
}

impl ChunkPos {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }

    #[inline]
    pub const fn x(self) -> i32 {
        self.0.x
    }

    #[inline]
    pub const fn z(self) -> i32 {
        self.0.y
    }

    #[inline]
    pub fn world_min(self) -> BlockPos {
        BlockPos::new(
            self.0.x * CHUNK_WIDTH as i32,
            0,
            self.0.y * CHUNK_DEPTH as i32,
        )
    }

    #[inline]
    pub fn distance_sq(self, other: Self) -> i32 {
        let diff = self.0 - other.0;
        diff.x * diff.x + diff.y * diff.y
    }
}

impl LocalBlockPos {
    #[inline]
    pub const fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn to_index(self) -> usize {
        self.x as usize
            + (self.z as usize) * CHUNK_WIDTH
            + (self.y as usize) * (CHUNK_WIDTH * CHUNK_DEPTH)
    }

    #[inline]
    pub fn from_index(index: usize) -> Self {
        let x = (index % CHUNK_WIDTH) as u8;
        let rem = index / CHUNK_WIDTH;
        let z = (rem % CHUNK_DEPTH) as u8;
        let y = (rem / CHUNK_DEPTH) as u8;
        Self { x, y, z }
    }

    #[inline]
    pub fn to_world(self, chunk: ChunkPos) -> BlockPos {
        BlockPos::new(
            chunk.0.x * CHUNK_WIDTH as i32 + i32::from(self.x),
            i32::from(self.y),
            chunk.0.y * CHUNK_DEPTH as i32 + i32::from(self.z),
        )
    }
}

impl From<IVec3> for BlockPos {
    #[inline]
    fn from(v: IVec3) -> Self {
        Self(v)
    }
}

impl From<BlockPos> for IVec3 {
    #[inline]
    fn from(p: BlockPos) -> Self {
        p.0
    }
}

impl From<IVec2> for ChunkPos {
    #[inline]
    fn from(v: IVec2) -> Self {
        Self(v)
    }
}

impl From<ChunkPos> for IVec2 {
    #[inline]
    fn from(p: ChunkPos) -> Self {
        p.0
    }
}

impl Add<IVec3> for BlockPos {
    type Output = Self;
    #[inline]
    fn add(self, rhs: IVec3) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign<IVec3> for BlockPos {
    #[inline]
    fn add_assign(&mut self, rhs: IVec3) {
        self.0 += rhs;
    }
}

impl Sub<IVec3> for BlockPos {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: IVec3) -> Self {
        Self(self.0 - rhs)
    }
}

impl SubAssign<IVec3> for BlockPos {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec3) {
        self.0 -= rhs;
    }
}

impl Add<IVec2> for ChunkPos {
    type Output = Self;
    #[inline]
    fn add(self, rhs: IVec2) -> Self {
        Self(self.0 + rhs)
    }
}

impl Sub<IVec2> for ChunkPos {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: IVec2) -> Self {
        Self(self.0 - rhs)
    }
}

impl Add<BlockPos> for BlockPos {
    type Output = Self;
    #[inline]
    fn add(self, rhs: BlockPos) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<BlockPos> for BlockPos {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: BlockPos) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Add<ChunkPos> for ChunkPos {
    type Output = Self;
    #[inline]
    fn add(self, rhs: ChunkPos) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub<ChunkPos> for ChunkPos {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: ChunkPos) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl std::fmt::Display for BlockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block({}, {}, {})", self.0.x, self.0.y, self.0.z)
    }
}

impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk({}, {})", self.0.x, self.0.y)
    }
}

impl std::fmt::Display for LocalBlockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local({}, {}, {})", self.x, self.y, self.z)
    }
}

impl std::fmt::Display for WorldPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "World({:.2}, {:.2}, {:.2})",
            self.0.x, self.0.y, self.0.z
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_to_chunk_and_local_roundtrip() {
        let block_pos = BlockPos::new(35, 64, -18);
        let (chunk_pos, local_pos) = block_pos.to_chunk_and_local();

        assert_eq!(chunk_pos.0.x, 35 / 16);
        assert_eq!(chunk_pos.0.y, (-18_i32).div_euclid(16));
        assert_eq!(local_pos.x, 3);
        assert_eq!(local_pos.y, 64);
        assert_eq!(local_pos.z, (-18_i32).rem_euclid(16) as u8);

        let reconstructed = local_pos.to_world(chunk_pos);
        assert_eq!(reconstructed, block_pos);
    }

    #[test]
    fn test_local_index_bijection() {
        for index in [
            0,
            15,
            256,
            1024,
            CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH - 1,
        ] {
            let local = LocalBlockPos::from_index(index);
            assert_eq!(local.to_index(), index);
        }
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn proptest_block_coord_roundtrip(
            x in -1_000_000..1_000_000_i32,
            y in 0..(CHUNK_HEIGHT as i32),
            z in -1_000_000..1_000_000_i32,
        ) {
            let block_pos = BlockPos::new(x, y, z);
            let (chunk_pos, local_pos) = block_pos.to_chunk_and_local();
            prop_assert!((local_pos.x as usize) < CHUNK_WIDTH);
            prop_assert!((local_pos.y as usize) < CHUNK_HEIGHT);
            prop_assert!((local_pos.z as usize) < CHUNK_DEPTH);

            let reconstructed = local_pos.to_world(chunk_pos);
            prop_assert_eq!(reconstructed, block_pos);
        }

        #[test]
        fn proptest_local_pos_index_bijection(
            index in 0..(CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH),
        ) {
            let local = LocalBlockPos::from_index(index);
            prop_assert!((local.x as usize) < CHUNK_WIDTH);
            prop_assert!((local.y as usize) < CHUNK_HEIGHT);
            prop_assert!((local.z as usize) < CHUNK_DEPTH);
            prop_assert_eq!(local.to_index(), index);
        }
    }
}
