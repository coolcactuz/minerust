#![allow(deprecated)]

pub mod ui;

#[cfg(test)]
mod tests;

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::BlockType;
use crate::menu::MenuState;
use crate::menu::types::Button;

pub use crate::item::{
    ItemStack, ItemType, MAX_STACK_SIZE, MaterialType, ToolTier, ToolType, block_drop,
};
pub use ui::{
    HotbarCountUi, HotbarIconUi, HotbarRoot, HotbarSlotUi, InventoryModal, ModalCountUi,
    ModalIconUi, ModalSlotBtn, ModalSlotUi, setup_inventory_ui, update_inventory_ui_system,
};

pub const HOTBAR_SLOTS: usize = 9;
pub const MAIN_INVENTORY_SLOTS: usize = 27;

pub const ALL_BLOCKS: &[BlockType] = &[
    BlockType::Grass,
    BlockType::Dirt,
    BlockType::Stone,
    BlockType::Cobblestone,
    BlockType::Wood,
    BlockType::Planks,
    BlockType::Leaves,
    BlockType::Glass,
    BlockType::Sand,
    BlockType::Sandstone,
    BlockType::Gravel,
    BlockType::Water,
    BlockType::Ice,
    BlockType::Snow,
    BlockType::Cactus,
    BlockType::CoalOre,
    BlockType::IronOre,
    BlockType::GoldOre,
    BlockType::DiamondOre,
    BlockType::Bedrock,
    BlockType::CraftingTable,
];

pub const fn block_name(block: BlockType) -> &'static str {
    match block {
        BlockType::Air => "Air",
        BlockType::Grass => "Grass",
        BlockType::Dirt => "Dirt",
        BlockType::Stone => "Stone",
        BlockType::Cobblestone => "Cobblestone",
        BlockType::Wood => "Wood",
        BlockType::Planks => "Planks",
        BlockType::Leaves => "Leaves",
        BlockType::Glass => "Glass",
        BlockType::Sand => "Sand",
        BlockType::Sandstone => "Sandstone",
        BlockType::Gravel => "Gravel",
        BlockType::Water => "Water",
        BlockType::Ice => "Ice",
        BlockType::Snow => "Snow",
        BlockType::Cactus => "Cactus",
        BlockType::CoalOre => "Coal Ore",
        BlockType::IronOre => "Iron Ore",
        BlockType::GoldOre => "Gold Ore",
        BlockType::DiamondOre => "Diamond Ore",
        BlockType::Bedrock => "Bedrock",
        BlockType::CraftingTable => "Crafting Table",
    }
}

/// Player inventory state with an empty starting hotbar and main inventory.
#[derive(Resource, Debug, Clone)]
pub struct Inventory {
    pub hotbar: [Option<ItemStack>; HOTBAR_SLOTS],
    pub main: [Option<ItemStack>; MAIN_INVENTORY_SLOTS],
    pub selected_slot: usize,
    pub is_open: bool,
    pub held_cursor_item: Option<ItemStack>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            hotbar: [None; HOTBAR_SLOTS],
            main: [None; MAIN_INVENTORY_SLOTS],
            selected_slot: 0,
            is_open: false,
            held_cursor_item: None,
        }
    }
}

impl Inventory {
    /// Returns any item currently held by the cursor back to the inventory.
    pub fn return_held_cursor_item(&mut self) {
        if let Some(stack) = self.held_cursor_item.take() {
            let leftover = self.add_item(stack.item, stack.count);
            if leftover > 0 {
                tracing::warn!("Inventory full! Dropped leftover cursor item {stack:?}");
            }
        }
    }

    /// Returns the item stack in the currently selected hotbar slot, if any.
    #[inline]
    #[must_use]
    pub fn selected_item(&self) -> Option<ItemStack> {
        self.hotbar[self.selected_slot]
    }

    /// Returns the block type in the currently selected hotbar slot, or Air if empty.
    #[inline]
    #[must_use]
    pub fn selected_block(&self) -> BlockType {
        self.hotbar[self.selected_slot].map_or(BlockType::Air, |s| s.block_type())
    }

    /// Consumes `amount` blocks from the currently selected hotbar slot.
    /// Returns true if items were consumed, false if the slot was empty.
    pub fn consume_selected(&mut self, amount: u32) -> bool {
        if let Some(ref mut stack) = self.hotbar[self.selected_slot] {
            if stack.count > amount {
                stack.count -= amount;
                true
            } else {
                self.hotbar[self.selected_slot] = None;
                true
            }
        } else {
            false
        }
    }

    /// Adds items of a given item type (or block) into the inventory.
    ///
    /// 1. Merges into existing non-full stacks (hotbar first, then main).
    /// 2. Fills empty slots (hotbar first, then main).
    ///
    /// Returns 0 if all items were added, or the remaining count if inventory is full.
    pub fn add_item(&mut self, item: impl Into<ItemType>, mut amount: u32) -> u32 {
        let item = item.into();
        if item.is_air() || amount == 0 {
            return amount;
        }

        let max_stack = item.max_stack_size();

        // 1. Merge into existing matching stacks in hotbar
        for slot in self.hotbar.iter_mut().flatten() {
            if slot.item == item && slot.count < max_stack {
                let space = max_stack - slot.count;
                let to_add = amount.min(space);
                slot.count += to_add;
                amount -= to_add;
                if amount == 0 {
                    return 0;
                }
            }
        }

        // 2. Merge into existing matching stacks in main inventory
        for slot in self.main.iter_mut().flatten() {
            if slot.item == item && slot.count < max_stack {
                let space = max_stack - slot.count;
                let to_add = amount.min(space);
                slot.count += to_add;
                amount -= to_add;
                if amount == 0 {
                    return 0;
                }
            }
        }

        // 3. Place into first empty slot in hotbar
        for slot in &mut self.hotbar {
            if slot.is_none() {
                let to_add = amount.min(max_stack);
                *slot = Some(ItemStack::new(item, to_add));
                amount -= to_add;
                if amount == 0 {
                    return 0;
                }
            }
        }

        // 4. Place into first empty slot in main inventory
        for slot in &mut self.main {
            if slot.is_none() {
                let to_add = amount.min(max_stack);
                *slot = Some(ItemStack::new(item, to_add));
                amount -= to_add;
                if amount == 0 {
                    return 0;
                }
            }
        }

        amount
    }

    /// Swaps or transfers items between a slot in main inventory and the active hotbar slot.
    pub fn swap_main_with_active_hotbar(&mut self, main_idx: usize) {
        if main_idx < MAIN_INVENTORY_SLOTS && self.selected_slot < HOTBAR_SLOTS {
            std::mem::swap(
                &mut self.main[main_idx],
                &mut self.hotbar[self.selected_slot],
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotLocation {
    Hotbar(usize),
    Main(usize),
}

pub fn inventory_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut inventory: ResMut<Inventory>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    menu: Option<Res<MenuState>>,
    mut crafting_state: Option<ResMut<crate::crafting::CraftingState>>,
) {
    if menu.is_some_and(|m| m.is_open()) {
        return;
    }

    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    // Key E: Open/Close Inventory
    if keys.just_pressed(KeyCode::KeyE) {
        inventory.is_open = !inventory.is_open;

        if inventory.is_open {
            if let Some(ref mut cs) = crafting_state {
                cs.mode = crate::crafting::CraftingMode::Personal2x2;
                cs.is_table_open = false;
                cs.update_output();
            }
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        } else {
            inventory.return_held_cursor_item();
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    }

    // ESC closes inventory if currently open
    if keys.just_pressed(KeyCode::Escape) && inventory.is_open {
        inventory.is_open = false;
        inventory.return_held_cursor_item();
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }

    // If inventory is closed, handle Hotbar selection (1-9 and Mouse Wheel)
    if !inventory.is_open {
        if keys.just_pressed(KeyCode::Digit1) {
            inventory.selected_slot = 0;
        } else if keys.just_pressed(KeyCode::Digit2) {
            inventory.selected_slot = 1;
        } else if keys.just_pressed(KeyCode::Digit3) {
            inventory.selected_slot = 2;
        } else if keys.just_pressed(KeyCode::Digit4) {
            inventory.selected_slot = 3;
        } else if keys.just_pressed(KeyCode::Digit5) {
            inventory.selected_slot = 4;
        } else if keys.just_pressed(KeyCode::Digit6) {
            inventory.selected_slot = 5;
        } else if keys.just_pressed(KeyCode::Digit7) {
            inventory.selected_slot = 6;
        } else if keys.just_pressed(KeyCode::Digit8) {
            inventory.selected_slot = 7;
        } else if keys.just_pressed(KeyCode::Digit9) {
            inventory.selected_slot = 8;
        }

        // Mouse wheel scroll to cycle hotbar slots
        if scroll.delta.y < -0.1 {
            inventory.selected_slot = (inventory.selected_slot + 1) % HOTBAR_SLOTS;
        } else if scroll.delta.y > 0.1 {
            inventory.selected_slot = (inventory.selected_slot + HOTBAR_SLOTS - 1) % HOTBAR_SLOTS;
        }
    }
}

pub fn inventory_interaction_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut interaction_query: Query<(&Interaction, &ModalSlotBtn, &mut BorderColor), With<Button>>,
    mut inventory: ResMut<Inventory>,
) {
    let left_clicked = mouse_buttons.just_pressed(MouseButton::Left);
    let right_clicked = mouse_buttons.just_pressed(MouseButton::Right);

    for (interaction, btn_slot, mut border) in &mut interaction_query {
        let is_hovered =
            *interaction == Interaction::Hovered || *interaction == Interaction::Pressed;

        if is_hovered && (left_clicked || right_clicked) {
            if let SlotLocation::Hotbar(idx) = btn_slot.0 {
                inventory.selected_slot = idx;
            }

            let slot_stack = match btn_slot.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar[idx],
                SlotLocation::Main(idx) => inventory.main[idx],
            };

            let (new_held, new_slot) = if left_clicked {
                match (inventory.held_cursor_item, slot_stack) {
                    (None, Some(s)) => (Some(s), None),
                    (Some(held), None) => (None, Some(held)),
                    (Some(mut held), Some(mut s)) => {
                        if held.item == s.item {
                            let max = held.item.max_stack_size();
                            let space = max.saturating_sub(s.count);
                            let to_move = held.count.min(space);
                            s.count += to_move;
                            held.count -= to_move;
                            let remaining_held = if held.count > 0 { Some(held) } else { None };
                            (remaining_held, Some(s))
                        } else {
                            (Some(s), Some(held))
                        }
                    }
                    (None, None) => (None, None),
                }
            } else {
                match (inventory.held_cursor_item, slot_stack) {
                    (None, Some(s)) => {
                        let take = s.count.div_ceil(2);
                        let remainder = s.count - take;
                        let held = Some(ItemStack::new(s.item, take));
                        let slot = if remainder > 0 {
                            Some(ItemStack::new(s.item, remainder))
                        } else {
                            None
                        };
                        (held, slot)
                    }
                    (Some(mut held), None) => {
                        let slot = Some(ItemStack::new(held.item, 1));
                        let held = if held.count <= 1 {
                            None
                        } else {
                            held.count -= 1;
                            Some(held)
                        };
                        (held, slot)
                    }
                    (Some(mut held), Some(mut s)) => {
                        if s.item == held.item && s.count < s.item.max_stack_size() {
                            s.count += 1;
                            let held = if held.count <= 1 {
                                None
                            } else {
                                held.count -= 1;
                                Some(held)
                            };
                            (held, Some(s))
                        } else {
                            (Some(held), Some(s))
                        }
                    }
                    (None, None) => (None, None),
                }
            };

            inventory.held_cursor_item = new_held;
            match btn_slot.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar[idx] = new_slot,
                SlotLocation::Main(idx) => inventory.main[idx] = new_slot,
            }
        }

        match *interaction {
            Interaction::Pressed => {
                *border = BorderColor::all(Color::srgb(1.0, 0.9, 0.2));
            }
            Interaction::Hovered => {
                *border = BorderColor::all(Color::srgb(0.9, 0.9, 1.0));
            }
            Interaction::None => {
                let is_selected_hotbar = match btn_slot.0 {
                    SlotLocation::Hotbar(idx) => idx == inventory.selected_slot,
                    SlotLocation::Main(_) => false,
                };
                *border = if is_selected_hotbar {
                    BorderColor::all(Color::srgb(1.0, 0.85, 0.2))
                } else {
                    BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6))
                };
            }
        }
    }
}

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_systems(Startup, setup_inventory_ui)
            .add_systems(
                Update,
                (
                    inventory_input_system.in_set(crate::stage::VoxelStage::InputHandling),
                    inventory_interaction_system,
                    update_inventory_ui_system,
                ),
            );
    }
}
