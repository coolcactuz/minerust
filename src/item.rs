#![forbid(unsafe_code)]

use crate::block::{BlockFace, BlockType};

/// Maximum item stack count for stackable items (standard blocks and materials).
pub const MAX_STACK_SIZE: u32 = 64;

/// Category of tool defining its function and harvest capabilities.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ToolType {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
}

impl ToolType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pickaxe => "Pickaxe",
            Self::Axe => "Axe",
            Self::Shovel => "Shovel",
            Self::Sword => "Sword",
            Self::Hoe => "Hoe",
        }
    }
}

/// Material quality tier of a tool, affecting harvest speed and durability.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ToolTier {
    Wood = 0,
    Stone = 1,
    Iron = 2,
    Diamond = 3,
    Gold = 4,
}

impl ToolTier {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wood => "Wooden",
            Self::Stone => "Stone",
            Self::Iron => "Iron",
            Self::Diamond => "Diamond",
            Self::Gold => "Golden",
        }
    }

    /// Mining speed multiplier applied when using this tool tier on compatible blocks.
    #[must_use]
    pub const fn speed_multiplier(self) -> f32 {
        match self {
            Self::Wood => 2.0,
            Self::Stone => 4.0,
            Self::Iron => 6.0,
            Self::Diamond => 8.0,
            Self::Gold => 12.0,
        }
    }

    /// Maximum durability (number of uses) for tools of this tier.
    #[must_use]
    pub const fn max_durability(self) -> u32 {
        match self {
            Self::Wood => 59,
            Self::Stone => 131,
            Self::Iron => 250,
            Self::Diamond => 1561,
            Self::Gold => 32,
        }
    }

    /// Minimum harvest tier rating (higher ratings can harvest harder blocks).
    #[must_use]
    pub const fn harvest_level(self) -> u8 {
        match self {
            Self::Wood | Self::Gold => 1,
            Self::Stone => 2,
            Self::Iron => 3,
            Self::Diamond => 4,
        }
    }

    /// Representative display color for this tier's UI icon.
    #[must_use]
    pub const fn color(self) -> [f32; 3] {
        match self {
            Self::Wood => [0.55, 0.38, 0.22],
            Self::Stone => [0.60, 0.60, 0.62],
            Self::Iron => [0.85, 0.85, 0.88],
            Self::Diamond => [0.30, 0.85, 0.90],
            Self::Gold => [1.00, 0.84, 0.20],
        }
    }
}

/// Basic non-block crafting ingredients and mined items.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MaterialType {
    Stick,
    Coal,
    IronIngot,
    GoldIngot,
    Diamond,
}

impl MaterialType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Stick => "Stick",
            Self::Coal => "Coal",
            Self::IronIngot => "Iron Ingot",
            Self::GoldIngot => "Gold Ingot",
            Self::Diamond => "Diamond",
        }
    }

    /// Representative display color for this material in UI icons.
    #[must_use]
    pub const fn color(self) -> [f32; 3] {
        match self {
            Self::Stick => [0.45, 0.30, 0.15],
            Self::Coal => [0.15, 0.15, 0.18],
            Self::IronIngot => [0.82, 0.82, 0.85],
            Self::GoldIngot => [0.98, 0.80, 0.20],
            Self::Diamond => [0.35, 0.88, 0.95],
        }
    }
}

/// Unified item representation distinguishing between placeable blocks, crafting materials, and tools.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    /// A placeable 3D voxel block.
    Block(BlockType),
    /// A loose crafting material or drop item.
    Material(MaterialType),
    /// An equippable tool with tier and usage characteristics.
    Tool { tool: ToolType, tier: ToolTier },
}

impl Default for ItemType {
    fn default() -> Self {
        Self::Block(BlockType::Air)
    }
}

impl From<BlockType> for ItemType {
    #[inline]
    fn from(block: BlockType) -> Self {
        Self::Block(block)
    }
}

impl From<MaterialType> for ItemType {
    #[inline]
    fn from(material: MaterialType) -> Self {
        Self::Material(material)
    }
}

impl ItemType {
    /// Creates a tool item with the specified tool category and material tier.
    #[inline]
    #[must_use]
    pub const fn tool(tool: ToolType, tier: ToolTier) -> Self {
        Self::Tool { tool, tier }
    }

    /// Returns human-readable name of the item.
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Block(b) => match b {
                BlockType::Air => "Air".to_string(),
                BlockType::Grass => "Grass".to_string(),
                BlockType::Dirt => "Dirt".to_string(),
                BlockType::Stone => "Stone".to_string(),
                BlockType::Cobblestone => "Cobblestone".to_string(),
                BlockType::Wood => "Wood".to_string(),
                BlockType::Planks => "Planks".to_string(),
                BlockType::Leaves => "Leaves".to_string(),
                BlockType::Glass => "Glass".to_string(),
                BlockType::Sand => "Sand".to_string(),
                BlockType::Sandstone => "Sandstone".to_string(),
                BlockType::Gravel => "Gravel".to_string(),
                BlockType::Water => "Water".to_string(),
                BlockType::Ice => "Ice".to_string(),
                BlockType::Snow => "Snow".to_string(),
                BlockType::Cactus => "Cactus".to_string(),
                BlockType::CoalOre => "Coal Ore".to_string(),
                BlockType::IronOre => "Iron Ore".to_string(),
                BlockType::GoldOre => "Gold Ore".to_string(),
                BlockType::DiamondOre => "Diamond Ore".to_string(),
                BlockType::Bedrock => "Bedrock".to_string(),
            },
            Self::Material(m) => m.name().to_string(),
            Self::Tool { tool, tier } => format!("{} {}", tier.name(), tool.name()),
        }
    }

    /// Returns the underlying block type if this item is a placeable block.
    #[inline]
    #[must_use]
    pub const fn as_block(self) -> Option<BlockType> {
        match self {
            Self::Block(b) if !matches!(b, BlockType::Air) => Some(b),
            _ => None,
        }
    }

    /// Returns true if this item is a block (including Air).
    #[inline]
    #[must_use]
    pub const fn is_block(self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// Returns true if this item is an equippable tool.
    #[inline]
    #[must_use]
    pub const fn is_tool(self) -> bool {
        matches!(self, Self::Tool { .. })
    }

    /// Returns true if this item represents empty air.
    #[inline]
    #[must_use]
    pub const fn is_air(self) -> bool {
        matches!(self, Self::Block(BlockType::Air))
    }

    /// Returns true if this item can be placed in the 3D voxel world.
    #[inline]
    #[must_use]
    pub const fn is_placeable(self) -> bool {
        match self {
            Self::Block(b) => b.is_solid() || b.is_water(),
            _ => false,
        }
    }

    /// Returns maximum stack capacity for this item.
    /// Tools cannot stack (limit 1), whereas blocks and materials stack up to 64.
    #[inline]
    #[must_use]
    pub const fn max_stack_size(self) -> u32 {
        match self {
            Self::Tool { .. } => 1,
            Self::Block(_) | Self::Material(_) => MAX_STACK_SIZE,
        }
    }

    /// Color preview used in HUD and Inventory slot rendering.
    #[must_use]
    pub fn color(self) -> [f32; 4] {
        match self {
            Self::Block(b) => b.color(BlockFace::Top),
            Self::Material(m) => {
                let [r, g, b] = m.color();
                [r, g, b, 1.0]
            }
            Self::Tool { tier, .. } => {
                let [r, g, b] = tier.color();
                [r, g, b, 1.0]
            }
        }
    }

    /// Computes mining speed multiplier against a target block.
    #[must_use]
    pub fn tool_speed_for(self, block: BlockType) -> f32 {
        match self {
            Self::Tool { tool, tier } => {
                let matches_tool = match tool {
                    ToolType::Pickaxe => matches!(
                        block,
                        BlockType::Stone
                            | BlockType::Cobblestone
                            | BlockType::CoalOre
                            | BlockType::IronOre
                            | BlockType::GoldOre
                            | BlockType::DiamondOre
                            | BlockType::Sandstone
                            | BlockType::Ice
                    ),
                    ToolType::Axe => matches!(block, BlockType::Wood | BlockType::Planks),
                    ToolType::Shovel => {
                        matches!(
                            block,
                            BlockType::Dirt
                                | BlockType::Grass
                                | BlockType::Sand
                                | BlockType::Gravel
                                | BlockType::Snow
                        )
                    }
                    ToolType::Sword => matches!(block, BlockType::Leaves | BlockType::Cactus),
                    ToolType::Hoe => false,
                };

                if matches_tool {
                    tier.speed_multiplier()
                } else {
                    1.0
                }
            }
            _ => 1.0,
        }
    }
}

/// Represents a stack of items with a quantity count, capped to item's maximum stack size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ItemStack {
    pub item: ItemType,
    pub count: u32,
}

impl ItemStack {
    /// Creates a new item stack clamped to the item's maximum stack limit.
    #[inline]
    pub fn new(item: impl Into<ItemType>, count: u32) -> Self {
        let item = item.into();
        Self {
            item,
            count: count.min(item.max_stack_size()),
        }
    }

    /// Convenience constructor for block item stacks.
    #[inline]
    pub fn block(block: BlockType, count: u32) -> Self {
        Self::new(ItemType::Block(block), count)
    }

    /// Convenience constructor for crafting material stacks.
    #[inline]
    pub fn material(mat: MaterialType, count: u32) -> Self {
        Self::new(ItemType::Material(mat), count)
    }

    /// Convenience constructor for tools (always stack size 1).
    #[inline]
    pub fn tool(tool: ToolType, tier: ToolTier) -> Self {
        Self::new(ItemType::tool(tool, tier), 1)
    }

    /// Returns the underlying block type if this item stack is a block, or `BlockType::Air`.
    #[inline]
    #[must_use]
    pub const fn block_type(&self) -> BlockType {
        match self.item {
            ItemType::Block(b) => b,
            _ => BlockType::Air,
        }
    }

    /// Checks if the stack has zero count.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// Determines the item dropped when a block is successfully mined.
#[must_use]
pub fn block_drop(block: BlockType) -> Option<ItemType> {
    match block {
        BlockType::Air | BlockType::Water | BlockType::Bedrock => None,
        BlockType::Grass => Some(ItemType::Block(BlockType::Dirt)),
        BlockType::Stone => Some(ItemType::Block(BlockType::Cobblestone)),
        BlockType::CoalOre => Some(ItemType::Material(MaterialType::Coal)),
        BlockType::DiamondOre => Some(ItemType::Material(MaterialType::Diamond)),
        BlockType::Leaves => Some(ItemType::Block(BlockType::Leaves)),
        other => Some(ItemType::Block(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_properties_and_names() {
        let pickaxe = ItemType::tool(ToolType::Pickaxe, ToolTier::Diamond);
        assert_eq!(pickaxe.name(), "Diamond Pickaxe");
        assert_eq!(pickaxe.max_stack_size(), 1);
        assert!(pickaxe.is_tool());
        assert!(!pickaxe.is_block());
        assert_eq!(pickaxe.as_block(), None);
        assert_eq!(pickaxe.tool_speed_for(BlockType::Stone), 8.0);
        assert_eq!(pickaxe.tool_speed_for(BlockType::Wood), 1.0);
    }

    #[test]
    fn test_material_properties() {
        let coal = ItemType::Material(MaterialType::Coal);
        assert_eq!(coal.name(), "Coal");
        assert_eq!(coal.max_stack_size(), 64);
        assert_eq!(coal.as_block(), None);
        assert!(!coal.is_placeable());
    }

    #[test]
    fn test_item_stack_clamping_for_tools() {
        let stack = ItemStack::new(ItemType::tool(ToolType::Axe, ToolTier::Iron), 10);
        // Tools must be capped to stack size 1
        assert_eq!(stack.count, 1);
        assert_eq!(stack.item, ItemType::tool(ToolType::Axe, ToolTier::Iron));
    }

    #[test]
    fn test_item_stack_from_block_type_conversion() {
        let stack = ItemStack::new(BlockType::Wood, 32);
        assert_eq!(stack.item, ItemType::Block(BlockType::Wood));
        assert_eq!(stack.count, 32);
        assert_eq!(stack.block_type(), BlockType::Wood);
    }

    #[test]
    fn test_block_drop_rules() {
        assert_eq!(block_drop(BlockType::Air), None);
        assert_eq!(block_drop(BlockType::Water), None);
        assert_eq!(block_drop(BlockType::Bedrock), None);
        assert_eq!(
            block_drop(BlockType::Grass),
            Some(ItemType::Block(BlockType::Dirt))
        );
        assert_eq!(
            block_drop(BlockType::Stone),
            Some(ItemType::Block(BlockType::Cobblestone))
        );
        assert_eq!(
            block_drop(BlockType::CoalOre),
            Some(ItemType::Material(MaterialType::Coal))
        );
        assert_eq!(
            block_drop(BlockType::DiamondOre),
            Some(ItemType::Material(MaterialType::Diamond))
        );
        assert_eq!(
            block_drop(BlockType::IronOre),
            Some(ItemType::Block(BlockType::IronOre))
        );
    }

    #[test]
    fn test_item_stack_serialization_roundtrip() {
        let original = ItemStack::tool(ToolType::Pickaxe, ToolTier::Stone);
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ItemStack = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
}
