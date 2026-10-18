#![forbid(unsafe_code)]
#![allow(deprecated)]

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::text::FontSize;

use super::{HOTBAR_SLOTS, Inventory, SlotLocation};
use crate::crafting::ui::{CraftingResultBtn, CraftingSlotBtn};
use crate::crafting::{CraftingState, spawn_crafting_widget};
use crate::item_icon::ItemIconAssets;
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

#[derive(Component)]
pub struct HoveredItemTextUi;

/// Minecraft-authentic sunken 3D slot border (dark shadow top/left, white highlight bottom/right).
#[must_use]
pub fn minecraft_slot_border() -> BorderColor {
    BorderColor {
        top: Color::srgb(0.22, 0.22, 0.22),
        left: Color::srgb(0.22, 0.22, 0.22),
        right: Color::srgb(1.0, 1.0, 1.0),
        bottom: Color::srgb(1.0, 1.0, 1.0),
    }
}

/// Minecraft-authentic raised 3D window border (white highlight top/left, dark shadow bottom/right).
#[must_use]
pub fn minecraft_window_border() -> BorderColor {
    BorderColor {
        top: Color::srgb(1.0, 1.0, 1.0),
        left: Color::srgb(1.0, 1.0, 1.0),
        bottom: Color::srgb(0.33, 0.33, 0.33),
        right: Color::srgb(0.33, 0.33, 0.33),
    }
}

pub fn setup_inventory_ui(mut commands: Commands) {
    // 1. HOTBAR HUD (Bottom center, always visible during gameplay)
    // 9 slots of 44px + 8 gaps of 4px = 428px inner width, 444px container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-222.0),
                    ..default()
                },
                width: Val::Px(444.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(5.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.776, 0.776, 0.776)), // Minecraft GUI Gray #C6C6C6
            minecraft_window_border(),
            HotbarRoot,
        ))
        .with_children(|parent| {
            for idx in 0..HOTBAR_SLOTS {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(44.0),
                            height: Val::Px(44.0),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(if idx == 0 { 2.5 } else { 2.0 })),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.545, 0.545, 0.545)), // Sunken slot gray #8B8B8B
                        if idx == 0 {
                            BorderColor::all(Color::srgb(1.0, 1.0, 1.0))
                        } else {
                            minecraft_slot_border()
                        },
                        HotbarSlotUi(idx),
                    ))
                    .with_children(|slot| {
                        // Slot number indicator (top-left)
                        slot.spawn((
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

                        // Item icon (centered 32x32 ImageNode)
                        slot.spawn((
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                            ImageNode::default(),
                            HotbarIconUi(idx),
                        ));

                        // Stack count label (bottom-right)
                        slot.spawn((
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
            BackgroundColor(Color::srgb(0.776, 0.776, 0.776)), // Minecraft GUI Stone Gray #C6C6C6
            minecraft_window_border(),
            Visibility::Hidden,
            InventoryModal,
        ))
        .with_children(|parent| {
            // Header Row: Crafting Title on left, Hovered Item Name on right
            parent
                .spawn(Node {
                    width: Val::Px(428.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("Crafting"),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.25, 0.25, 0.25)),
                    ));

                    header.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.15, 0.15, 0.20)),
                        HoveredItemTextUi,
                    ));
                });

            // Embedded Crafting Widget Section (Grid, Arrow, Result Slot)
            spawn_crafting_widget(parent);

            // Main Inventory Section Label
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

            // Main 3x9 grid (27 slots): exactly 428px wide x 140px high
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
                    for idx in 0..super::MAIN_INVENTORY_SLOTS {
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
                            ModalSlotBtn(loc),
                            ModalSlotUi(loc),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                ImageNode::default(),
                                ModalIconUi(loc),
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
                                ModalCountUi(loc),
                            ));
                        });
                    }
                });

            // Hotbar 1x9 grid in modal: exactly 428px wide x 44px high, column-aligned
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
                            ModalSlotBtn(loc),
                            ModalSlotUi(loc),
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
                                ModalIconUi(loc),
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
                                ModalCountUi(loc),
                            ));
                        });
                    }
                });

            // Footer instructions
            parent.spawn((
                Text::new("Press [E] or [ESC] to return to game"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
            ));
        });
}

#[derive(SystemParam)]
pub struct InventoryStateContext<'w> {
    pub inventory: Res<'w, Inventory>,
    pub crafting_state: Option<Res<'w, CraftingState>>,
    pub chest_state: Option<Res<'w, crate::chest::ChestOpenState>>,
    pub furnace_state: Option<Res<'w, crate::furnace::FurnaceOpenState>>,
    pub item_icon_assets: Option<Res<'w, ItemIconAssets>>,
    pub menu: Option<Res<'w, MenuState>>,
}

pub fn update_inventory_ui_system(
    ctx: InventoryStateContext,
    mut modal_query: Query<&mut Visibility, With<InventoryModal>>,
    mut hotbar_query: Query<&mut Visibility, (With<HotbarRoot>, Without<InventoryModal>)>,
    mut hud_slots_query: Query<(&HotbarSlotUi, &mut Node, &mut BorderColor), Without<ModalSlotUi>>,
    mut hud_icons_query: Query<(&HotbarIconUi, &mut ImageNode), Without<ModalIconUi>>,
    mut hud_counts_query: Query<(&HotbarCountUi, &mut Text), Without<ModalCountUi>>,
    mut modal_slots_query: Query<
        (&ModalSlotUi, &mut BorderColor, &mut BackgroundColor),
        Without<HotbarSlotUi>,
    >,
    mut modal_icons_query: Query<(&ModalIconUi, &mut ImageNode), Without<HotbarIconUi>>,
    mut modal_counts_query: Query<(&ModalCountUi, &mut Text), Without<HotbarCountUi>>,
    modal_btn_query: Query<(&Interaction, &ModalSlotBtn)>,
    crafting_slot_btn_query: Query<(&Interaction, &CraftingSlotBtn)>,
    crafting_result_btn_query: Query<&Interaction, With<CraftingResultBtn>>,
    mut hovered_text_query: Query<
        &mut Text,
        (
            With<HoveredItemTextUi>,
            Without<ModalCountUi>,
            Without<HotbarCountUi>,
        ),
    >,
) {
    let inventory = &ctx.inventory;
    let crafting_state = ctx.crafting_state.as_ref();
    let chest_state = ctx.chest_state.as_ref();
    let furnace_state = ctx.furnace_state.as_ref();
    let item_icon_assets = ctx.item_icon_assets.as_ref();
    let menu = ctx.menu.as_ref();

    let menu_open = menu.is_some_and(|m| m.is_open());
    let chest_open = chest_state.as_ref().is_some_and(|c| c.is_open());
    let furnace_open = furnace_state.as_ref().is_some_and(|f| f.is_open());

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

    // 1. Show/Hide inventory modal window (suppressed when chest or furnace modal is open)
    if let Ok(mut vis) = modal_query.single_mut() {
        let target_vis = if inventory.is_open && !menu_open && !chest_open && !furnace_open {
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
        node.border = UiRect::all(Val::Px(if is_selected { 2.5 } else { 2.0 }));
        *border = if is_selected {
            BorderColor::all(Color::srgb(1.0, 1.0, 1.0))
        } else {
            minecraft_slot_border()
        };
    }

    // 3. Update HUD hotbar icons
    for (icon, mut image_node) in &mut hud_icons_query {
        if icon.0 < HOTBAR_SLOTS {
            if let Some(stack) = inventory.hotbar[icon.0] {
                if let Some(assets) = item_icon_assets {
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
    }

    // 4. Update HUD hotbar counts
    for (count_ui, mut text) in &mut hud_counts_query {
        if count_ui.0 < HOTBAR_SLOTS {
            if let Some(stack) = inventory.hotbar[count_ui.0] {
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

    // 5. If modal is open, update modal slots and hovered tooltip
    if inventory.is_open && !menu_open {
        // Compute hovered item name from any slot hovered
        let mut hovered_name = String::new();

        for (interaction, btn) in &modal_btn_query {
            if *interaction == Interaction::Hovered {
                let item_opt = match btn.0 {
                    SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
                    SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
                };
                if let Some(stack) = item_opt {
                    hovered_name = stack.item.name();
                    break;
                }
            }
        }

        if hovered_name.is_empty() {
            if let Some(cs) = crafting_state {
                for (interaction, btn) in &crafting_slot_btn_query {
                    if *interaction == Interaction::Hovered {
                        if let Some(stack) = cs.grid.get(btn.0).copied().flatten() {
                            hovered_name = stack.item.name();
                            break;
                        }
                    }
                }

                if hovered_name.is_empty() {
                    for interaction in &crafting_result_btn_query {
                        if *interaction == Interaction::Hovered {
                            if let Some(output) = cs.output {
                                hovered_name = output.item.name();
                                break;
                            }
                        }
                    }
                }
            }
        }

        for mut text in &mut hovered_text_query {
            if text.as_str() != hovered_name {
                *text = Text::new(hovered_name.clone());
            }
        }

        // Modal slot borders & hover highlight
        for (slot_ui, mut border, mut bg) in &mut modal_slots_query {
            let is_selected_hotbar = match slot_ui.0 {
                SlotLocation::Hotbar(idx) => idx == inventory.selected_slot,
                SlotLocation::Main(_) => false,
            };

            *border = if is_selected_hotbar {
                BorderColor::all(Color::srgb(1.0, 1.0, 1.0))
            } else {
                minecraft_slot_border()
            };

            // Check if this slot button is hovered
            let is_hovered = modal_btn_query.iter().any(|(inter, btn)| {
                btn.0 == slot_ui.0
                    && (*inter == Interaction::Hovered || *inter == Interaction::Pressed)
            });

            bg.0 = if is_hovered {
                Color::srgb(0.65, 0.65, 0.65) // Light highlight on hover
            } else {
                Color::srgb(0.545, 0.545, 0.545)
            };
        }

        // Modal icons
        for (icon_ui, mut image_node) in &mut modal_icons_query {
            let item_opt = match icon_ui.0 {
                SlotLocation::Hotbar(idx) => inventory.hotbar.get(idx).copied().flatten(),
                SlotLocation::Main(idx) => inventory.main.get(idx).copied().flatten(),
            };
            if let Some(stack) = item_opt {
                if let Some(assets) = item_icon_assets {
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

        // Modal counts
        for (count_ui, mut text) in &mut modal_counts_query {
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
}
