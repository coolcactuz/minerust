#![forbid(unsafe_code)]
#![allow(deprecated)]

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::inventory::ui::{minecraft_slot_border, minecraft_window_border};
use crate::inventory::{HOTBAR_SLOTS, Inventory, ItemStack, MAIN_INVENTORY_SLOTS, SlotLocation};
use crate::item_icon::ItemIconAssets;
use crate::menu::MenuState;
use crate::menu::types::Button;

pub const CHEST_SLOTS: usize = 27;

/// Persisted world-wide storage for all chests keyed by their 3D voxel position.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct ChestStorage {
    pub containers: HashMap<IVec3, [Option<ItemStack>; CHEST_SLOTS]>,
}

impl ChestStorage {
    /// Gets a mutable reference to the 27 slots of a chest at `pos`, creating it if empty.
    pub fn get_or_create(&mut self, pos: IVec3) -> &mut [Option<ItemStack>; CHEST_SLOTS] {
        self.containers.entry(pos).or_insert([None; CHEST_SLOTS])
    }

    /// Removes and yields all items from a chest destroyed at `pos`.
    pub fn remove_and_dump(&mut self, pos: IVec3) -> Vec<ItemStack> {
        self.containers
            .remove(&pos)
            .map_or_else(Vec::new, |slots| slots.into_iter().flatten().collect())
    }
}

/// Tracks which chest is currently being viewed by the player.
#[derive(Resource, Default, Debug, Clone)]
pub struct ChestOpenState {
    pub open_pos: Option<IVec3>,
}

impl ChestOpenState {
    pub fn is_open(&self) -> bool {
        self.open_pos.is_some()
    }

    pub fn close(&mut self) {
        self.open_pos = None;
    }
}

// Marker Components
#[derive(Component)]
pub struct ChestModal;

#[derive(Component)]
pub struct ChestSlotBtn(pub usize);

#[derive(Component)]
pub struct ChestIconUi(pub usize);

#[derive(Component)]
pub struct ChestCountUi(pub usize);

#[derive(Component)]
pub struct ChestModalInvBtn(pub SlotLocation);

#[derive(Component)]
pub struct ChestModalInvIcon(pub SlotLocation);

#[derive(Component)]
pub struct ChestModalInvCount(pub SlotLocation);

/// Spawns the Minecraft-authentic Chest GUI Modal with 27 chest slots on top and 36 player slots below.
pub fn setup_chest_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-240.0),
                    top: Val::Px(-280.0),
                    ..default()
                },
                width: Val::Px(480.0),
                height: Val::Px(560.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.776, 0.776, 0.776)),
            minecraft_window_border(),
            Visibility::Hidden,
            ChestModal,
        ))
        .with_children(|parent| {
            // Chest Section Label
            parent.spawn((
                Text::new("Chest"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.25, 0.25, 0.25)),
                Node {
                    width: Val::Px(428.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));

            // Chest 3x9 grid (27 slots): 428px x 140px
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    height: Val::Px(140.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..CHEST_SLOTS {
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.545, 0.545, 0.545)),
                            minecraft_slot_border(),
                            ChestSlotBtn(idx),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                ChestIconUi(idx),
                            ));
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(1.0),
                                    right: Val::Px(3.0),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(12.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                                ChestCountUi(idx),
                            ));
                        });
                    }
                });

            // Player Inventory Section Label
            parent.spawn((
                Text::new("Inventory"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.25, 0.25, 0.25)),
                Node {
                    width: Val::Px(428.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));

            // Main Inventory 3x9 grid (27 slots): 428px x 140px
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    height: Val::Px(140.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..MAIN_INVENTORY_SLOTS {
                        let loc = SlotLocation::Main(idx);
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.545, 0.545, 0.545)),
                            minecraft_slot_border(),
                            ChestModalInvBtn(loc),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                ChestModalInvIcon(loc),
                            ));
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(1.0),
                                    right: Val::Px(3.0),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(12.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                                ChestModalInvCount(loc),
                            ));
                        });
                    }
                });

            // Hotbar 1x9 grid: 428px x 44px
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    height: Val::Px(44.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..HOTBAR_SLOTS {
                        let loc = SlotLocation::Hotbar(idx);
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(if idx == 0 { 2.5 } else { 2.0 })),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.545, 0.545, 0.545)),
                            if idx == 0 {
                                BorderColor::all(Color::srgb(1.0, 1.0, 1.0))
                            } else {
                                minecraft_slot_border()
                            },
                            ChestModalInvBtn(loc),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(1.0),
                                    left: Val::Px(3.0),
                                    ..default()
                                },
                                Text::new(format!("{}", idx + 1)),
                                TextFont {
                                    font_size: FontSize::Px(10.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.30, 0.30, 0.30)),
                            ));
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                ChestModalInvIcon(loc),
                            ));
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(1.0),
                                    right: Val::Px(3.0),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(12.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                                ChestModalInvCount(loc),
                            ));
                        });
                    }
                });

            // Footer instructions
            parent.spawn((
                Text::new("Press [E] or [ESC] to close chest"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
            ));
        });
}

/// Handles player clicks inside the Chest modal window.
pub fn chest_interaction_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut chest_slot_query: Query<(&Interaction, &ChestSlotBtn), With<Button>>,
    mut inv_slot_query: Query<(&Interaction, &ChestModalInvBtn), With<Button>>,
    mut chest_storage: ResMut<ChestStorage>,
    chest_state: Res<ChestOpenState>,
    mut inventory: ResMut<Inventory>,
    sfx: Option<Res<crate::audio::SoundEffects>>,
) {
    let Some(chest_pos) = chest_state.open_pos else {
        return;
    };

    let left_clicked = mouse_buttons.just_pressed(MouseButton::Left);
    let right_clicked = mouse_buttons.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    let chest_slots = chest_storage.get_or_create(chest_pos);

    // 1. Clicks on chest container slots
    for (interaction, btn) in &mut chest_slot_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let slot_idx = btn.0;
            if slot_idx < CHEST_SLOTS {
                let slot_ref = &mut chest_slots[slot_idx];
                let prev_held = inventory.held_cursor_item;
                let prev_slot = *slot_ref;

                if left_clicked {
                    match (inventory.held_cursor_item, *slot_ref) {
                        (None, Some(s)) => {
                            inventory.held_cursor_item = Some(s);
                            *slot_ref = None;
                        }
                        (Some(held), None) => {
                            *slot_ref = Some(held);
                            inventory.held_cursor_item = None;
                        }
                        (Some(mut held), Some(mut s)) => {
                            if held.item == s.item {
                                let max = held.item.max_stack_size();
                                let space = max.saturating_sub(s.count);
                                let to_move = held.count.min(space);
                                s.count += to_move;
                                held.count -= to_move;
                                *slot_ref = Some(s);
                                inventory.held_cursor_item =
                                    if held.count > 0 { Some(held) } else { None };
                            } else {
                                std::mem::swap(&mut inventory.held_cursor_item, slot_ref);
                            }
                        }
                        (None, None) => {}
                    }
                } else if right_clicked {
                    match (inventory.held_cursor_item, *slot_ref) {
                        (None, Some(s)) => {
                            let take = s.count.div_ceil(2);
                            let remainder = s.count - take;
                            inventory.held_cursor_item = Some(ItemStack::new(s.item, take));
                            *slot_ref = if remainder > 0 {
                                Some(ItemStack::new(s.item, remainder))
                            } else {
                                None
                            };
                        }
                        (Some(mut held), None) => {
                            *slot_ref = Some(ItemStack::new(held.item, 1));
                            if held.count <= 1 {
                                inventory.held_cursor_item = None;
                            } else {
                                held.count -= 1;
                                inventory.held_cursor_item = Some(held);
                            }
                        }
                        (Some(mut held), Some(ref mut s)) => {
                            if s.item == held.item && s.count < s.item.max_stack_size() {
                                s.count += 1;
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
                }

                if inventory.held_cursor_item != prev_held || *slot_ref != prev_slot {
                    if let Some(ref sfx) = sfx {
                        crate::audio::play_sound(&mut commands, &sfx.click, 0.75);
                    }
                }
            }
        }
    }

    // 2. Clicks on player inventory slots inside the chest modal
    for (interaction, btn) in &mut inv_slot_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let slot_stack = match btn.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
                SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
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
                            let remaining = if held.count > 0 { Some(held) } else { None };
                            (remaining, Some(s))
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

            let changed = inventory.held_cursor_item != new_held || slot_stack != new_slot;
            inventory.held_cursor_item = new_held;
            match btn.0 {
                SlotLocation::Hotbar(idx) => {
                    if let Some(slot) = inventory.hotbar.get_mut(idx) {
                        *slot = new_slot;
                    }
                }
                SlotLocation::Main(idx) => {
                    if let Some(slot) = inventory.main.get_mut(idx) {
                        *slot = new_slot;
                    }
                }
            }
            if changed {
                if let Some(ref sfx) = sfx {
                    crate::audio::play_sound(&mut commands, &sfx.click, 0.75);
                }
            }
        }
    }
}

/// Updates chest icons and counts, and synchronizes modal visibility.
pub fn chest_ui_update_system(
    chest_state: Res<ChestOpenState>,
    chest_storage: Res<ChestStorage>,
    inventory: Res<Inventory>,
    item_icon_assets: Option<Res<ItemIconAssets>>,
    menu: Option<Res<MenuState>>,
    mut modal_query: Query<&mut Visibility, With<ChestModal>>,
    mut chest_icons_query: Query<(&ChestIconUi, &mut ImageNode)>,
    mut chest_counts_query: Query<(&ChestCountUi, &mut Text)>,
    mut inv_icons_query: Query<(&ChestModalInvIcon, &mut ImageNode), Without<ChestIconUi>>,
    mut inv_counts_query: Query<(&ChestModalInvCount, &mut Text), Without<ChestCountUi>>,
) {
    let menu_open = menu.is_some_and(|m| m.is_open());
    let Ok(mut vis) = modal_query.single_mut() else {
        return;
    };

    let target_vis = if chest_state.is_open() && !menu_open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    if *vis != target_vis {
        *vis = target_vis;
    }

    let Some(chest_pos) = chest_state.open_pos else {
        return;
    };

    let empty_chest = [None; CHEST_SLOTS];
    let slots = chest_storage
        .containers
        .get(&chest_pos)
        .unwrap_or(&empty_chest);

    // 1. Update Chest Slot Icons & Counts
    for (icon_ui, mut image_node) in &mut chest_icons_query {
        let idx = icon_ui.0;
        if let Some(stack) = slots.get(idx).copied().flatten() {
            if let Some(ref assets) = item_icon_assets {
                if let Some(handle) = assets.get(stack.item) {
                    image_node.image = handle.clone();
                    image_node.color = Color::WHITE;
                }
            }
        } else {
            image_node.image = bevy::image::TRANSPARENT_IMAGE_HANDLE;
            image_node.color = Color::NONE;
        }
    }

    for (count_ui, mut text) in &mut chest_counts_query {
        let idx = count_ui.0;
        if let Some(stack) = slots.get(idx).copied().flatten() {
            *text = Text::new(if stack.count > 1 {
                format!("{}", stack.count)
            } else {
                String::new()
            });
        } else {
            *text = Text::new("");
        }
    }

    // 2. Update Player Inventory Icons & Counts inside Chest Modal
    for (icon_ui, mut image_node) in &mut inv_icons_query {
        let item_opt = match icon_ui.0 {
            SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
            SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
        };
        if let Some(stack) = item_opt {
            if let Some(ref assets) = item_icon_assets {
                if let Some(handle) = assets.get(stack.item) {
                    image_node.image = handle.clone();
                    image_node.color = Color::WHITE;
                }
            }
        } else {
            image_node.image = bevy::image::TRANSPARENT_IMAGE_HANDLE;
            image_node.color = Color::NONE;
        }
    }

    for (count_ui, mut text) in &mut inv_counts_query {
        let item_opt = match count_ui.0 {
            SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
            SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
        };
        if let Some(stack) = item_opt {
            *text = Text::new(if stack.count > 1 {
                format!("{}", stack.count)
            } else {
                String::new()
            });
        } else {
            *text = Text::new("");
        }
    }
}

pub struct ChestPlugin;

impl Plugin for ChestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChestStorage>()
            .init_resource::<ChestOpenState>()
            .add_systems(Startup, setup_chest_ui)
            .add_systems(Update, (chest_interaction_system, chest_ui_update_system));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_chest_storage_insert_and_dump() {
        let mut storage = ChestStorage::default();
        let pos = IVec3::new(10, 64, -20);
        {
            let container = storage.get_or_create(pos);
            container[0] = Some(ItemStack::block(BlockType::Wood, 10));
            container[1] = Some(ItemStack::block(BlockType::Cobblestone, 64));
        }

        assert_eq!(storage.containers.len(), 1);
        let dumped = storage.remove_and_dump(pos);
        assert_eq!(dumped.len(), 2);
        assert_eq!(dumped[0], ItemStack::block(BlockType::Wood, 10));
        assert_eq!(dumped[1], ItemStack::block(BlockType::Cobblestone, 64));
        assert!(storage.containers.is_empty());
    }

    #[test]
    fn test_chest_multiple_containers_independent() {
        let mut storage = ChestStorage::default();
        let pos1 = IVec3::new(0, 10, 0);
        let pos2 = IVec3::new(5, 10, 5);

        {
            let c1 = storage.get_or_create(pos1);
            c1[0] = Some(ItemStack::block(BlockType::DiamondOre, 1));
        }
        {
            let c2 = storage.get_or_create(pos2);
            c2[0] = Some(ItemStack::block(BlockType::GoldOre, 2));
        }

        assert_eq!(storage.containers.len(), 2);
        assert_eq!(
            storage.containers.get(&pos1).unwrap()[0],
            Some(ItemStack::block(BlockType::DiamondOre, 1))
        );
        assert_eq!(
            storage.containers.get(&pos2).unwrap()[0],
            Some(ItemStack::block(BlockType::GoldOre, 2))
        );

        // Dumping pos1 should not affect pos2
        let dumped1 = storage.remove_and_dump(pos1);
        assert_eq!(dumped1.len(), 1);
        assert_eq!(dumped1[0], ItemStack::block(BlockType::DiamondOre, 1));
        assert_eq!(storage.containers.len(), 1);
        assert!(storage.containers.contains_key(&pos2));
    }

    #[test]
    fn test_chest_open_state() {
        let mut state = ChestOpenState::default();
        assert!(!state.is_open());
        assert_eq!(state.open_pos, None);

        state.open_pos = Some(IVec3::new(1, 2, 3));
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
        assert_eq!(state.open_pos, None);
    }
}
