#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::PrimaryWindow;

use super::{CraftingMode, CraftingState, GRID_SIZE};
use crate::inventory::Inventory;
use crate::menu::types::Button;

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

#[derive(Component)]
pub struct CraftingTitleUi;

#[derive(Component)]
pub struct CursorFollower;

#[derive(Component)]
pub struct CursorFollowerIcon;

#[derive(Component)]
pub struct CursorFollowerCount;

/// Spawns the 3x3/2x2 crafting UI section inside the inventory modal parent.
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
        CraftingTitleUi,
    ));

    parent
        .spawn(Node {
            width: Val::Px(450.0),
            height: Val::Px(136.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|crafting_row| {
            // 3x3 Crafting Grid container (132px x 132px, 3 rows of 3 slots)
            crafting_row
                .spawn(Node {
                    width: Val::Px(132.0),
                    height: Val::Px(132.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..GRID_SIZE {
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(40.0),
                                height: Val::Px(40.0),
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
                                    width: Val::Px(22.0),
                                    height: Val::Px(22.0),
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
                                    bottom: Val::Px(1.0),
                                    right: Val::Px(3.0),
                                    ..default()
                                },
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(10.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                                CraftingCountUi(idx),
                            ));
                        });
                    }
                });

            // Arrow separator (➔)
            crafting_row.spawn((
                Text::new("➔"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                Node {
                    margin: UiRect::axes(Val::Px(20.0), Val::Px(0.0)),
                    ..default()
                },
            ));

            // Result slot (larger with gold border)
            crafting_row
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(54.0),
                        height: Val::Px(54.0),
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
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 0.3)),
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
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            Visibility::Hidden,
            CursorFollower,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(22.0),
                    height: Val::Px(22.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                CursorFollowerIcon,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(2.0),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.2)),
                CursorFollowerCount,
            ));
        });
}

/// Updates colors, counts, title and slot visibility for crafting UI.
pub fn crafting_ui_update_system(
    crafting_state: Res<CraftingState>,
    inventory: Res<Inventory>,
    mut title_query: Query<
        &mut Text,
        (
            With<CraftingTitleUi>,
            Without<CraftingCountUi>,
            Without<CraftingResultCountUi>,
        ),
    >,
    mut slot_nodes_query: Query<
        (&CraftingSlotUi, &mut Visibility, &mut Node),
        (Without<CursorFollower>, Without<CraftingIconUi>),
    >,
    mut slot_icons_query: Query<
        (&CraftingIconUi, &mut BackgroundColor),
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
    mut result_icon_query: Query<&mut BackgroundColor, With<CraftingResultIconUi>>,
    mut result_count_query: Query<&mut Text, With<CraftingResultCountUi>>,
) {
    if !inventory.is_open && !crafting_state.is_table_open {
        return;
    }

    // 1. Update Title Text
    for mut title in &mut title_query {
        *title = Text::new(match crafting_state.mode {
            CraftingMode::Personal2x2 => "Crafting (2x2)",
            CraftingMode::Table3x3 => "Crafting Table (3x3)",
        });
    }

    // 2. Update Slots
    let is_2x2 = crafting_state.mode == CraftingMode::Personal2x2;
    for (slot_ui, mut vis, mut node) in &mut slot_nodes_query {
        let idx = slot_ui.0;
        // In 2x2 mode, only slots (0, 1) and (3, 4) in the 3x3 grid are active
        let is_active_slot = if is_2x2 {
            idx == 0 || idx == 1 || idx == 3 || idx == 4
        } else {
            idx < GRID_SIZE
        };

        let target_vis = if is_active_slot {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        if *vis != target_vis {
            *vis = target_vis;
        }

        // Adjust dimensions: slightly larger when in 2x2 mode
        if is_2x2 && is_active_slot {
            node.width = Val::Px(58.0);
            node.height = Val::Px(58.0);
        } else {
            node.width = Val::Px(40.0);
            node.height = Val::Px(40.0);
        }
    }

    // 3. Update Slot Icons
    for (icon_ui, mut bg) in &mut slot_icons_query {
        let idx = icon_ui.0;
        if let Some(stack) = crafting_state.grid.get(idx).copied().flatten() {
            let col = stack.item.color();
            bg.0 = Color::srgb(col[0], col[1], col[2]);
        } else {
            bg.0 = Color::NONE;
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
    if let Ok(mut bg) = result_icon_query.single_mut() {
        if let Some(output) = crafting_state.output {
            let col = output.item.color();
            bg.0 = Color::srgb(col[0], col[1], col[2]);
        } else {
            bg.0 = Color::NONE;
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
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut follower_query: Query<(&mut Node, &mut Visibility), With<CursorFollower>>,
    mut icon_query: Query<&mut BackgroundColor, With<CursorFollowerIcon>>,
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

            if let Ok(mut bg) = icon_query.single_mut() {
                let col = held.item.color();
                bg.0 = Color::srgb(col[0], col[1], col[2]);
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
