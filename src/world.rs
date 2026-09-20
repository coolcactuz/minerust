use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::block::BlockType;
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};

#[derive(Resource, Default)]
pub struct WorldGrid {
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_entities: HashMap<IVec2, Entity>,
}

impl WorldGrid {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::default(),
            chunk_entities: HashMap::default(),
        }
    }

    #[inline]
    pub fn world_to_chunk_coord(wx: i32, wz: i32) -> (IVec2, usize, usize) {
        let cx = wx.div_euclid(CHUNK_WIDTH as i32);
        let cz = wz.div_euclid(CHUNK_DEPTH as i32);
        let lx = wx.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let lz = wz.rem_euclid(CHUNK_DEPTH as i32) as usize;
        (IVec2::new(cx, cz), lx, lz)
    }

    pub fn get_block(&self, pos: IVec3) -> BlockType {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }
        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        if let Some(chunk) = self.chunks.get(&c_coord) {
            chunk.get(lx as i32, pos.y, lz as i32)
        } else {
            BlockType::Air
        }
    }

    pub fn is_solid_at(&self, pos: IVec3) -> bool {
        self.get_block(pos).is_solid()
    }

    pub fn set_block(&mut self, pos: IVec3, block: BlockType) -> Vec<IVec2> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return Vec::new();
        }

        let (c_coord, lx, lz) = Self::world_to_chunk_coord(pos.x, pos.z);
        let mut dirty_chunks = Vec::new();

        if let Some(chunk) = self.chunks.get_mut(&c_coord) {
            chunk.set(lx as i32, pos.y, lz as i32, block);
            dirty_chunks.push(c_coord);

            // Se il blocco modificato è al bordo del chunk, aggiorna anche il chunk vicino
            if lx == 0 {
                dirty_chunks.push(c_coord + IVec2::new(-1, 0));
            } else if lx == CHUNK_WIDTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(1, 0));
            }

            if lz == 0 {
                dirty_chunks.push(c_coord + IVec2::new(0, -1));
            } else if lz == CHUNK_DEPTH - 1 {
                dirty_chunks.push(c_coord + IVec2::new(0, 1));
            }
        }

        dirty_chunks
    }

    pub fn generate_world(&mut self, radius: i32) {
        for cx in -radius..=radius {
            for cz in -radius..=radius {
                let coord = IVec2::new(cx, cz);
                let chunk = generate_chunk(cx, cz);
                self.chunks.insert(coord, chunk);
            }
        }
    }
}

fn sample_height(wx: f32, wz: f32) -> i32 {
    let n1 = (wx * 0.04).sin() * (wz * 0.04).cos();
    let n2 = (wx * 0.1 + 1.2).cos() * (wz * 0.08 - 0.7).sin() * 0.5;
    let n3 = (wx * 0.015 - 2.5).sin() * (wz * 0.02 + 1.8).cos() * 1.5;
    let combined = (n1 + n2 + n3) / 3.0;

    let base = 12.0;
    let amplitude = 7.0;
    let height = (base + combined * amplitude).round() as i32;
    height.clamp(3, (CHUNK_HEIGHT - 8) as i32)
}

fn pseudo_hash(x: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x45d9f3b) ^ (z as u32).wrapping_mul(0x27d4eb2d);
    h = ((h >> 16) ^ h).wrapping_mul(0x45d9f3b);
    (h >> 16) ^ h
}

fn generate_chunk(cx: i32, cz: i32) -> Chunk {
    let mut chunk = Chunk::new();
    let world_base_x = cx * CHUNK_WIDTH as i32;
    let world_base_z = cz * CHUNK_DEPTH as i32;

    // 1. Terreno base
    for lx in 0..CHUNK_WIDTH {
        for lz in 0..CHUNK_DEPTH {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = sample_height(wx as f32, wz as f32);

            for y in 0..=h {
                let block = if y == h {
                    BlockType::Grass
                } else if y >= h - 3 {
                    BlockType::Dirt
                } else {
                    BlockType::Stone
                };
                chunk.set(lx as i32, y, lz as i32, block);
            }
        }
    }

    // 2. Generazione alberi (lasciando un margine per le foglie)
    for lx in 2..(CHUNK_WIDTH - 2) {
        for lz in 2..(CHUNK_DEPTH - 2) {
            let wx = world_base_x + lx as i32;
            let wz = world_base_z + lz as i32;
            let h = sample_height(wx as f32, wz as f32);

            if pseudo_hash(wx, wz) % 47 == 0 && h + 6 < CHUNK_HEIGHT as i32 {
                // Tronco
                for ty in (h + 1)..=(h + 4) {
                    chunk.set(lx as i32, ty, lz as i32, BlockType::Wood);
                }

                // Chioma foglie
                for dx in -1_i32..=1_i32 {
                    for dz in -1_i32..=1_i32 {
                        for dy in (h + 3)..=(h + 5) {
                            if dy == h + 5 && dx.abs() == 1 && dz.abs() == 1 {
                                continue; // Angoli smussati in cima
                            }
                            let tx = lx as i32 + dx;
                            let tz = lz as i32 + dz;
                            if chunk.get(tx, dy, tz) == BlockType::Air {
                                chunk.set(tx, dy, tz, BlockType::Leaves);
                            }
                        }
                    }
                }
            }
        }
    }

    chunk
}
