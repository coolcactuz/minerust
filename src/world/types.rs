use bevy::prelude::*;

pub const SEA_LEVEL: i32 = 64;
pub const VIEW_DISTANCE: i32 = 16;
pub const MAX_CHUNK_DISPATCH_PER_FRAME: usize = 12;
pub const MAX_MESHES_PER_FRAME: usize = 6;
pub const CHUNK_CACHE_CAPACITY: usize = 512;

/// Represents the world seed (numeric or derived from string/text)
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(133742)
    }
}

impl std::str::FromStr for WorldSeed {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if let Ok(num) = trimmed.parse::<u64>() {
            Ok(Self(num))
        } else {
            // Deterministic 64-bit FNV-1a hash algorithm for strings
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in trimmed.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Self(hash))
        }
    }
}

impl WorldSeed {
    #[must_use]
    pub fn random() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(133742, |d| d.as_nanos());
        let mut z = (nanos as u64).wrapping_add(0x9e3779b97f4a7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        Self((z ^ (z >> 31)) & 0x7fff_ffff_ffff_ffff)
    }

    #[must_use]
    pub fn from_seed_str(s: &str) -> Self {
        use std::str::FromStr;
        Self::from_str(s).unwrap_or_default()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BiomeType {
    Ocean,
    FrozenOcean,
    Beach,
    Plains,
    Forest,
    Desert,
    SnowyTundra,
    Mountains,
}
