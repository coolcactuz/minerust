#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::block::BlockType;
use crate::inventory::{Inventory, ItemStack, ItemType, MaterialType, ToolTier, ToolType};
use crate::menu::types::Button;

pub const CRAFTING_GRID_SIZE: usize = 4; // 2x2 grid

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

/// A shapeless crafting recipe where ingredients can appear in any grid arrangement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapelessRecipe {
    pub ingredients: Vec<ItemType>,
    pub output: ItemStack,
}

impl ShapelessRecipe {
    #[must_use]
    pub fn matches(&self, grid: &[Option<ItemStack>]) -> bool {
        let mut grid_items: Vec<ItemType> = grid
            .iter()
            .flatten()
            .filter(|s| !s.is_empty() && !s.item.is_air())
            .map(|s| s.item)
            .collect();

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

/// Static registry of available crafting recipes.
#[must_use]
pub fn all_recipes() -> Vec<Recipe> {
    vec![
        // 1. Wood -> 4 Planks (Shapeless)
        Recipe::Shapeless(ShapelessRecipe {
            ingredients: vec![ItemType::Block(BlockType::Wood)],
            output: ItemStack::new(BlockType::Planks, 4),
        }),
        // 2. 2 Planks vertical (1x2) -> 4 Sticks (Shaped)
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
        // 3. 4 Sand (2x2) -> 1 Sandstone
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
        // 4. 4 Cobblestone (2x2) -> 4 Stone
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
        // 5. Tools (2x2 compact and 3x3 standard)
        // 5a. Wooden Pickaxe (2x2: Planks Planks over Stick None)
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
        // 5b. Stone Pickaxe (2x2)
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
        // 5c. Iron Pickaxe (2x2)
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Material(MaterialType::IronIngot)),
                Some(ItemType::Material(MaterialType::IronIngot)),
                Some(ItemType::Material(MaterialType::Stick)),
                None,
            ],
            output: ItemStack::tool(ToolType::Pickaxe, ToolTier::Iron),
            mirrored: true,
        }),
        // 5d. Diamond Pickaxe (2x2)
        Recipe::Shaped(ShapedRecipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(ItemType::Material(MaterialType::Diamond)),
                Some(ItemType::Material(MaterialType::Diamond)),
                Some(ItemType::Material(MaterialType::Stick)),
                None,
            ],
            output: ItemStack::tool(ToolType::Pickaxe, ToolTier::Diamond),
            mirrored: true,
        }),
        // 5e. Wooden Axe (2x2)
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
        // 5f. Stone Axe (2x2)
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
        // 5g. Wooden Shovel (1x2)
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
        // 5h. Stone Shovel (1x2)
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
    ]
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

/// Active crafting state tracking the player's 2x2 grid and recipe output.
#[derive(Resource, Default, Debug, Clone)]
pub struct CraftingState {
    pub grid: [Option<ItemStack>; CRAFTING_GRID_SIZE],
    pub output: Option<ItemStack>,
}

impl CraftingState {
    /// Re-evaluates matching recipe and updates the output preview.
    pub fn update_output(&mut self) {
        self.output = match_recipe(&self.grid, 2, 2);
    }

    /// Clears any leftover crafting items back into the player's inventory.
    pub fn return_to_inventory(&mut self, inventory: &mut Inventory) {
        for slot in &mut self.grid {
            if let Some(stack) = slot.take() {
                inventory.add_item(stack.item, stack.count);
            }
        }
        self.output = None;
    }
}

// UI Marker Components
#[derive(Component)]
pub struct CraftingSlotBtn(pub usize);

#[derive(Component)]
pub struct CraftingSlotUi(pub usize);

#[derive(Component)]
pub struct CraftingIconUi(pub usize);

#[derive(Component)]
pub struct CraftingCountUi(pub usize);

#[derive(Component)]
pub struct CraftingResultBtn;

#[derive(Component)]
pub struct CraftingResultIconUi;

#[derive(Component)]
pub struct CraftingResultCountUi;

/// Spawns the 2x2 crafting UI section inside the inventory modal parent.
pub fn spawn_crafting_widget(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Text::new("Crafting (2x2)"),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.2)),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            align_self: AlignSelf::FlexStart,
            ..default()
        },
    ));

    parent
        .spawn(Node {
            width: Val::Px(450.0),
            height: Val::Px(104.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|crafting_row| {
            // 2x2 Crafting Grid (4 slots)
            crafting_row
                .spawn(Node {
                    width: Val::Px(104.0),
                    height: Val::Px(104.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..CRAFTING_GRID_SIZE {
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 0.9)),
                            BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6)),
                            CraftingSlotBtn(idx),
                            CraftingSlotUi(idx),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                                CraftingIconUi(idx),
                            ));
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(2.0),
                                    right: Val::Px(4.0),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                                CraftingCountUi(idx),
                            ));
                        });
                    }
                });

            // Arrow separator (==>)
            crafting_row.spawn((
                Text::new("➔"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                Node {
                    margin: UiRect::axes(Val::Px(24.0), Val::Px(0.0)),
                    ..default()
                },
            ));

            // Result slot (larger with prominent gold border)
            crafting_row
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(56.0),
                        height: Val::Px(56.0),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.5)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.22, 0.22, 0.30, 0.95)),
                    BorderColor::all(Color::srgb(1.0, 0.85, 0.2)),
                    CraftingResultBtn,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        CraftingResultIconUi,
                    ));
                    btn.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(2.0),
                            right: Val::Px(4.0),
                            ..default()
                        },
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 0.3)),
                        CraftingResultCountUi,
                    ));
                });
        });
}

/// Handles player clicks on crafting input slots and result button.
pub fn crafting_interaction_system(
    mut crafting_slot_query: Query<
        (&Interaction, &CraftingSlotBtn),
        (Changed<Interaction>, With<Button>),
    >,
    mut crafting_result_query: Query<
        &Interaction,
        (Changed<Interaction>, With<CraftingResultBtn>, With<Button>),
    >,
    mut crafting_state: ResMut<CraftingState>,
    mut inventory: ResMut<Inventory>,
) {
    // 1. Clicks on crafting input slots
    for (interaction, btn) in &mut crafting_slot_query {
        if *interaction == Interaction::Pressed {
            let slot_idx = btn.0;
            if slot_idx < CRAFTING_GRID_SIZE {
                let active_hotbar_idx = inventory.selected_slot;
                let hotbar_stack = inventory.hotbar[active_hotbar_idx];

                if let Some(mut h_stack) = hotbar_stack {
                    if let Some(ref mut c_stack) = crafting_state.grid[slot_idx] {
                        if c_stack.item == h_stack.item
                            && c_stack.count < c_stack.item.max_stack_size()
                        {
                            // Add 1 item from hotbar to matching crafting stack
                            c_stack.count += 1;
                            if h_stack.count <= 1 {
                                inventory.hotbar[active_hotbar_idx] = None;
                            } else {
                                h_stack.count -= 1;
                                inventory.hotbar[active_hotbar_idx] = Some(h_stack);
                            }
                        } else {
                            // Swap crafting slot with hotbar slot
                            let old_c = crafting_state.grid[slot_idx];
                            crafting_state.grid[slot_idx] = Some(h_stack);
                            inventory.hotbar[active_hotbar_idx] = old_c;
                        }
                    } else {
                        // Place 1 item from hotbar into empty crafting slot
                        crafting_state.grid[slot_idx] = Some(ItemStack::new(h_stack.item, 1));
                        if h_stack.count <= 1 {
                            inventory.hotbar[active_hotbar_idx] = None;
                        } else {
                            h_stack.count -= 1;
                            inventory.hotbar[active_hotbar_idx] = Some(h_stack);
                        }
                    }
                } else if let Some(c_stack) = crafting_state.grid[slot_idx].take() {
                    // Pick up entire stack from crafting grid into hotbar
                    inventory.hotbar[active_hotbar_idx] = Some(c_stack);
                }

                crafting_state.update_output();
            }
        }
    }

    // 2. Click on crafting result button
    for interaction in &mut crafting_result_query {
        if *interaction == Interaction::Pressed {
            if let Some(output_stack) = crafting_state.output {
                // Try to store crafted item in player inventory
                let leftover = inventory.add_item(output_stack.item, output_stack.count);
                if leftover == 0 {
                    // Consumes 1 item from each non-empty crafting grid slot
                    for slot in &mut crafting_state.grid {
                        if let Some(s) = slot {
                            if s.count <= 1 {
                                *slot = None;
                            } else {
                                s.count -= 1;
                            }
                        }
                    }
                    crafting_state.update_output();
                } else {
                    tracing::info!("Inventory full! Cannot collect crafted output.");
                }
            }
        }
    }
}

/// Updates colors and counts displayed on crafting slot and result buttons.
pub fn crafting_ui_update_system(
    crafting_state: Res<CraftingState>,
    inventory: Res<Inventory>,
    mut slot_icons: Query<(&CraftingIconUi, &mut BackgroundColor)>,
    mut slot_counts: Query<(&CraftingCountUi, &mut Text)>,
    mut result_icon: Query<
        &mut BackgroundColor,
        (With<CraftingResultIconUi>, Without<CraftingIconUi>),
    >,
    mut result_count: Query<&mut Text, (With<CraftingResultCountUi>, Without<CraftingCountUi>)>,
) {
    if !inventory.is_open {
        return;
    }

    // Update 2x2 crafting input slot visuals
    for (icon_ui, mut bg) in &mut slot_icons {
        if let Some(stack) = crafting_state.grid.get(icon_ui.0).copied().flatten() {
            let col = stack.item.color();
            bg.0 = Color::srgb(col[0], col[1], col[2]);
        } else {
            bg.0 = Color::NONE;
        }
    }

    for (count_ui, mut text) in &mut slot_counts {
        if let Some(stack) = crafting_state.grid.get(count_ui.0).copied().flatten() {
            *text = Text::new(format!("{}", stack.count));
        } else {
            *text = Text::new("");
        }
    }

    // Update result slot visuals
    if let Ok(mut bg) = result_icon.single_mut() {
        if let Some(output) = crafting_state.output {
            let col = output.item.color();
            bg.0 = Color::srgb(col[0], col[1], col[2]);
        } else {
            bg.0 = Color::NONE;
        }
    }

    if let Ok(mut text) = result_count.single_mut() {
        if let Some(output) = crafting_state.output {
            *text = Text::new(format!("{}", output.count));
        } else {
            *text = Text::new("");
        }
    }
}

/// Automatically returns any items left on the crafting grid when inventory is closed.
pub fn crafting_return_on_close_system(
    mut crafting_state: ResMut<CraftingState>,
    mut inventory: ResMut<Inventory>,
) {
    if !inventory.is_open {
        let has_items = crafting_state
            .grid
            .iter()
            .any(|s| s.is_some_and(|st| !st.is_empty()));
        if has_items {
            crafting_state.return_to_inventory(&mut inventory);
        }
    }
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CraftingState>().add_systems(
            Update,
            (
                crafting_interaction_system,
                crafting_ui_update_system,
                crafting_return_on_close_system,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shapeless_wood_to_planks() {
        let mut grid = [None; 4];
        // Placed in bottom-right slot of 2x2
        grid[3] = Some(ItemStack::new(BlockType::Wood, 1));

        let res = match_recipe(&grid, 2, 2);
        assert_eq!(res, Some(ItemStack::new(BlockType::Planks, 4)));
    }

    #[test]
    fn test_shaped_planks_to_sticks_anywhere() {
        // Test in column 0 (slots 0 and 2)
        let mut grid = [None; 4];
        grid[0] = Some(ItemStack::new(BlockType::Planks, 1));
        grid[2] = Some(ItemStack::new(BlockType::Planks, 1));

        let res = match_recipe(&grid, 2, 2);
        assert_eq!(res, Some(ItemStack::material(MaterialType::Stick, 4)));

        // Test in column 1 (slots 1 and 3)
        let mut grid2 = [None; 4];
        grid2[1] = Some(ItemStack::new(BlockType::Planks, 1));
        grid2[3] = Some(ItemStack::new(BlockType::Planks, 1));

        let res2 = match_recipe(&grid2, 2, 2);
        assert_eq!(res2, Some(ItemStack::material(MaterialType::Stick, 4)));
    }

    #[test]
    fn test_tool_pickaxe_crafting_and_mirroring() {
        // Planks Planks over Stick None
        let mut grid = [None; 4];
        grid[0] = Some(ItemStack::new(BlockType::Planks, 1));
        grid[1] = Some(ItemStack::new(BlockType::Planks, 1));
        grid[2] = Some(ItemStack::material(MaterialType::Stick, 1));
        grid[3] = None;

        let res = match_recipe(&grid, 2, 2);
        assert_eq!(
            res,
            Some(ItemStack::tool(ToolType::Pickaxe, ToolTier::Wood))
        );

        // Mirrored: Planks Planks over None Stick
        let mut grid_mirrored = [None; 4];
        grid_mirrored[0] = Some(ItemStack::new(BlockType::Planks, 1));
        grid_mirrored[1] = Some(ItemStack::new(BlockType::Planks, 1));
        grid_mirrored[2] = None;
        grid_mirrored[3] = Some(ItemStack::material(MaterialType::Stick, 1));

        let res_mirrored = match_recipe(&grid_mirrored, 2, 2);
        assert_eq!(
            res_mirrored,
            Some(ItemStack::tool(ToolType::Pickaxe, ToolTier::Wood))
        );
    }

    #[test]
    fn test_return_to_inventory_on_close() {
        let mut inv = Inventory::default();
        let mut craft = CraftingState::default();
        craft.grid[0] = Some(ItemStack::new(BlockType::Wood, 5));
        craft.grid[3] = Some(ItemStack::material(MaterialType::Stick, 2));

        craft.return_to_inventory(&mut inv);

        assert_eq!(craft.grid[0], None);
        assert_eq!(craft.grid[3], None);
        assert_eq!(craft.output, None);
        assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Wood, 5)));
        assert_eq!(
            inv.hotbar[1],
            Some(ItemStack::material(MaterialType::Stick, 2))
        );
    }
}
