#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
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
    CoalOre = 12,
    IronOre = 13,
    GoldOre = 14,
    DiamondOre = 15,
    Sandstone = 16,
    Cactus = 17,
    Gravel = 18,
    Ice = 19,
    Glass = 20,
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
    pub fn is_solid(self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    #[inline]
    pub fn is_water(self) -> bool {
        self == BlockType::Water
    }

    #[inline]
    pub fn is_transparent(self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Glass)
    }

    #[inline]
    pub fn drop_item(self) -> Option<BlockType> {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Bedrock => None,
            BlockType::Grass => Some(BlockType::Dirt),
            BlockType::Stone => Some(BlockType::Cobblestone),
            BlockType::Leaves => Some(BlockType::Leaves),
            other => Some(other),
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => BlockType::Grass,
            2 => BlockType::Dirt,
            3 => BlockType::Stone,
            4 => BlockType::Wood,
            5 => BlockType::Leaves,
            6 => BlockType::Cobblestone,
            7 => BlockType::Planks,
            8 => BlockType::Sand,
            9 => BlockType::Water,
            10 => BlockType::Snow,
            11 => BlockType::Bedrock,
            12 => BlockType::CoalOre,
            13 => BlockType::IronOre,
            14 => BlockType::GoldOre,
            15 => BlockType::DiamondOre,
            16 => BlockType::Sandstone,
            17 => BlockType::Cactus,
            18 => BlockType::Gravel,
            19 => BlockType::Ice,
            20 => BlockType::Glass,
            _ => BlockType::Air,
        }
    }

    #[inline]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn color(self, face: BlockFace) -> [f32; 4] {
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
            BlockType::CoalOre => ([0.30, 0.30, 0.30], face_shade(face)),
            BlockType::IronOre => ([0.68, 0.56, 0.46], face_shade(face)),
            BlockType::GoldOre => ([0.82, 0.72, 0.25], face_shade(face)),
            BlockType::DiamondOre => ([0.32, 0.82, 0.85], face_shade(face)),
            BlockType::Sandstone => ([0.80, 0.74, 0.52], face_shade(face)),
            BlockType::Cactus => ([0.22, 0.60, 0.24], face_shade(face)),
            BlockType::Gravel => ([0.46, 0.45, 0.45], face_shade(face)),
            BlockType::Ice => ([0.62, 0.82, 0.96], face_shade(face)),
            BlockType::Glass => ([0.85, 0.92, 0.96], face_shade(face)),
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
