use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::{BlockFace, BlockType};
use crate::menu::MenuState;

pub const HOTBAR_SLOTS: usize = 9;
pub const MAIN_INVENTORY_SLOTS: usize = 27;
pub const MAX_STACK_SIZE: u32 = 64;

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
    }
}

/// Represents a stack of blocks of a specific type with a quantity count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ItemStack {
    pub block_type: BlockType,
    pub count: u32,
}

impl ItemStack {
    #[inline]
    pub fn new(block_type: BlockType, count: u32) -> Self {
        Self {
            block_type,
            count: count.min(MAX_STACK_SIZE),
        }
    }
}

/// Player inventory state with an empty starting hotbar and main inventory.
#[derive(Resource, Debug, Clone)]
pub struct Inventory {
    pub hotbar: [Option<ItemStack>; HOTBAR_SLOTS],
    pub main: [Option<ItemStack>; MAIN_INVENTORY_SLOTS],
    pub selected_slot: usize,
    pub is_open: bool,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            hotbar: [None; HOTBAR_SLOTS],
            main: [None; MAIN_INVENTORY_SLOTS],
            selected_slot: 0,
            is_open: false,
        }
    }
}

impl Inventory {
    /// Returns the item stack in the currently selected hotbar slot, if any.
    #[inline]
    pub fn selected_item(&self) -> Option<ItemStack> {
        self.hotbar[self.selected_slot]
    }

    /// Returns the block type in the currently selected hotbar slot, or Air if empty.
    #[inline]
    pub fn selected_block(&self) -> BlockType {
        self.hotbar[self.selected_slot].map_or(BlockType::Air, |s| s.block_type)
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

    /// Adds items of a given block type into the inventory.
    ///
    /// 1. Merges into existing non-full stacks (hotbar first, then main).
    /// 2. Fills empty slots (hotbar first, then main).
    ///
    /// Returns 0 if all items were added, or the remaining count if inventory is full.
    pub fn add_item(&mut self, block: BlockType, mut amount: u32) -> u32 {
        if block == BlockType::Air || amount == 0 {
            return amount;
        }

        // 1. Merge into existing matching stacks in hotbar
        for slot in self.hotbar.iter_mut().flatten() {
            if slot.block_type == block && slot.count < MAX_STACK_SIZE {
                let space = MAX_STACK_SIZE - slot.count;
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
            if slot.block_type == block && slot.count < MAX_STACK_SIZE {
                let space = MAX_STACK_SIZE - slot.count;
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
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some(ItemStack::new(block, to_add));
                amount -= to_add;
                if amount == 0 {
                    return 0;
                }
            }
        }

        // 4. Place into first empty slot in main inventory
        for slot in &mut self.main {
            if slot.is_none() {
                let to_add = amount.min(MAX_STACK_SIZE);
                *slot = Some(ItemStack::new(block, to_add));
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
            for i in 0..HOTBAR_SLOTS {
                let is_selected = i == 0;
                parent
                    .spawn((
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(if is_selected { 3.0 } else { 1.5 })),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.9)),
                        BorderColor::all(if is_selected {
                            Color::srgb(1.0, 0.85, 0.2) // Gold when selected
                        } else {
                            Color::srgba(0.4, 0.4, 0.45, 0.6)
                        }),
                        HotbarSlotUi(i),
                    ))
                    .with_children(|slot| {
                        // Slot number (1-9) at top-left
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                            Text::new(format!("{}", i + 1)),
                            TextFont {
                                font_size: FontSize::Px(10.0),
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.85, 0.8)),
                        ));

                        // Block color icon in center (empty at start)
                        slot.spawn((
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                            HotbarIconUi(i),
                        ));

                        // Count badge at bottom-right
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
                            HotbarCountUi(i),
                        ));
                    });
            }
        });

    // 2. FULL INVENTORY MODAL (Toggled with 'E' key)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-270.0),
                    top: Val::Px(-230.0),
                    ..default()
                },
                width: Val::Px(540.0),
                height: Val::Px(460.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
            BorderColor::all(Color::srgb(0.8, 0.7, 0.2)),
            Visibility::Hidden,
            InventoryModal,
        ))
        .with_children(|parent| {
            // Header title
            parent.spawn((
                Text::new("SURVIVAL INVENTORY"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Mine blocks to collect resources. Click hotbar to select, then click main to swap."),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.75, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Main Inventory Section Label
            parent.spawn((
                Text::new("Main Storage (3x9)"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
            ));

            // Grid of 27 Main Inventory Slots (3 rows of 9)
            parent
                .spawn(Node {
                    width: Val::Px(450.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceBetween,
                    row_gap: Val::Px(4.0),
                    column_gap: Val::Px(4.0),
                    margin: UiRect::bottom(Val::Px(14.0)),
                    ..default()
                })
                .with_children(|grid| {
                    for idx in 0..MAIN_INVENTORY_SLOTS {
                        let loc = SlotLocation::Main(idx);
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(46.0),
                                height: Val::Px(46.0),
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

            // Hotbar Section Label
            parent.spawn((
                Text::new("Hotbar (Quick Access 1-9)"),
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

            // Row of 9 Hotbar Slots in Modal
            parent
                .spawn(Node {
                    width: Val::Px(450.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                })
                .with_children(|row| {
                    for idx in 0..HOTBAR_SLOTS {
                        let loc = SlotLocation::Hotbar(idx);
                        let is_selected = idx == 0;
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(46.0),
                                height: Val::Px(46.0),
                                position_type: PositionType::Relative,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(if is_selected { 2.5 } else { 1.5 })),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 0.9)),
                            BorderColor::all(if is_selected {
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

pub fn inventory_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut inventory: ResMut<Inventory>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    menu: Option<Res<MenuState>>,
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
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        } else {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    }

    // ESC closes inventory if currently open
    if keys.just_pressed(KeyCode::Escape) && inventory.is_open {
        inventory.is_open = false;
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
    mut interaction_query: Query<
        (&Interaction, &ModalSlotBtn, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut inventory: ResMut<Inventory>,
) {
    for (interaction, btn_slot, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match btn_slot.0 {
                    SlotLocation::Hotbar(idx) => {
                        if inventory.selected_slot == idx {
                            // If clicking the active slot, quick-transfer item to first empty main slot
                            if let Some(stack) = inventory.hotbar[idx] {
                                for main_slot in &mut inventory.main {
                                    if main_slot.is_none() {
                                        *main_slot = Some(stack);
                                        inventory.hotbar[idx] = None;
                                        break;
                                    }
                                }
                            }
                        } else {
                            inventory.selected_slot = idx;
                        }
                    }
                    SlotLocation::Main(idx) => {
                        inventory.swap_main_with_active_hotbar(idx);
                    }
                }
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
                let col = stack.block_type.color(BlockFace::Top);
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
                let col = stack.block_type.color(BlockFace::Top);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_starts_completely_empty() {
        let inv = Inventory::default();
        for slot in &inv.hotbar {
            assert!(slot.is_none(), "Hotbar slot must be None at start");
        }
        for slot in &inv.main {
            assert!(slot.is_none(), "Main inventory slot must be None at start");
        }
        assert_eq!(inv.selected_slot, 0);
        assert!(!inv.is_open);
        assert_eq!(inv.selected_block(), BlockType::Air);
        assert!(inv.selected_item().is_none());
    }

    #[test]
    fn test_inventory_add_item_fills_hotbar_then_main() {
        let mut inv = Inventory::default();

        // Add 1 Wood -> should go into hotbar[0]
        let leftover = inv.add_item(BlockType::Wood, 1);
        assert_eq!(leftover, 0);
        assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Wood, 1)));

        // Add 10 Dirt -> should go into hotbar[1]
        let leftover = inv.add_item(BlockType::Dirt, 10);
        assert_eq!(leftover, 0);
        assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockType::Dirt, 10)));
    }

    #[test]
    fn test_inventory_add_item_merges_matching_stacks() {
        let mut inv = Inventory::default();

        // Fill hotbar[0] with 50 Dirt
        inv.add_item(BlockType::Dirt, 50);
        assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Dirt, 50)));

        // Add 20 more Dirt -> should top off hotbar[0] to 64 and spill 6 into hotbar[1]
        let leftover = inv.add_item(BlockType::Dirt, 20);
        assert_eq!(leftover, 0);
        assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockType::Dirt, 64)));
        assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockType::Dirt, 6)));
    }

    #[test]
    fn test_inventory_consume_selected() {
        let mut inv = Inventory::default();
        inv.hotbar[0] = Some(ItemStack::new(BlockType::Cobblestone, 2));

        assert!(inv.consume_selected(1));
        assert_eq!(
            inv.hotbar[0],
            Some(ItemStack::new(BlockType::Cobblestone, 1))
        );

        assert!(inv.consume_selected(1));
        assert_eq!(inv.hotbar[0], None);

        // Consuming from empty slot returns false
        assert!(!inv.consume_selected(1));
    }

    #[test]
    fn test_inventory_swap_main_with_hotbar() {
        let mut inv = Inventory::default();
        inv.hotbar[0] = Some(ItemStack::new(BlockType::Wood, 5));
        inv.main[3] = Some(ItemStack::new(BlockType::DiamondOre, 12));

        inv.selected_slot = 0;
        inv.swap_main_with_active_hotbar(3);

        assert_eq!(
            inv.hotbar[0],
            Some(ItemStack::new(BlockType::DiamondOre, 12))
        );
        assert_eq!(inv.main[3], Some(ItemStack::new(BlockType::Wood, 5)));
    }
}
