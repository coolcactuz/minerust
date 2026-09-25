use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH, Chunk};
use super::SECTION_HEIGHT;

const TOTAL_VOXELS: usize = CHUNK_WIDTH * SECTION_HEIGHT * CHUNK_DEPTH; // 4096

/// Directional faces of a 16x16x16 sub-chunk section.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SectionFace {
    North = 0, // +Z (z = 15)
    South = 1, // -Z (z = 0)
    West = 2,  // -X (x = 0)
    East = 3,  // +X (x = 15)
    Down = 4,  // -Y (y = 0)
    Up = 5,    // +Y (y = 15)
}

impl SectionFace {
    pub const ALL: [Self; 6] = [
        Self::North,
        Self::South,
        Self::West,
        Self::East,
        Self::Down,
        Self::Up,
    ];

    #[inline]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
        }
    }

    #[inline]
    #[must_use]
    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Self::North => (0, 0, 1),
            Self::South => (0, 0, -1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
        }
    }
}

/// Directional reachability graph for a 16x16x16 sub-chunk section.
///
/// Encodes which faces of the 16x16x16 cube can communicate with each other through non-opaque blocks (air).
/// Used by the Software Occlusion Culler to rapidly traverse visible sections from the camera's location
/// and cull completely sealed subterranean caves and hollows from GPU rendering.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SectionConnectivity {
    /// Bitmask for each face: bit `target` is 1 if `source` can reach `target` through non-opaque blocks.
    pub mask: [u8; 6],
    /// True if the section consists entirely of solid opaque blocks.
    pub is_solid: bool,
    /// True if the section consists entirely of non-opaque blocks (e.g. open air).
    pub is_empty: bool,
}

impl SectionConnectivity {
    /// Creates a fully open connectivity mask where all 6 faces can see all 6 faces (open sky / air).
    #[inline]
    #[must_use]
    pub const fn full() -> Self {
        Self {
            mask: [0b111111; 6],
            is_solid: false,
            is_empty: true,
        }
    }

    /// Checks if a ray or line of sight entering via `from` face can exit through `to` face through air.
    #[inline]
    #[must_use]
    pub const fn can_see(&self, from: SectionFace, to: SectionFace) -> bool {
        (self.mask[from as usize] & (1 << (to as u8))) != 0
    }

    /// Returns true if this section has any air opening to the sky (+Y / Up face).
    #[inline]
    #[must_use]
    pub const fn reaches_up(&self) -> bool {
        (self.mask[SectionFace::Up as usize] & (1 << (SectionFace::Up as u8))) != 0
    }
}

#[inline]
const fn local_idx(lx: usize, ry: usize, lz: usize) -> usize {
    lx + (lz << 4) + (ry << 8)
}

#[inline]
const fn face_mask_for_pos(lx: usize, ry: usize, lz: usize) -> u8 {
    let mut mask = 0u8;
    if ry == 0 {
        mask |= 1 << (SectionFace::Down as u8);
    }
    if ry == 15 {
        mask |= 1 << (SectionFace::Up as u8);
    }
    if lx == 0 {
        mask |= 1 << (SectionFace::West as u8);
    }
    if lx == 15 {
        mask |= 1 << (SectionFace::East as u8);
    }
    if lz == 0 {
        mask |= 1 << (SectionFace::South as u8);
    }
    if lz == 15 {
        mask |= 1 << (SectionFace::North as u8);
    }
    mask
}

/// Computes the directional connectivity graph for a 16x16x16 sub-chunk section.
///
/// Runs an L1-cache friendly flood-fill on the 4,096 voxels of the section, identifying which
/// of the 6 outer bounding faces are mutually reachable through air corridors.
#[must_use]
pub fn compute_section_connectivity(chunk: &Chunk, section_y: usize) -> SectionConnectivity {
    let y_start = section_y * SECTION_HEIGHT;

    // Quick preliminary scan for fully solid or fully empty sections
    let mut solid_count = 0usize;
    for ry in 0..SECTION_HEIGHT {
        let ly = y_start + ry;
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                if chunk.get_fast(lx, ly, lz).is_opaque() {
                    solid_count += 1;
                }
            }
        }
    }

    if solid_count == TOTAL_VOXELS {
        return SectionConnectivity {
            mask: [0; 6],
            is_solid: true,
            is_empty: false,
        };
    }

    if solid_count == 0 {
        return SectionConnectivity::full();
    }

    // Stack-allocated visited and queue buffers (12 KB total, fits comfortably in L1 cache)
    let mut visited = [false; TOTAL_VOXELS];
    let mut queue = [0u16; TOTAL_VOXELS];
    let mut connectivity = SectionConnectivity::default();

    // Flood fill only starting from non-opaque boundary voxels
    for ry in 0..SECTION_HEIGHT {
        let ly = y_start + ry;
        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let mask = face_mask_for_pos(lx, ry, lz);
                if mask == 0 {
                    continue; // Skip interior voxels as flood-fill entry points
                }

                let idx = local_idx(lx, ry, lz);
                if visited[idx] || chunk.get_fast(lx, ly, lz).is_opaque() {
                    continue;
                }

                // New air component touching the boundary
                visited[idx] = true;
                let mut head = 0usize;
                let mut tail = 0usize;
                queue[tail] = idx as u16;
                tail += 1;

                let mut component_faces = 0u8;

                while head < tail {
                    let cur = queue[head] as usize;
                    head += 1;

                    let clx = cur & 15;
                    let clz = (cur >> 4) & 15;
                    let cry = (cur >> 8) & 15;

                    component_faces |= face_mask_for_pos(clx, cry, clz);

                    // 6 orthogonal neighbor steps
                    let neighbors = [
                        (clx > 0).then(|| (clx - 1, cry, clz)),
                        (clx < 15).then(|| (clx + 1, cry, clz)),
                        (cry > 0).then(|| (clx, cry - 1, clz)),
                        (cry < 15).then(|| (clx, cry + 1, clz)),
                        (clz > 0).then(|| (clx, cry, clz - 1)),
                        (clz < 15).then(|| (clx, cry, clz + 1)),
                    ];

                    for (nlx, nry, nlz) in neighbors.into_iter().flatten() {
                        let n_idx = local_idx(nlx, nry, nlz);
                        if !visited[n_idx] && !chunk.get_fast(nlx, y_start + nry, nlz).is_opaque() {
                            visited[n_idx] = true;
                            queue[tail] = n_idx as u16;
                            tail += 1;
                        }
                    }
                }

                // Connect all faces touched by this air component to one another
                for face in SectionFace::ALL {
                    if (component_faces & (1 << (face as u8))) != 0 {
                        connectivity.mask[face as usize] |= component_faces;
                    }
                }
            }
        }
    }

    connectivity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_section_connectivity_empty_air() {
        let chunk = Chunk::new();
        let conn = compute_section_connectivity(&chunk, 0);
        assert!(conn.is_empty);
        assert!(!conn.is_solid);
        for f1 in SectionFace::ALL {
            for f2 in SectionFace::ALL {
                assert!(conn.can_see(f1, f2));
            }
        }
    }

    #[test]
    fn test_section_connectivity_solid_stone() {
        let mut chunk = Chunk::new();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Stone);
                }
            }
        }
        let conn = compute_section_connectivity(&chunk, 0);
        assert!(conn.is_solid);
        assert!(!conn.is_empty);
        for f1 in SectionFace::ALL {
            for f2 in SectionFace::ALL {
                assert!(!conn.can_see(f1, f2));
            }
        }
    }

    #[test]
    fn test_section_connectivity_tunnel_west_to_east() {
        let mut chunk = Chunk::new();
        // Fill entire section with solid stone
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Stone);
                }
            }
        }
        // Carve horizontal tunnel from West (x=0) to East (x=15) at y=8, z=8
        for x in 0..16 {
            chunk.set_fast(x, 8, 8, BlockType::Air);
        }

        let conn = compute_section_connectivity(&chunk, 0);
        assert!(!conn.is_solid);
        assert!(!conn.is_empty);

        // West and East should see each other
        assert!(conn.can_see(SectionFace::West, SectionFace::East));
        assert!(conn.can_see(SectionFace::East, SectionFace::West));

        // Up, Down, North, South should NOT see each other or the tunnel
        assert!(!conn.can_see(SectionFace::Up, SectionFace::Down));
        assert!(!conn.can_see(SectionFace::West, SectionFace::Up));
        assert!(!conn.can_see(SectionFace::West, SectionFace::North));
    }

    #[test]
    fn test_section_connectivity_sealed_interior_cave() {
        let mut chunk = Chunk::new();
        // Fill entire section with stone
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    chunk.set_fast(x, y, z, BlockType::Stone);
                }
            }
        }
        // Carve interior room in the middle: x in 5..10, y in 5..10, z in 5..10
        // (does not touch any of the 6 outer bounding faces)
        for y in 5..10 {
            for z in 5..10 {
                for x in 5..10 {
                    chunk.set_fast(x, y, z, BlockType::Air);
                }
            }
        }

        let conn = compute_section_connectivity(&chunk, 0);
        assert!(!conn.is_solid);
        assert!(!conn.is_empty);

        // Because the cave does not touch any boundary face, no face can see any other face
        for f1 in SectionFace::ALL {
            for f2 in SectionFace::ALL {
                assert!(!conn.can_see(f1, f2));
            }
        }
    }
}
