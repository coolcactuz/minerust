#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Wood = 4,
    Leaves = 5,
    Cobblestone = 6,
    Planks = 7,
    Sand = 8,
    Water = 9,
    Snow = 10,
    Bedrock = 11,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockFace {
    Top,    // +Y
    Bottom, // -Y
    North,  // +Z
    South,  // -Z
    East,   // +X
    West,   // -X
}

impl BlockType {
    #[inline]
    pub fn is_solid(&self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    #[inline]
    pub fn is_water(&self) -> bool {
        matches!(self, BlockType::Water)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn is_transparent(&self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Leaves)
    }

    pub fn color(&self, face: BlockFace) -> [f32; 4] {
        let (base_rgb, shade) = match self {
            BlockType::Air => ([0.0, 0.0, 0.0], 1.0),
            BlockType::Grass => match face {
                BlockFace::Top => ([0.28, 0.72, 0.22], 1.0),
                BlockFace::Bottom => ([0.45, 0.28, 0.16], 0.5),
                _ => ([0.38, 0.52, 0.20], face_shade(face)),
            },
            BlockType::Dirt => ([0.48, 0.30, 0.18], face_shade(face)),
            BlockType::Stone => ([0.52, 0.52, 0.52], face_shade(face)),
            BlockType::Wood => match face {
                BlockFace::Top | BlockFace::Bottom => ([0.65, 0.50, 0.32], face_shade(face)),
                _ => ([0.42, 0.28, 0.16], face_shade(face)),
            },
            BlockType::Leaves => ([0.18, 0.58, 0.18], face_shade(face)),
            BlockType::Cobblestone => ([0.42, 0.42, 0.44], face_shade(face)),
            BlockType::Planks => ([0.72, 0.55, 0.35], face_shade(face)),
            BlockType::Sand => ([0.86, 0.82, 0.58], face_shade(face)),
            BlockType::Water => ([0.18, 0.48, 0.88], face_shade(face)),
            BlockType::Snow => match face {
                BlockFace::Top => ([0.95, 0.96, 0.98], 1.0),
                _ => ([0.88, 0.90, 0.92], face_shade(face)),
            },
            BlockType::Bedrock => ([0.15, 0.15, 0.16], face_shade(face)),
        };

        [
            base_rgb[0] * shade,
            base_rgb[1] * shade,
            base_rgb[2] * shade,
            1.0,
        ]
    }
}

fn face_shade(face: BlockFace) -> f32 {
    match face {
        BlockFace::Top => 1.0,
        BlockFace::Bottom => 0.5,
        BlockFace::North | BlockFace::South => 0.85,
        BlockFace::East | BlockFace::West => 0.7,
    }
}
