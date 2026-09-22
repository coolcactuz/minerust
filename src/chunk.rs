use std::sync::Arc;

use crate::block::BlockType;
use crate::coords::LocalBlockPos;
use crate::error::WorldError;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_BLOCKS: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub blocks: Arc<[BlockType; CHUNK_BLOCKS]>,
    pub max_y: usize,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        let boxed: Box<[BlockType; CHUNK_BLOCKS]> = match vec![BlockType::Air; CHUNK_BLOCKS]
            .into_boxed_slice()
            .try_into()
        {
            Ok(b) => b,
            Err(_) => unreachable!("vec length exactly matches CHUNK_BLOCKS"),
        };
        Self {
            blocks: Arc::from(boxed),
            max_y: 0,
        }
    }

    #[inline]
    pub const fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && x < CHUNK_WIDTH as i32
            && y >= 0
            && y < CHUNK_HEIGHT as i32
            && z >= 0
            && z < CHUNK_DEPTH as i32
    }

    #[inline]
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * (CHUNK_WIDTH * CHUNK_DEPTH)
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockType {
        if Self::in_bounds(x, y, z) {
            self.blocks[Self::index(x as usize, y as usize, z as usize)]
        } else {
            BlockType::Air
        }
    }

    #[inline(always)]
    pub fn get_fast(&self, x: usize, y: usize, z: usize) -> BlockType {
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if Self::in_bounds(x, y, z) {
            let idx = Self::index(x as usize, y as usize, z as usize);
            Arc::make_mut(&mut self.blocks)[idx] = block;
            if block != BlockType::Air && (y as usize) > self.max_y {
                self.max_y = y as usize;
            }
        }
    }

    #[inline(always)]
    pub fn set_fast(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        Arc::make_mut(&mut self.blocks)[idx] = block;
        if block != BlockType::Air && y > self.max_y {
            self.max_y = y;
        }
    }

    #[inline(always)]
    pub fn get_local(&self, pos: LocalBlockPos) -> BlockType {
        self.blocks[pos.to_index()]
    }

    #[inline(always)]
    pub fn set_local(&mut self, pos: LocalBlockPos, block: BlockType) {
        let idx = pos.to_index();
        Arc::make_mut(&mut self.blocks)[idx] = block;
        if block != BlockType::Air && usize::from(pos.y) > self.max_y {
            self.max_y = usize::from(pos.y);
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.blocks.iter().map(|b| b.to_u8()).collect()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WorldError> {
        if bytes.len() != CHUNK_BLOCKS {
            return Err(WorldError::Corruption(format!(
                "Invalid chunk byte length: expected {}, got {}",
                CHUNK_BLOCKS,
                bytes.len()
            )));
        }
        let mut chunk = Self::new();
        let mut max_y = 0;
        let mut_blocks = Arc::make_mut(&mut chunk.blocks);
        for (i, b) in bytes.iter().enumerate() {
            let block = BlockType::from_u8(*b);
            mut_blocks[i] = block;
            if block != BlockType::Air {
                let y = i / (CHUNK_WIDTH * CHUNK_DEPTH);
                if y > max_y {
                    max_y = y;
                }
            }
        }
        chunk.max_y = max_y;
        Ok(chunk)
    }

    /// Serializes the chunk to LZ4-compressed bytes, dramatically reducing memory and disk size.
    pub fn to_compressed_bytes(&self) -> Vec<u8> {
        let raw = self.to_bytes();
        lz4_flex::compress_prepend_size(&raw)
    }

    /// Deserializes a chunk from bytes, automatically handling both LZ4-compressed data and legacy uncompressed chunks.
    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<Self, WorldError> {
        if bytes.len() == CHUNK_BLOCKS {
            return Self::from_bytes(bytes);
        }
        let decompressed = lz4_flex::decompress_size_prepended(bytes)
            .map_err(|e| WorldError::Corruption(format!("LZ4 decompression failed: {e}")))?;
        Self::from_bytes(&decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_new_is_air() {
        let chunk = Chunk::new();
        assert_eq!(chunk.max_y, 0);
        assert_eq!(chunk.get_fast(0, 0, 0), BlockType::Air);
        assert_eq!(
            chunk.get_local(LocalBlockPos::new(15, 64, 15)),
            BlockType::Air
        );
    }

    #[test]
    fn test_chunk_get_set_local() {
        let mut chunk = Chunk::new();
        let pos = LocalBlockPos::new(3, 42, 7);
        chunk.set_local(pos, BlockType::Stone);
        assert_eq!(chunk.get_local(pos), BlockType::Stone);
        assert_eq!(chunk.max_y, 42);
    }

    #[test]
    fn test_chunk_serialization_roundtrip() {
        let mut chunk = Chunk::new();
        chunk.set_local(LocalBlockPos::new(0, 0, 0), BlockType::Bedrock);
        chunk.set_local(LocalBlockPos::new(5, 50, 5), BlockType::GoldOre);
        let bytes = chunk.to_bytes();
        assert_eq!(bytes.len(), CHUNK_BLOCKS);

        let loaded = Chunk::from_bytes(&bytes).expect("roundtrip should succeed");
        assert_eq!(
            loaded.get_local(LocalBlockPos::new(0, 0, 0)),
            BlockType::Bedrock
        );
        assert_eq!(
            loaded.get_local(LocalBlockPos::new(5, 50, 5)),
            BlockType::GoldOre
        );
        assert_eq!(loaded.max_y, 50);
    }

    #[test]
    fn test_chunk_compression_ratio_and_roundtrip() {
        let mut chunk = Chunk::new();
        // Emulate realistic terrain: bottom bedrock + stone layers + grass top
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_DEPTH {
                chunk.set_local(LocalBlockPos::new(x as u8, 0, z as u8), BlockType::Bedrock);
                for y in 1..=40 {
                    chunk.set_local(LocalBlockPos::new(x as u8, y, z as u8), BlockType::Stone);
                }
                chunk.set_local(LocalBlockPos::new(x as u8, 41, z as u8), BlockType::Grass);
            }
        }

        let compressed = chunk.to_compressed_bytes();
        // Uncompressed is 32,768 bytes. Highly redundant voxel data compresses to < 1,000 bytes!
        assert!(
            compressed.len() < 1000,
            "Expected high compression ratio on voxel data, got {} bytes",
            compressed.len()
        );

        let loaded =
            Chunk::from_compressed_bytes(&compressed).expect("decompression should succeed");
        assert_eq!(loaded.max_y, 41);
        assert_eq!(
            loaded.get_local(LocalBlockPos::new(7, 41, 7)),
            BlockType::Grass
        );
        assert_eq!(
            loaded.get_local(LocalBlockPos::new(7, 20, 7)),
            BlockType::Stone
        );
    }

    #[test]
    fn test_chunk_deserialization_corruption_error() {
        let corrupted_bytes = vec![0_u8; 100];
        let result = Chunk::from_bytes(&corrupted_bytes);
        assert!(matches!(result, Err(WorldError::Corruption(_))));
    }
}
