#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;

use super::{HOTBAR_SLOTS, Inventory, SlotLocation};
use crate::crafting::spawn_crafting_widget;
use crate::menu::MenuState;
use crate::menu::types::Button;

#[derive(Component)]
pub struct HotbarSlotUi(pub usize);

#[derive(Component)]
pub struct HotbarIconUi(pub usize);

#[derive(Component)]
pub struct HotbarCountUi(pub usize);

#[derive(Component)]
pub struct InventoryModal;

#[derive(Component)]
pub struct HotbarRoot;

#[derive(Component)]
pub struct ModalSlotBtn(pub SlotLocation);

#[derive(Component)]
pub struct ModalSlotUi(pub SlotLocation);

#[derive(Component)]
pub struct ModalIconUi(pub SlotLocation);

#[derive(Component)]
pub struct ModalCountUi(pub SlotLocation);

pub fn setup_inventory_ui(mut commands: Commands) {
    // 1. HOTBAR HUD (Bottom center, always visible during gameplay)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-240.0), // Center the 480px width
                    ..default()
                },
                width: Val::Px(480.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.10, 0.85)),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.35, 0.9)),
            HotbarRoot,
        ))
        .with_children(|parent| {
            for idx in 0..HOTBAR_SLOTS {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(if idx == 0 { 3.0 } else { 1.5 })),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.20, 0.8)),
                        BorderColor::all(if idx == 0 {
                            Color::srgb(1.0, 0.85, 0.2)
                        } else {
                            Color::srgba(0.4, 0.4, 0.45, 0.6)
                        }),
                        HotbarSlotUi(idx),
                    ))
                    .with_children(|slot| {
                        // Slot number indicator (top-left)
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                            Text::new(format!("{}", idx + 1)),
                            TextFont {
                                font_size: FontSize::Px(10.0),
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.85, 0.8)),
                        ));

                        // Item icon color swatch (centered)
                        slot.spawn((
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                            HotbarIconUi(idx),
                        ));

                        // Stack count label (bottom-right)
                        slot.spawn((
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
                            HotbarCountUi(idx),
                        ));
                    });
            }
        });

    // 2. INVENTORY MODAL (Full 27-slot Main + 9-slot Hotbar overlay + Crafting Grid)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-270.0),
                    top: Val::Px(-300.0),
                    ..default()
                },
                width: Val::Px(540.0),
                height: Val::Px(600.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.13, 0.96)),
            BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 0.9)),
            Visibility::Hidden,
            InventoryModal,
        ))
        .with_children(|parent| {
            // Header title
            parent.spawn((
                Text::new("Inventory & Crafting"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.95)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Embedded Crafting Widget Section
            spawn_crafting_widget(parent);

            // Main Inventory Section Label
            parent.spawn((
                Text::new("Main Inventory"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.75, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
            ));

            // Main 3x9 grid (27 slots)
            parent
                .spawn(Node {
                    width: Val::Px(480.0),
                    height: Val::Px(156.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    align_content: AlignContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..super::MAIN_INVENTORY_SLOTS {
                        let loc = SlotLocation::Main(idx);
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
                            ModalSlotBtn(loc),
                            ModalSlotUi(loc),
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
                                ModalIconUi(loc),
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
                                ModalCountUi(loc),
                            ));
                        });
                    }
                });

            // Hotbar Mirror Section Label
            parent.spawn((
                Text::new("Hotbar (Quick Access)"),
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

            // Hotbar 1x9 grid in modal
            parent
                .spawn(Node {
                    width: Val::Px(480.0),
                    height: Val::Px(52.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..HOTBAR_SLOTS {
                        let loc = SlotLocation::Hotbar(idx);
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(if idx == 0 { 2.5 } else { 1.5 })),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.20, 0.20, 0.26, 0.95)),
                            BorderColor::all(if idx == 0 {
                                Color::srgb(1.0, 0.85, 0.2)
                            } else {
                                Color::srgba(0.4, 0.4, 0.5, 0.6)
                            }),
                            ModalSlotBtn(loc),
                            ModalSlotUi(loc),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(2.0),
                                    left: Val::Px(4.0),
                                    ..default()
                                },
                                Text::new(format!("{}", idx + 1)),
                                TextFont {
                                    font_size: FontSize::Px(10.0),
                                    ..default()
                                },
                                TextColor(Color::srgba(0.8, 0.8, 0.85, 0.8)),
                            ));
                            btn.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                                BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                                ModalIconUi(loc),
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
                                ModalCountUi(loc),
                            ));
                        });
                    }
                });

            // Footer instructions
            parent.spawn((
                Text::new("Press [E] or [ESC] to close inventory and return to game"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.75, 0.9)),
            ));
        });
}

pub fn update_inventory_ui_system(
    inventory: Res<Inventory>,
    menu: Option<Res<MenuState>>,
    mut modal_query: Query<&mut Visibility, With<InventoryModal>>,
    mut hotbar_query: Query<&mut Visibility, (With<HotbarRoot>, Without<InventoryModal>)>,
    mut hud_slots_query: Query<(&HotbarSlotUi, &mut Node, &mut BorderColor), Without<ModalSlotUi>>,
    mut hud_icons_query: Query<(&HotbarIconUi, &mut BackgroundColor), Without<ModalIconUi>>,
    mut hud_counts_query: Query<(&HotbarCountUi, &mut Text), Without<ModalCountUi>>,
    mut modal_slots_query: Query<(&ModalSlotUi, &mut BorderColor), Without<HotbarSlotUi>>,
    mut modal_icons_query: Query<(&ModalIconUi, &mut BackgroundColor), Without<HotbarIconUi>>,
    mut modal_counts_query: Query<(&ModalCountUi, &mut Text), Without<HotbarCountUi>>,
) {
    let menu_open = menu.is_some_and(|m| m.is_open());

    // 0. Show/Hide hotbar HUD based on menu state
    if let Ok(mut vis) = hotbar_query.single_mut() {
        let target_vis = if menu_open {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }

    // 1. Show/Hide inventory modal window
    if let Ok(mut vis) = modal_query.single_mut() {
        let target_vis = if inventory.is_open && !menu_open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }

    // 2. Update HUD hotbar slot borders
    for (slot, mut node, mut border) in &mut hud_slots_query {
        let is_selected = slot.0 == inventory.selected_slot;
        node.border = UiRect::all(Val::Px(if is_selected { 3.0 } else { 1.5 }));
        *border = if is_selected {
            BorderColor::all(Color::srgb(1.0, 0.85, 0.2)) // Bright gold
        } else {
            BorderColor::all(Color::srgba(0.4, 0.4, 0.45, 0.6))
        };
    }

    // 3. Update HUD hotbar icons
    for (icon, mut bg) in &mut hud_icons_query {
        if icon.0 < HOTBAR_SLOTS {
            if let Some(stack) = inventory.hotbar[icon.0] {
                let col = stack.item.color();
                bg.0 = Color::srgb(col[0], col[1], col[2]);
            } else {
                bg.0 = Color::NONE;
            }
        }
    }

    // 4. Update HUD hotbar counts
    for (count_ui, mut text) in &mut hud_counts_query {
        if count_ui.0 < HOTBAR_SLOTS {
            if let Some(stack) = inventory.hotbar[count_ui.0] {
                *text = Text::new(format!("{}", stack.count));
            } else {
                *text = Text::new("");
            }
        }
    }

    // 5. If modal is open or inventory changed, update modal slots
    if inventory.is_open && !menu_open {
        // Modal slot borders (highlight selected hotbar slot)
        for (slot_ui, mut border) in &mut modal_slots_query {
            match slot_ui.0 {
                SlotLocation::Hotbar(idx) => {
                    let is_selected = idx == inventory.selected_slot;
                    *border = if is_selected {
                        BorderColor::all(Color::srgb(1.0, 0.85, 0.2))
                    } else {
                        BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6))
                    };
                }
                SlotLocation::Main(_) => {
                    *border = BorderColor::all(Color::srgba(0.4, 0.4, 0.5, 0.6));
                }
            }
        }

        // Modal icons
        for (icon_ui, mut bg) in &mut modal_icons_query {
            let item_opt = match icon_ui.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
                SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
            };
            if let Some(stack) = item_opt {
                let col = stack.item.color();
                bg.0 = Color::srgb(col[0], col[1], col[2]);
            } else {
                bg.0 = Color::NONE;
            }
        }

        // Modal counts
        for (count_ui, mut text) in &mut modal_counts_query {
            let item_opt = match count_ui.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
                SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
            };
            if let Some(stack) = item_opt {
                *text = Text::new(format!("{}", stack.count));
            } else {
                *text = Text::new("");
            }
        }
    }
}
