#![forbid(unsafe_code)]

use crate::block::BlockType;
use crate::inventory::{ItemStack, ItemType, MaterialType, ToolTier, ToolType};

/// Bounding box enclosing all non-empty items in a crafting grid.
#[derive(Debug, PartialEq, Eq)]
pub struct GridBounds {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
    pub width: usize,
    pub height: usize,
}

/// Finds the tightest bounding box of non-empty items within a 2D crafting grid.
#[must_use]
pub fn get_grid_bounds(
    grid: &[Option<ItemStack>],
    grid_w: usize,
    grid_h: usize,
) -> Option<GridBounds> {
    let mut min_x = usize::MAX;
    let mut max_x = 0;
    let mut min_y = usize::MAX;
    let mut max_y = 0;
    let mut has_items = false;

    for y in 0..grid_h {
        for x in 0..grid_w {
            let idx = y * grid_w + x;
            if let Some(stack) = grid.get(idx).copied().flatten() {
                if !stack.is_empty() && !stack.item.is_air() {
                    has_items = true;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
    }

    if !has_items {
        None
    } else {
        Some(GridBounds {
            min_x,
            max_x,
            min_y,
            max_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        })
    }
}

/// A shaped crafting recipe where relative arrangement of ingredients matters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapedRecipe {
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Option<ItemType>>,
    pub output: ItemStack,
    pub mirrored: bool,
}

impl ShapedRecipe {
    #[must_use]
    pub fn matches(&self, grid: &[Option<ItemStack>], grid_w: usize, grid_h: usize) -> bool {
        let Some(bounds) = get_grid_bounds(grid, grid_w, grid_h) else {
            return false;
        };

        if bounds.width != self.width || bounds.height != self.height {
            return false;
        }

        // Test normal orientation
        let mut matches_normal = true;
        for y in 0..self.height {
            for x in 0..self.width {
                let grid_x = bounds.min_x + x;
                let grid_y = bounds.min_y + y;
                let grid_item = grid
                    .get(grid_y * grid_w + grid_x)
                    .copied()
                    .flatten()
                    .map(|s| s.item);
                let recipe_item = self.pattern.get(y * self.width + x).copied().flatten();

                if grid_item != recipe_item {
                    matches_normal = false;
                    break;
                }
            }
            if !matches_normal {
                break;
            }
        }

        if matches_normal {
            return true;
        }

        // Test horizontally mirrored orientation
        if self.mirrored && self.width > 1 {
            let mut matches_mirrored = true;
            for y in 0..self.height {
                for x in 0..self.width {
                    let grid_x = bounds.min_x + x;
                    let grid_y = bounds.min_y + y;
                    let grid_item = grid
                        .get(grid_y * grid_w + grid_x)
                        .copied()
                        .flatten()
                        .map(|s| s.item);
                    let mirror_x = self.width - 1 - x;
                    let recipe_item = self
                        .pattern
                        .get(y * self.width + mirror_x)
                        .copied()
                        .flatten();

                    if grid_item != recipe_item {
                        matches_mirrored = false;
                        break;
                    }
                }
                if !matches_mirrored {
                    break;
                }
            }
            if matches_mirrored {
                return true;
            }
        }

        false
    }
}

/// A shapeless crafting recipe where ingredients can be placed anywhere in any order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapelessRecipe {
    pub ingredients: Vec<ItemType>,
    pub output: ItemStack,
}

impl ShapelessRecipe {
    #[must_use]
    pub fn matches(&self, grid: &[Option<ItemStack>]) -> bool {
        let mut grid_items = Vec::new();
        for slot in grid.iter().flatten() {
            if !slot.is_empty() && !slot.item.is_air() {
                grid_items.push(slot.item);
            }
        }

        if grid_items.len() != self.ingredients.len() {
            return false;
        }

        grid_items.sort_by_key(|i| format!("{i:?}"));
        let mut req_items = self.ingredients.clone();
        req_items.sort_by_key(|i| format!("{i:?}"));

        grid_items == req_items
    }
}

/// Unified crafting recipe type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
}

impl Recipe {
    #[must_use]
    pub fn matches(&self, grid: &[Option<ItemStack>], grid_w: usize, grid_h: usize) -> bool {
        match self {
            Self::Shaped(r) => r.matches(grid, grid_w, grid_h),
            Self::Shapeless(r) => r.matches(grid),
        }
    }

    #[must_use]
    pub fn output(&self) -> ItemStack {
        match self {
            Self::Shaped(r) => r.output,
            Self::Shapeless(r) => r.output,
        }
    }
}

/// Helper to generate 3x3 tool recipes for standard Minecraft patterns.
fn tool_recipe_3x3(mat: ItemType, tool: ToolType, tier: ToolTier) -> Vec<Recipe> {
    let stick = ItemType::Material(MaterialType::Stick);
    match tool {
        ToolType::Pickaxe => vec![Recipe::Shaped(ShapedRecipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(mat),
                Some(mat),
                Some(mat),
                None,
                Some(stick),
                None,
                None,
                Some(stick),
                None,
            ],
            output: ItemStack::tool(ToolType::Pickaxe, tier),
            mirrored: false,
        })],
        ToolType::Axe => vec![Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 3,
            pattern: vec![
                Some(mat),
                Some(mat),
                Some(mat),
                Some(stick),
                None,
                Some(stick),
            ],
            output: ItemStack::tool(ToolType::Axe, tier),
            mirrored: true,
        })],
        ToolType::Shovel => vec![Recipe::Shaped(ShapedRecipe {
            width: 1,
            height: 3,
            pattern: vec![Some(mat), Some(stick), Some(stick)],
            output: ItemStack::tool(ToolType::Shovel, tier),
            mirrored: false,
        })],
        ToolType::Sword => vec![Recipe::Shaped(ShapedRecipe {
            width: 1,
            height: 3,
            pattern: vec![Some(mat), Some(mat), Some(stick)],
            output: ItemStack::tool(ToolType::Sword, tier),
            mirrored: false,
        })],
        ToolType::Hoe => vec![Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 3,
            pattern: vec![Some(mat), Some(mat), None, Some(stick), None, Some(stick)],
            output: ItemStack::tool(ToolType::Hoe, tier),
            mirrored: true,
        })],
    }
}

/// Static registry of available crafting recipes.
#[must_use]
pub fn all_recipes() -> Vec<Recipe> {
    let mut recipes = vec![
        // 1. Basic conversions
        // Wood -> 4 Planks (Shapeless)
        Recipe::Shapeless(ShapelessRecipe {
            ingredients: vec![ItemType::Block(BlockType::Wood)],
            output: ItemStack::new(BlockType::Planks, 4),
        }),
        // 4 Planks (2x2) -> 1 Crafting Table
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
            ],
            output: ItemStack::new(BlockType::CraftingTable, 1),
            mirrored: false,
        }),
        // 2 Planks vertical (1x2) -> 4 Sticks
        Recipe::Shaped(ShapedRecipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
            ],
            output: ItemStack::material(MaterialType::Stick, 4),
            mirrored: false,
        }),
        // 4 Sand (2x2) -> 1 Sandstone
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Sand)),
                Some(ItemType::Block(BlockType::Sand)),
                Some(ItemType::Block(BlockType::Sand)),
                Some(ItemType::Block(BlockType::Sand)),
            ],
            output: ItemStack::new(BlockType::Sandstone, 1),
            mirrored: false,
        }),
        // 4 Cobblestone (2x2) -> 4 Stone
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
            ],
            output: ItemStack::new(BlockType::Stone, 4),
            mirrored: false,
        }),
        // 2x2 Compact tools (handy shortcuts)
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Material(MaterialType::Stick)),
                None,
            ],
            output: ItemStack::tool(ToolType::Pickaxe, ToolTier::Wood),
            mirrored: true,
        }),
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Material(MaterialType::Stick)),
                None,
            ],
            output: ItemStack::tool(ToolType::Pickaxe, ToolTier::Stone),
            mirrored: true,
        }),
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Material(MaterialType::Stick)),
            ],
            output: ItemStack::tool(ToolType::Axe, ToolTier::Wood),
            mirrored: true,
        }),
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Material(MaterialType::Stick)),
            ],
            output: ItemStack::tool(ToolType::Axe, ToolTier::Stone),
            mirrored: true,
        }),
        Recipe::Shaped(ShapedRecipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Planks)),
                Some(ItemType::Material(MaterialType::Stick)),
            ],
            output: ItemStack::tool(ToolType::Shovel, ToolTier::Wood),
            mirrored: false,
        }),
        Recipe::Shaped(ShapedRecipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(ItemType::Block(BlockType::Cobblestone)),
                Some(ItemType::Material(MaterialType::Stick)),
            ],
            output: ItemStack::tool(ToolType::Shovel, ToolTier::Stone),
            mirrored: false,
        }),
    ];

    // Standard 3x3 Tool Recipes for all 5 material tiers
    let tiers_and_mats = [
        (ItemType::Block(BlockType::Planks), ToolTier::Wood),
        (ItemType::Block(BlockType::Cobblestone), ToolTier::Stone),
        (ItemType::Material(MaterialType::IronIngot), ToolTier::Iron),
        (ItemType::Material(MaterialType::Diamond), ToolTier::Diamond),
        (ItemType::Material(MaterialType::GoldIngot), ToolTier::Gold),
    ];

    for (mat, tier) in tiers_and_mats {
        for tool in [
            ToolType::Pickaxe,
            ToolType::Axe,
            ToolType::Shovel,
            ToolType::Sword,
            ToolType::Hoe,
        ] {
            recipes.extend(tool_recipe_3x3(mat, tool, tier));
        }
    }

    recipes
}

/// Evaluates a grid against all recipes and returns the output stack if a match is found.
#[must_use]
pub fn match_recipe(grid: &[Option<ItemStack>], grid_w: usize, grid_h: usize) -> Option<ItemStack> {
    let recipes = all_recipes();
    for recipe in recipes {
        if recipe.matches(grid, grid_w, grid_h) {
            return Some(recipe.output());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shapeless_wood_to_planks() {
        let grid = [Some(ItemStack::block(BlockType::Wood, 1)), None, None, None];
        let out = match_recipe(&grid, 2, 2);
        assert_eq!(out, Some(ItemStack::new(BlockType::Planks, 4)));
    }

    #[test]
    fn test_crafting_table_recipe() {
        let grid = [
            Some(ItemStack::block(BlockType::Planks, 1)),
            Some(ItemStack::block(BlockType::Planks, 1)),
            Some(ItemStack::block(BlockType::Planks, 1)),
            Some(ItemStack::block(BlockType::Planks, 1)),
        ];
        let out = match_recipe(&grid, 2, 2);
        assert_eq!(out, Some(ItemStack::new(BlockType::CraftingTable, 1)));
    }

    #[test]
    fn test_standard_3x3_pickaxe_recipe() {
        let stick = ItemType::Material(MaterialType::Stick);
        let iron = ItemType::Material(MaterialType::IronIngot);

        let grid = [
            Some(ItemStack::new(iron, 1)),
            Some(ItemStack::new(iron, 1)),
            Some(ItemStack::new(iron, 1)),
            None,
            Some(ItemStack::new(stick, 1)),
            None,
            None,
            Some(ItemStack::new(stick, 1)),
            None,
        ];

        let out = match_recipe(&grid, 3, 3);
        assert_eq!(
            out,
            Some(ItemStack::tool(ToolType::Pickaxe, ToolTier::Iron))
        );
    }

    #[test]
    fn test_standard_3x3_sword_recipe() {
        let stick = ItemType::Material(MaterialType::Stick);
        let diamond = ItemType::Material(MaterialType::Diamond);

        let grid = [
            None,
            Some(ItemStack::new(diamond, 1)),
            None,
            None,
            Some(ItemStack::new(diamond, 1)),
            None,
            None,
            Some(ItemStack::new(stick, 1)),
            None,
        ];

        let out = match_recipe(&grid, 3, 3);
        assert_eq!(
            out,
            Some(ItemStack::tool(ToolType::Sword, ToolTier::Diamond))
        );
    }

    #[test]
    fn test_shaped_planks_to_sticks_anywhere() {
        let grid = [
            None,
            Some(ItemStack::block(BlockType::Planks, 1)),
            None,
            Some(ItemStack::block(BlockType::Planks, 1)),
        ];
        let out = match_recipe(&grid, 2, 2);
        assert_eq!(out, Some(ItemStack::material(MaterialType::Stick, 4)));
    }

    #[test]
    fn test_standard_3x3_axe_recipe_and_mirroring() {
        let stick = ItemType::Material(MaterialType::Stick);
        let planks = ItemType::Block(BlockType::Planks);

        // Mirrored axe (blade on right side)
        let grid = [
            None,
            Some(ItemStack::new(planks, 1)),
            Some(ItemStack::new(planks, 1)),
            None,
            Some(ItemStack::new(stick, 1)),
            Some(ItemStack::new(planks, 1)),
            None,
            Some(ItemStack::new(stick, 1)),
            None,
        ];

        let out = match_recipe(&grid, 3, 3);
        assert_eq!(out, Some(ItemStack::tool(ToolType::Axe, ToolTier::Wood)));
    }
}
