#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::PrimaryWindow;

use super::{CraftingMode, CraftingState};
use crate::inventory::Inventory;
use crate::item_icon::ItemIconAssets;
use crate::menu::types::Button;

// UI Marker Components
#[derive(Component)]
pub struct CraftingSlotBtn(pub usize);

#[derive(Component)]
pub struct CraftingSlotUi(pub usize);

#[derive(Component)]
pub struct CraftingRowUi(pub usize);

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

#[derive(Component)]
pub struct CraftingTitleUi;

#[derive(Component)]
pub struct CursorFollower;

#[derive(Component)]
pub struct CursorFollowerIcon;

#[derive(Component)]
pub struct CursorFollowerCount;

/// Helper to spawn an individual crafting slot button with Minecraft inset bevel.
fn spawn_crafting_slot_btn(parent: &mut ChildSpawnerCommands, idx: usize) {
    parent
        .spawn((
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
            BorderColor {
                top: Color::srgb(0.22, 0.22, 0.22),
                left: Color::srgb(0.22, 0.22, 0.22),
                right: Color::srgb(1.0, 1.0, 1.0),
                bottom: Color::srgb(1.0, 1.0, 1.0),
            },
            CraftingSlotBtn(idx),
            CraftingSlotUi(idx),
        ))
        .with_children(|btn| {
            btn.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ImageNode::default(),
                CraftingIconUi(idx),
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
                CraftingCountUi(idx),
            ));
        });
}

/// Spawns the 3x3/2x2 crafting UI section inside the inventory modal parent.
pub fn spawn_crafting_widget(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            width: Val::Px(428.0),
            height: Val::Px(148.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|crafting_row| {
            // Crafting Grid container: Column of 3 rows (Row 0, 1, 2)
            crafting_row
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|grid_col| {
                    // Row 0: Slots 0, 1, 2
                    grid_col
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            CraftingRowUi(0),
                        ))
                        .with_children(|row| {
                            spawn_crafting_slot_btn(row, 0);
                            spawn_crafting_slot_btn(row, 1);
                            spawn_crafting_slot_btn(row, 2);
                        });

                    // Row 1: Slots 3, 4, 5
                    grid_col
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            CraftingRowUi(1),
                        ))
                        .with_children(|row| {
                            spawn_crafting_slot_btn(row, 3);
                            spawn_crafting_slot_btn(row, 4);
                            spawn_crafting_slot_btn(row, 5);
                        });

                    // Row 2: Slots 6, 7, 8
                    grid_col
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            CraftingRowUi(2),
                        ))
                        .with_children(|row| {
                            spawn_crafting_slot_btn(row, 6);
                            spawn_crafting_slot_btn(row, 7);
                            spawn_crafting_slot_btn(row, 8);
                        });
                });

            // Minecraft Crafting Arrow
            crafting_row.spawn((
                Text::new("➔"),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
                Node {
                    margin: UiRect::axes(Val::Px(24.0), Val::Px(0.0)),
                    ..default()
                },
            ));

            // Result slot (48px x 48px, sunken 3D slot with 36px icon)
            crafting_row
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(48.0),
                        height: Val::Px(48.0),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.545, 0.545, 0.545)),
                    BorderColor {
                        top: Color::srgb(0.22, 0.22, 0.22),
                        left: Color::srgb(0.22, 0.22, 0.22),
                        right: Color::srgb(1.0, 1.0, 1.0),
                        bottom: Color::srgb(1.0, 1.0, 1.0),
                    },
                    CraftingResultBtn,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Node {
                            width: Val::Px(36.0),
                            height: Val::Px(36.0),
                            ..default()
                        },
                        ImageNode::default(),
                        CraftingResultIconUi,
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
                        CraftingResultCountUi,
                    ));
                });
        });
}

/// Spawns the floating cursor follower root for dragging item stacks.
pub fn spawn_cursor_follower(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-100.0),
                top: Val::Px(-100.0),
                width: Val::Px(36.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            CursorFollower,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                ImageNode::default(),
                CursorFollowerIcon,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    right: Val::Px(2.0),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                CursorFollowerCount,
            ));
        });
}

/// Updates colors, counts, title and slot visibility for crafting UI.
pub fn crafting_ui_update_system(
    crafting_state: Res<CraftingState>,
    inventory: Res<Inventory>,
    item_icon_assets: Option<Res<ItemIconAssets>>,
    mut title_query: Query<
        &mut Text,
        (
            With<CraftingTitleUi>,
            Without<CraftingCountUi>,
            Without<CraftingResultCountUi>,
        ),
    >,
    mut rows_query: Query<(&CraftingRowUi, &mut Node), Without<CraftingSlotUi>>,
    mut slot_nodes_query: Query<
        (&CraftingSlotUi, &mut Node),
        (
            Without<CraftingRowUi>,
            Without<CursorFollower>,
            Without<CraftingIconUi>,
        ),
    >,
    mut slot_icons_query: Query<
        (&CraftingIconUi, &mut ImageNode),
        (Without<CraftingResultIconUi>, Without<CursorFollowerIcon>),
    >,
    mut slot_counts_query: Query<
        (&CraftingCountUi, &mut Text),
        (
            Without<CraftingResultCountUi>,
            Without<CraftingTitleUi>,
            Without<CursorFollowerCount>,
        ),
    >,
    mut result_icon_query: Query<&mut ImageNode, With<CraftingResultIconUi>>,
    mut result_count_query: Query<&mut Text, With<CraftingResultCountUi>>,
) {
    if !inventory.is_open && !crafting_state.is_table_open {
        return;
    }

    // 1. Update Title Text
    for mut title in &mut title_query {
        *title = Text::new(match crafting_state.mode {
            CraftingMode::Personal2x2 => "Crafting",
            CraftingMode::Table3x3 => "Crafting Table",
        });
    }

    // 2. Update Row & Slot Display (Flex vs None)
    let is_2x2 = crafting_state.mode == CraftingMode::Personal2x2;

    // Toggle 3rd row visibility
    for (row_ui, mut node) in &mut rows_query {
        if row_ui.0 == 2 {
            let target_display = if is_2x2 { Display::None } else { Display::Flex };
            if node.display != target_display {
                node.display = target_display;
            }
        }
    }

    // Toggle rightmost slot visibility in rows 0 and 1 (slots 2 and 5)
    for (slot_ui, mut node) in &mut slot_nodes_query {
        let idx = slot_ui.0;
        if idx == 2 || idx == 5 {
            let target_display = if is_2x2 { Display::None } else { Display::Flex };
            if node.display != target_display {
                node.display = target_display;
            }
        }
    }

    // 3. Update Slot Icons
    for (icon_ui, mut image_node) in &mut slot_icons_query {
        let idx = icon_ui.0;
        if let Some(stack) = crafting_state.grid.get(idx).copied().flatten() {
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

    // 4. Update Slot Counts
    for (count_ui, mut text) in &mut slot_counts_query {
        let idx = count_ui.0;
        if let Some(stack) = crafting_state.grid.get(idx).copied().flatten() {
            *text = Text::new(if stack.count > 1 {
                format!("{}", stack.count)
            } else {
                String::new()
            });
        } else {
            *text = Text::new("");
        }
    }

    // 5. Update Result Slot
    if let Ok(mut image_node) = result_icon_query.single_mut() {
        if let Some(output) = crafting_state.output {
            if let Some(ref assets) = item_icon_assets {
                if let Some(handle) = assets.get(output.item) {
                    image_node.image = handle.clone();
                    image_node.color = Color::WHITE;
                }
            }
        } else {
            image_node.image = bevy::image::TRANSPARENT_IMAGE_HANDLE;
            image_node.color = Color::NONE;
        }
    }

    if let Ok(mut text) = result_count_query.single_mut() {
        if let Some(output) = crafting_state.output {
            *text = Text::new(if output.count > 1 {
                format!("{}", output.count)
            } else {
                String::new()
            });
        } else {
            *text = Text::new("");
        }
    }
}

/// Positions and renders the floating item stack following the cursor when dragging.
pub fn cursor_follower_update_system(
    inventory: Res<Inventory>,
    item_icon_assets: Option<Res<ItemIconAssets>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut follower_query: Query<(&mut Node, &mut Visibility), With<CursorFollower>>,
    mut icon_query: Query<&mut ImageNode, With<CursorFollowerIcon>>,
    mut count_query: Query<&mut Text, With<CursorFollowerCount>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Ok((mut node, mut vis)) = follower_query.single_mut() else {
        return;
    };

    if !inventory.is_open && inventory.held_cursor_item.is_none() {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        return;
    }

    if let Some(held) = inventory.held_cursor_item {
        if let Some(cursor_pos) = window.cursor_position() {
            node.left = Val::Px(cursor_pos.x + 8.0);
            node.top = Val::Px(cursor_pos.y + 8.0);
            if *vis != Visibility::Visible {
                *vis = Visibility::Visible;
            }

            if let Ok(mut image_node) = icon_query.single_mut() {
                if let Some(ref assets) = item_icon_assets {
                    if let Some(handle) = assets.get(held.item) {
                        image_node.image = handle.clone();
                        image_node.color = Color::WHITE;
                    }
                }
            }

            if let Ok(mut text) = count_query.single_mut() {
                *text = Text::new(if held.count > 1 {
                    format!("{}", held.count)
                } else {
                    String::new()
                });
            }
            return;
        }
    }

    if *vis != Visibility::Hidden {
        *vis = Visibility::Hidden;
    }
}
