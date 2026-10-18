#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;

use super::{FurnaceData, FurnaceOpenState, FurnaceStorage};
use crate::inventory::ui::{minecraft_slot_border, minecraft_window_border};
use crate::inventory::{HOTBAR_SLOTS, Inventory, ItemStack, MAIN_INVENTORY_SLOTS, SlotLocation};
use crate::item_icon::ItemIconAssets;
use crate::menu::MenuState;
use crate::menu::types::Button;

#[derive(Component)]
pub struct FurnaceModal;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FurnaceSlotType {
    Input,
    Fuel,
    Output,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FurnaceSlotBtn(pub FurnaceSlotType);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FurnaceSlotIcon(pub FurnaceSlotType);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FurnaceSlotCount(pub FurnaceSlotType);

#[derive(Component)]
pub struct FurnaceFlameText;

#[derive(Component)]
pub struct FurnaceCookProgressText;

#[derive(Component)]
pub struct FurnaceModalInvBtn(pub SlotLocation);

#[derive(Component)]
pub struct FurnaceModalInvIcon(pub SlotLocation);

#[derive(Component)]
pub struct FurnaceModalInvCount(pub SlotLocation);

/// Spawns the Minecraft Furnace GUI Modal.
pub fn setup_furnace_ui(mut commands: Commands) {
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
            FurnaceModal,
        ))
        .with_children(|parent| {
            // Header Label
            parent.spawn((
                Text::new("Furnace"),
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

            // Furnace Processing Section (Input, Fuel, Flame, Arrow, Output)
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    height: Val::Px(140.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    // Left Column: Input Slot (top) -> Flame (mid) -> Fuel Slot (bottom)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        height: Val::Px(132.0),
                        margin: UiRect::right(Val::Px(24.0)),
                        ..default()
                    })
                    .with_children(|col| {
                        // Input slot
                        col.spawn((
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
                            FurnaceSlotBtn(FurnaceSlotType::Input),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                FurnaceSlotIcon(FurnaceSlotType::Input),
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
                                FurnaceSlotCount(FurnaceSlotType::Input),
                            ));
                        });

                        // Flame icon (dormant gray or fiery orange when burning)
                        col.spawn((
                            Text::new("🔥"),
                            TextFont {
                                font_size: FontSize::Px(18.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.35, 0.35, 0.35)),
                            FurnaceFlameText,
                        ));

                        // Fuel slot
                        col.spawn((
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
                            FurnaceSlotBtn(FurnaceSlotType::Fuel),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                FurnaceSlotIcon(FurnaceSlotType::Fuel),
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
                                FurnaceSlotCount(FurnaceSlotType::Fuel),
                            ));
                        });
                    });

                    // Middle Column: Smelting Arrow & Percentage
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::right(Val::Px(24.0)),
                        ..default()
                    })
                    .with_children(|mid| {
                        mid.spawn((
                            Text::new("➜"),
                            TextFont {
                                font_size: FontSize::Px(28.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.40, 0.40, 0.40)),
                        ));
                        mid.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.25, 0.25, 0.25)),
                            FurnaceCookProgressText,
                        ));
                    });

                    // Right Column: Output Slot (Large)
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(52.0),
                            height: Val::Px(52.0),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.545, 0.545, 0.545)),
                        minecraft_slot_border(),
                        FurnaceSlotBtn(FurnaceSlotType::Output),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Node {
                                width: Val::Px(36.0),
                                height: Val::Px(36.0),
                                ..default()
                            },
                            ImageNode::default(),
                            FurnaceSlotIcon(FurnaceSlotType::Output),
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
                            FurnaceSlotCount(FurnaceSlotType::Output),
                        ));
                    });
                });

            // Player Inventory Label
            parent.spawn((
                Text::new("Inventory"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.25, 0.25, 0.25)),
                Node {
                    width: Val::Px(428.0),
                    margin: UiRect {
                        top: Val::Px(4.0),
                        bottom: Val::Px(2.0),
                        ..default()
                    },
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));

            // Player Main Inventory (3 rows x 9 columns)
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(9, 44.0),
                    grid_template_rows: RepeatedGridTrack::px(3, 44.0),
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
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
                            FurnaceModalInvBtn(loc),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                FurnaceModalInvIcon(loc),
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
                                FurnaceModalInvCount(loc),
                            ));
                        });
                    }
                });

            // Player Hotbar (1 row x 9 columns)
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(9, 44.0),
                    grid_template_rows: RepeatedGridTrack::px(1, 44.0),
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
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
                            FurnaceModalInvBtn(loc),
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
                                FurnaceModalInvIcon(loc),
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
                                FurnaceModalInvCount(loc),
                            ));
                        });
                    }
                });

            // Footer instructions
            parent.spawn((
                Text::new("Press [E] or [ESC] to close furnace"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
            ));
        });
}

/// Handles player clicks inside the Furnace modal window.
pub fn furnace_interaction_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut furnace_btn_query: Query<(&Interaction, &FurnaceSlotBtn), With<Button>>,
    mut inv_slot_query: Query<(&Interaction, &FurnaceModalInvBtn), With<Button>>,
    mut furnace_storage: ResMut<FurnaceStorage>,
    furnace_state: Res<FurnaceOpenState>,
    mut inventory: ResMut<Inventory>,
    sfx: Option<Res<crate::audio::SoundEffects>>,
) {
    let Some(pos) = furnace_state.open_pos else {
        return;
    };

    let left_clicked = mouse_buttons.just_pressed(MouseButton::Left);
    let right_clicked = mouse_buttons.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    let furnace = furnace_storage.get_or_create(pos);

    // Helper for stack transfers on single slots
    let handle_slot_click = |slot_ref: &mut Option<ItemStack>, inv: &mut Inventory, left: bool| {
        let prev_held = inv.held_cursor_item;
        let prev_slot = *slot_ref;

        if left {
            match (inv.held_cursor_item, *slot_ref) {
                (None, Some(s)) => {
                    inv.held_cursor_item = Some(s);
                    *slot_ref = None;
                }
                (Some(held), None) => {
                    *slot_ref = Some(held);
                    inv.held_cursor_item = None;
                }
                (Some(mut held), Some(mut s)) => {
                    if held.item == s.item {
                        let max = held.item.max_stack_size();
                        let space = max.saturating_sub(s.count);
                        let to_move = held.count.min(space);
                        s.count += to_move;
                        held.count -= to_move;
                        *slot_ref = Some(s);
                        inv.held_cursor_item = if held.count > 0 { Some(held) } else { None };
                    } else {
                        std::mem::swap(&mut inv.held_cursor_item, slot_ref);
                    }
                }
                (None, None) => {}
            }
        } else {
            match (inv.held_cursor_item, *slot_ref) {
                (None, Some(s)) => {
                    let take = s.count.div_ceil(2);
                    let remainder = s.count - take;
                    inv.held_cursor_item = Some(ItemStack::new(s.item, take));
                    *slot_ref = if remainder > 0 {
                        Some(ItemStack::new(s.item, remainder))
                    } else {
                        None
                    };
                }
                (Some(mut held), None) => {
                    *slot_ref = Some(ItemStack::new(held.item, 1));
                    if held.count <= 1 {
                        inv.held_cursor_item = None;
                    } else {
                        held.count -= 1;
                        inv.held_cursor_item = Some(held);
                    }
                }
                (Some(mut held), Some(ref mut s)) => {
                    if s.item == held.item && s.count < s.item.max_stack_size() {
                        s.count += 1;
                        if held.count <= 1 {
                            inv.held_cursor_item = None;
                        } else {
                            held.count -= 1;
                            inv.held_cursor_item = Some(held);
                        }
                    }
                }
                (None, None) => {}
            }
        }

        inv.held_cursor_item != prev_held || *slot_ref != prev_slot
    };

    // 1. Furnace processing slots (Input, Fuel, Output)
    for (interaction, btn) in &mut furnace_btn_query {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            match btn.0 {
                FurnaceSlotType::Input => {
                    let changed =
                        handle_slot_click(&mut furnace.input, &mut inventory, left_clicked);
                    if changed {
                        if let Some(ref sfx) = sfx {
                            crate::audio::play_sound(&mut commands, &sfx.click, 0.75);
                        }
                    }
                }
                FurnaceSlotType::Fuel => {
                    let changed =
                        handle_slot_click(&mut furnace.fuel, &mut inventory, left_clicked);
                    if changed {
                        if let Some(ref sfx) = sfx {
                            crate::audio::play_sound(&mut commands, &sfx.click, 0.75);
                        }
                    }
                }
                FurnaceSlotType::Output => {
                    if let Some(out) = furnace.output {
                        let mut collected = false;
                        match inventory.held_cursor_item {
                            None => {
                                inventory.held_cursor_item = Some(out);
                                furnace.output = None;
                                collected = true;
                            }
                            Some(mut held) => {
                                if held.item == out.item
                                    && held.count + out.count <= held.item.max_stack_size()
                                {
                                    held.count += out.count;
                                    inventory.held_cursor_item = Some(held);
                                    furnace.output = None;
                                    collected = true;
                                }
                            }
                        }
                        if collected {
                            if let Some(ref sfx) = sfx {
                                crate::audio::play_sound(&mut commands, &sfx.pop, 0.9);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Inventory slots inside modal
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

/// Updates furnace UI elements, flames, progress, and inventory slots.
pub fn furnace_ui_update_system(
    furnace_state: Res<FurnaceOpenState>,
    furnace_storage: Res<FurnaceStorage>,
    inventory: Res<Inventory>,
    item_icon_assets: Option<Res<ItemIconAssets>>,
    menu: Option<Res<MenuState>>,
    mut modal_query: Query<&mut Visibility, With<FurnaceModal>>,
    mut slot_icons_query: Query<(&FurnaceSlotIcon, &mut ImageNode)>,
    mut slot_counts_query: Query<(&FurnaceSlotCount, &mut Text)>,
    mut flame_text_query: Query<
        (&mut TextColor, &mut Text),
        (
            With<FurnaceFlameText>,
            Without<FurnaceSlotCount>,
            Without<FurnaceCookProgressText>,
            Without<FurnaceModalInvCount>,
        ),
    >,
    mut cook_text_query: Query<
        &mut Text,
        (
            With<FurnaceCookProgressText>,
            Without<FurnaceSlotCount>,
            Without<FurnaceFlameText>,
            Without<FurnaceModalInvCount>,
        ),
    >,
    mut inv_icons_query: Query<(&FurnaceModalInvIcon, &mut ImageNode), Without<FurnaceSlotIcon>>,
    mut inv_counts_query: Query<
        (&FurnaceModalInvCount, &mut Text),
        (
            Without<FurnaceSlotCount>,
            Without<FurnaceFlameText>,
            Without<FurnaceCookProgressText>,
        ),
    >,
) {
    let menu_open = menu.is_some_and(|m| m.is_open());
    let Ok(mut vis) = modal_query.single_mut() else {
        return;
    };

    let target_vis = if furnace_state.is_open() && !menu_open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    if *vis != target_vis {
        *vis = target_vis;
    }

    let Some(pos) = furnace_state.open_pos else {
        return;
    };

    let empty_furnace = FurnaceData::default();
    let furnace = furnace_storage.furnaces.get(&pos).unwrap_or(&empty_furnace);

    // Helper to update an icon node
    let update_icon = |image_node: &mut ImageNode, stack: Option<ItemStack>| {
        if let Some(s) = stack {
            if let Some(ref assets) = item_icon_assets {
                if let Some(handle) = assets.get(s.item) {
                    image_node.image = handle.clone();
                    image_node.color = Color::WHITE;
                    return;
                }
            }
        }
        image_node.image = bevy::image::TRANSPARENT_IMAGE_HANDLE;
        image_node.color = Color::NONE;
    };

    // 1. Update furnace slots (Input, Fuel, Output)
    for (slot_icon, mut image_node) in &mut slot_icons_query {
        let stack = match slot_icon.0 {
            FurnaceSlotType::Input => furnace.input,
            FurnaceSlotType::Fuel => furnace.fuel,
            FurnaceSlotType::Output => furnace.output,
        };
        update_icon(&mut image_node, stack);
    }

    for (slot_count, mut text) in &mut slot_counts_query {
        let stack = match slot_count.0 {
            FurnaceSlotType::Input => furnace.input,
            FurnaceSlotType::Fuel => furnace.fuel,
            FurnaceSlotType::Output => furnace.output,
        };
        *text = Text::new(stack.map_or(String::new(), |s| {
            if s.count > 1 {
                format!("{}", s.count)
            } else {
                String::new()
            }
        }));
    }

    // 2. Flame Indicator
    if let Ok((mut col, mut text)) = flame_text_query.single_mut() {
        if furnace.is_burning() {
            col.0 = Color::srgb(1.0, 0.55, 0.1); // Fiery orange
            *text = Text::new("🔥");
        } else {
            col.0 = Color::srgb(0.35, 0.35, 0.35); // Gray dormant flame
            *text = Text::new("🔥");
        }
    }

    // 3. Cook Progress
    if let Ok(mut text) = cook_text_query.single_mut() {
        let pct = (furnace.cook_ratio() * 100.0) as u32;
        *text = Text::new(if pct > 0 {
            format!("{pct}%")
        } else {
            String::new()
        });
    }

    // 4. Player Inventory inside Furnace Modal
    for (icon_ui, mut image_node) in &mut inv_icons_query {
        let item_opt = match icon_ui.0 {
            SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
            SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
        };
        update_icon(&mut image_node, item_opt);
    }

    for (count_ui, mut text) in &mut inv_counts_query {
        let item_opt = match count_ui.0 {
            SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
            SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
        };
        *text = Text::new(item_opt.map_or(String::new(), |s| {
            if s.count > 1 {
                format!("{}", s.count)
            } else {
                String::new()
            }
        }));
    }
}
