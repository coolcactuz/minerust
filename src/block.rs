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
    CraftingTable = 21,
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

use crate::item::{ToolTier, ToolType};

/// Structural properties defining mining hardness, tool compatibility, and minimum harvest tier.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlockProperties {
    /// Mining hardness (higher is slower to mine, infinity is indestructible).
    pub hardness: f32,
    /// The preferred tool category that accelerates mining of this block.
    pub preferred_tool: Option<ToolType>,
    /// Minimum tool tier required to drop resources upon destruction.
    /// `None` indicates the block can be harvested with bare hands.
    pub min_tier: Option<ToolTier>,
}

impl BlockType {
    #[must_use]
    pub const fn properties(self) -> BlockProperties {
        match self {
            Self::Air => BlockProperties {
                hardness: 0.0,
                preferred_tool: None,
                min_tier: None,
            },
            Self::Grass | Self::Gravel => BlockProperties {
                hardness: 0.6,
                preferred_tool: Some(ToolType::Shovel),
                min_tier: None,
            },
            Self::Dirt | Self::Sand => BlockProperties {
                hardness: 0.5,
                preferred_tool: Some(ToolType::Shovel),
                min_tier: None,
            },
            Self::Stone => BlockProperties {
                hardness: 1.5,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Wood),
            },
            Self::Wood | Self::Planks | Self::CraftingTable => BlockProperties {
                hardness: 2.0,
                preferred_tool: Some(ToolType::Axe),
                min_tier: None,
            },
            Self::Leaves => BlockProperties {
                hardness: 0.2,
                preferred_tool: Some(ToolType::Sword),
                min_tier: None,
            },
            Self::Cobblestone => BlockProperties {
                hardness: 2.0,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Wood),
            },
            Self::Water | Self::Bedrock => BlockProperties {
                hardness: f32::INFINITY,
                preferred_tool: None,
                min_tier: None,
            },
            Self::Snow => BlockProperties {
                hardness: 0.2,
                preferred_tool: Some(ToolType::Shovel),
                min_tier: None,
            },
            Self::CoalOre => BlockProperties {
                hardness: 3.0,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Wood),
            },
            Self::IronOre => BlockProperties {
                hardness: 3.0,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Stone),
            },
            Self::GoldOre | Self::DiamondOre => BlockProperties {
                hardness: 3.0,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Iron),
            },
            Self::Sandstone => BlockProperties {
                hardness: 0.8,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: Some(ToolTier::Wood),
            },
            Self::Cactus => BlockProperties {
                hardness: 0.4,
                preferred_tool: Some(ToolType::Sword),
                min_tier: None,
            },
            Self::Ice => BlockProperties {
                hardness: 0.5,
                preferred_tool: Some(ToolType::Pickaxe),
                min_tier: None,
            },
            Self::Glass => BlockProperties {
                hardness: 0.3,
                preferred_tool: None,
                min_tier: None,
            },
        }
    }

    #[inline]
    #[must_use]
    pub const fn hardness(self) -> f32 {
        self.properties().hardness
    }

    /// Checks if a player equipped with a given tool category and tier can harvest (collect) this block's drops.
    #[must_use]
    pub fn can_harvest(self, equipped_tool: Option<(ToolType, ToolTier)>) -> bool {
        let props = self.properties();
        match props.min_tier {
            None => true,
            Some(required_tier) => match (props.preferred_tool, equipped_tool) {
                (Some(pref), Some((tool, tier))) if pref == tool => tier >= required_tier,
                _ => false,
            },
        }
    }

    #[inline]
    pub const fn is_solid(self) -> bool {
        !matches!(self, BlockType::Air | BlockType::Water)
    }

    #[inline]
    pub const fn is_water(self) -> bool {
        matches!(self, BlockType::Water)
    }

    #[inline]
    pub const fn is_transparent(self) -> bool {
        matches!(self, BlockType::Air | BlockType::Water | BlockType::Glass)
    }

    #[inline]
    pub const fn is_opaque(self) -> bool {
        !self.is_transparent()
    }

    #[inline]
    pub const fn drop_item(self) -> Option<BlockType> {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Bedrock => None,
            BlockType::Grass => Some(BlockType::Dirt),
            BlockType::Stone => Some(BlockType::Cobblestone),
            BlockType::Leaves => Some(BlockType::Leaves),
            other => Some(other),
        }
    }

    pub const fn from_u8(val: u8) -> Self {
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
            21 => BlockType::CraftingTable,
            _ => BlockType::Air,
        }
    }

    #[inline]
    pub const fn to_u8(self) -> u8 {
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
            BlockType::CraftingTable => ([0.65, 0.45, 0.28], face_shade(face)),
        };

        [
            base_rgb[0] * shade,
            base_rgb[1] * shade,
            base_rgb[2] * shade,
            1.0,
        ]
    }
}

const fn face_shade(face: BlockFace) -> f32 {
    match face {
        BlockFace::Top => 1.0,
        BlockFace::Bottom => 0.5,
        BlockFace::North | BlockFace::South => 0.85,
        BlockFace::East | BlockFace::West => 0.7,
    }
}

/// Computes the time (in seconds) required to break a block with an optional equipped tool.
#[must_use]
pub fn block_break_time(block: BlockType, equipped_tool: Option<(ToolType, ToolTier)>) -> f32 {
    let props = block.properties();
    if props.hardness <= 0.0 {
        return 0.0;
    }
    if props.hardness.is_infinite() {
        return f32::INFINITY;
    }

    let can_harvest = block.can_harvest(equipped_tool);
    let speed_multiplier = match equipped_tool {
        Some((tool, tier)) => {
            if props.preferred_tool == Some(tool) {
                tier.speed_multiplier()
            } else {
                1.0
            }
        }
        None => 1.0,
    };

    if can_harvest {
        (props.hardness * 1.5) / speed_multiplier
    } else {
        props.hardness * 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_properties_and_hardness() {
        assert_eq!(BlockType::Air.hardness(), 0.0);
        assert_eq!(BlockType::Dirt.hardness(), 0.5);
        assert_eq!(BlockType::Stone.hardness(), 1.5);
        assert_eq!(BlockType::Wood.hardness(), 2.0);
        assert!(BlockType::Bedrock.hardness().is_infinite());
        assert!(BlockType::Water.hardness().is_infinite());
    }

    #[test]
    fn test_can_harvest_mechanics() {
        // Dirt can be harvested with bare hands or anything
        assert!(BlockType::Dirt.can_harvest(None));
        assert!(BlockType::Dirt.can_harvest(Some((ToolType::Shovel, ToolTier::Wood))));

        // Stone requires at least a wooden pickaxe
        assert!(!BlockType::Stone.can_harvest(None));
        assert!(!BlockType::Stone.can_harvest(Some((ToolType::Axe, ToolTier::Diamond))));
        assert!(BlockType::Stone.can_harvest(Some((ToolType::Pickaxe, ToolTier::Wood))));
        assert!(BlockType::Stone.can_harvest(Some((ToolType::Pickaxe, ToolTier::Iron))));

        // DiamondOre requires at least an iron pickaxe
        assert!(!BlockType::DiamondOre.can_harvest(None));
        assert!(!BlockType::DiamondOre.can_harvest(Some((ToolType::Pickaxe, ToolTier::Stone))));
        assert!(BlockType::DiamondOre.can_harvest(Some((ToolType::Pickaxe, ToolTier::Iron))));
        assert!(BlockType::DiamondOre.can_harvest(Some((ToolType::Pickaxe, ToolTier::Diamond))));
    }

    #[test]
    fn test_block_break_time_calculations() {
        // Air is instantaneous
        assert_eq!(block_break_time(BlockType::Air, None), 0.0);

        // Bedrock is impossible
        assert!(block_break_time(BlockType::Bedrock, None).is_infinite());

        // Dirt: 0.5 * 1.5 = 0.75s by hand, 0.75 / 2.0 = 0.375s with wooden shovel
        let dirt_hand = block_break_time(BlockType::Dirt, None);
        assert!((dirt_hand - 0.75).abs() < 1e-4);

        let dirt_wood_shovel =
            block_break_time(BlockType::Dirt, Some((ToolType::Shovel, ToolTier::Wood)));
        assert!((dirt_wood_shovel - 0.375).abs() < 1e-4);

        // Stone: 1.5 * 5.0 = 7.5s by hand (penalty because cannot harvest),
        // (1.5 * 1.5) / 2.0 = 1.125s with wooden pickaxe,
        // (1.5 * 1.5) / 8.0 = 0.28125s with diamond pickaxe
        let stone_hand = block_break_time(BlockType::Stone, None);
        assert!((stone_hand - 7.5).abs() < 1e-4);

        let stone_wood_pick =
            block_break_time(BlockType::Stone, Some((ToolType::Pickaxe, ToolTier::Wood)));
        assert!((stone_wood_pick - 1.125).abs() < 1e-4);

        let stone_diamond_pick = block_break_time(
            BlockType::Stone,
            Some((ToolType::Pickaxe, ToolTier::Diamond)),
        );
        assert!((stone_diamond_pick - 0.28125).abs() < 1e-4);
    }
}
