#![forbid(unsafe_code)]
#![allow(deprecated)]

pub mod recipes;
pub mod ui;

use bevy::prelude::*;

use crate::inventory::{Inventory, ItemStack};
use crate::menu::types::Button;

pub use recipes::{
    GridBounds, Recipe, ShapedRecipe, ShapelessRecipe, all_recipes, get_grid_bounds, match_recipe,
};
pub use ui::{
    CraftingCountUi, CraftingIconUi, CraftingResultBtn, CraftingResultCountUi,
    CraftingResultIconUi, CraftingSlotBtn, CraftingSlotUi, CraftingTitleUi, CursorFollower,
    cursor_follower_update_system, spawn_crafting_widget,
};

pub const GRID_SIZE: usize = 9; // 3x3 max grid size

/// Active crafting context: 2x2 personal inventory crafting or 3x3 workbench table.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum CraftingMode {
    #[default]
    Personal2x2,
    Table3x3,
}

/// Active crafting state tracking the 3x3 grid slots, mode, and output preview.
#[derive(Resource, Default, Debug, Clone)]
pub struct CraftingState {
    pub grid: [Option<ItemStack>; GRID_SIZE],
    pub output: Option<ItemStack>,
    pub mode: CraftingMode,
    pub is_table_open: bool,
}

impl CraftingState {
    /// Re-evaluates matching recipes and updates the output preview.
    pub fn update_output(&mut self) {
        match self.mode {
            CraftingMode::Personal2x2 => {
                // In 2x2 mode, map slots (0, 1) and (3, 4) into a 2x2 array
                let grid_2x2 = [self.grid[0], self.grid[1], self.grid[3], self.grid[4]];
                self.output = match_recipe(&grid_2x2, 2, 2);
            }
            CraftingMode::Table3x3 => {
                self.output = match_recipe(&self.grid, 3, 3);
            }
        }
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

    /// Consumes one item from each active ingredient slot after crafting.
    pub fn consume_ingredients(&mut self) {
        let is_2x2 = self.mode == CraftingMode::Personal2x2;
        for (idx, slot) in self.grid.iter_mut().enumerate() {
            let is_active = if is_2x2 {
                idx == 0 || idx == 1 || idx == 3 || idx == 4
            } else {
                idx < GRID_SIZE
            };

            if is_active {
                if let Some(s) = slot {
                    if s.count <= 1 {
                        *slot = None;
                    } else {
                        s.count -= 1;
                    }
                }
            }
        }
    }
}

/// Handles player left and right clicks on crafting input slots and result button.
pub fn crafting_interaction_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut crafting_slot_query: Query<(&Interaction, &CraftingSlotBtn), With<Button>>,
    mut crafting_result_query: Query<&Interaction, (With<CraftingResultBtn>, With<Button>)>,
    mut crafting_state: ResMut<CraftingState>,
    mut inventory: ResMut<Inventory>,
) {
    let left_clicked = mouse_buttons.just_pressed(MouseButton::Left);
    let right_clicked = mouse_buttons.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    // 1. Clicks on crafting input slots
    for (interaction, btn) in &mut crafting_slot_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let slot_idx = btn.0;
            if slot_idx < GRID_SIZE {
                let slot_ref = &mut crafting_state.grid[slot_idx];

                if left_clicked {
                    match (inventory.held_cursor_item, *slot_ref) {
                        (None, Some(slot_stack)) => {
                            // Pick up entire stack from crafting grid into cursor
                            inventory.held_cursor_item = Some(slot_stack);
                            *slot_ref = None;
                        }
                        (Some(held), None) => {
                            // Place entire held stack into empty crafting slot
                            *slot_ref = Some(held);
                            inventory.held_cursor_item = None;
                        }
                        (Some(mut held), Some(mut slot_stack)) => {
                            if held.item == slot_stack.item {
                                // Merge matching stacks up to max
                                let max = held.item.max_stack_size();
                                let space = max.saturating_sub(slot_stack.count);
                                let to_move = held.count.min(space);
                                slot_stack.count += to_move;
                                held.count -= to_move;
                                *slot_ref = Some(slot_stack);
                                inventory.held_cursor_item =
                                    if held.count > 0 { Some(held) } else { None };
                            } else {
                                // Swap held item with crafting slot
                                std::mem::swap(&mut inventory.held_cursor_item, slot_ref);
                            }
                        }
                        (None, None) => {}
                    }
                    crafting_state.update_output();
                } else if right_clicked {
                    match (inventory.held_cursor_item, *slot_ref) {
                        (None, Some(slot_stack)) => {
                            // Split stack: cursor gets half (rounded up), crafting slot keeps remainder
                            let take = slot_stack.count.div_ceil(2);
                            let remainder = slot_stack.count - take;
                            inventory.held_cursor_item =
                                Some(ItemStack::new(slot_stack.item, take));
                            *slot_ref = if remainder > 0 {
                                Some(ItemStack::new(slot_stack.item, remainder))
                            } else {
                                None
                            };
                        }
                        (Some(mut held), None) => {
                            // Place 1 single item into empty crafting slot
                            *slot_ref = Some(ItemStack::new(held.item, 1));
                            if held.count <= 1 {
                                inventory.held_cursor_item = None;
                            } else {
                                held.count -= 1;
                                inventory.held_cursor_item = Some(held);
                            }
                        }
                        (Some(mut held), Some(ref mut slot_stack)) => {
                            // Deposit 1 item into matching crafting slot
                            if slot_stack.item == held.item
                                && slot_stack.count < slot_stack.item.max_stack_size()
                            {
                                slot_stack.count += 1;
                                if held.count <= 1 {
                                    inventory.held_cursor_item = None;
                                } else {
                                    held.count -= 1;
                                    inventory.held_cursor_item = Some(held);
                                }
                            }
                        }
                        (None, None) => {}
                    }
                    crafting_state.update_output();
                }
            }
        }
    }

    // 2. Clicks on crafting result button (Left click crafts)
    if left_clicked {
        for interaction in &mut crafting_result_query {
            if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
                if let Some(output_stack) = crafting_state.output {
                    match inventory.held_cursor_item {
                        None => {
                            // Collect crafted output onto empty cursor
                            inventory.held_cursor_item = Some(output_stack);
                            crafting_state.consume_ingredients();
                            crafting_state.update_output();
                        }
                        Some(mut held) => {
                            // Merge into matching cursor stack if space allows
                            if held.item == output_stack.item
                                && held.count + output_stack.count <= held.item.max_stack_size()
                            {
                                held.count += output_stack.count;
                                inventory.held_cursor_item = Some(held);
                                crafting_state.consume_ingredients();
                                crafting_state.update_output();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Automatically returns crafting items and held cursor items to inventory when closing modal.
pub fn crafting_return_on_close_system(
    mut crafting_state: ResMut<CraftingState>,
    mut inventory: ResMut<Inventory>,
) {
    if !inventory.is_open && !crafting_state.is_table_open {
        // Return leftover crafting grid items
        crafting_state.return_to_inventory(&mut inventory);
        // Return any item held on the cursor
        inventory.return_held_cursor_item();
    }
}

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CraftingState>()
            .add_systems(Startup, ui::spawn_cursor_follower)
            .add_systems(
                Update,
                (
                    crafting_interaction_system,
                    ui::crafting_ui_update_system,
                    ui::cursor_follower_update_system,
                    crafting_return_on_close_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_crafting_state_return_to_inventory() {
        let mut state = CraftingState::default();
        let mut inv = Inventory::default();

        state.grid[0] = Some(ItemStack::block(BlockType::Wood, 5));
        state.grid[1] = Some(ItemStack::block(BlockType::Stone, 10));

        state.return_to_inventory(&mut inv);

        assert!(state.grid[0].is_none());
        assert!(state.grid[1].is_none());
        assert_eq!(inv.hotbar[0], Some(ItemStack::block(BlockType::Wood, 5)));
        assert_eq!(inv.hotbar[1], Some(ItemStack::block(BlockType::Stone, 10)));
    }

    #[test]
    fn test_crafting_consume_ingredients_2x2() {
        let mut state = CraftingState {
            mode: CraftingMode::Personal2x2,
            ..default()
        };

        state.grid[0] = Some(ItemStack::block(BlockType::Planks, 2));
        state.grid[1] = Some(ItemStack::block(BlockType::Planks, 1));

        state.consume_ingredients();

        assert_eq!(state.grid[0], Some(ItemStack::block(BlockType::Planks, 1)));
        assert_eq!(state.grid[1], None);
    }
}
