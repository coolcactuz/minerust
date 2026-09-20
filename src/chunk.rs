use crate::block::BlockType;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_BLOCKS: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Box<[BlockType; CHUNK_BLOCKS]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![BlockType::Air; CHUNK_BLOCKS]
                .into_boxed_slice()
                .try_into()
                .unwrap_or_else(|_| panic!("Failed to allocate chunk")),
        }
    }

    #[inline]
    pub fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x >= 0
            && x < CHUNK_WIDTH as i32
            && y >= 0
            && y < CHUNK_HEIGHT as i32
            && z >= 0
            && z < CHUNK_DEPTH as i32
    }

    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
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
            self.blocks[idx] = block;
        }
    }

    #[inline(always)]
    pub fn set_fast(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        self.blocks[idx] = block;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHUNK_BLOCKS);
        for b in self.blocks.iter() {
            bytes.push(b.to_u8());
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != CHUNK_BLOCKS {
            return None;
        }
        let mut chunk = Self::new();
        for (i, b) in bytes.iter().enumerate() {
            chunk.blocks[i] = BlockType::from_u8(*b);
        }
        Some(chunk)
    }
}
