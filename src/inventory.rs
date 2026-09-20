use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::{BlockFace, BlockType};

pub const HOTBAR_SLOTS: usize = 9;

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

pub fn block_name(block: BlockType) -> &'static str {
    match block {
        BlockType::Air => "Aria",
        BlockType::Grass => "Erba",
        BlockType::Dirt => "Terra",
        BlockType::Stone => "Pietra",
        BlockType::Cobblestone => "Pietrisco",
        BlockType::Wood => "Legno",
        BlockType::Planks => "Assi",
        BlockType::Leaves => "Foglie",
        BlockType::Glass => "Vetro",
        BlockType::Sand => "Sabbia",
        BlockType::Sandstone => "Arenaria",
        BlockType::Gravel => "Ghiaia",
        BlockType::Water => "Acqua",
        BlockType::Ice => "Ghiaccio",
        BlockType::Snow => "Neve",
        BlockType::Cactus => "Cactus",
        BlockType::CoalOre => "Carbone",
        BlockType::IronOre => "Ferro",
        BlockType::GoldOre => "Oro",
        BlockType::DiamondOre => "Diamante",
        BlockType::Bedrock => "Bedrock",
    }
}

#[derive(Resource)]
pub struct Inventory {
    pub hotbar: [BlockType; HOTBAR_SLOTS],
    pub selected_slot: usize,
    pub is_open: bool,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            hotbar: [
                BlockType::Grass,
                BlockType::Dirt,
                BlockType::Stone,
                BlockType::Cobblestone,
                BlockType::Wood,
                BlockType::Planks,
                BlockType::Glass,
                BlockType::DiamondOre,
                BlockType::Water,
            ],
            selected_slot: 0,
            is_open: false,
        }
    }
}

impl Inventory {
    pub fn selected_block(&self) -> BlockType {
        self.hotbar[self.selected_slot]
    }
}

#[derive(Component)]
pub struct HotbarSlotUi(pub usize);

#[derive(Component)]
pub struct HotbarIconUi(pub usize);

#[derive(Component)]
pub struct InventoryModal;

#[derive(Component)]
pub struct InventorySlotBtn(pub BlockType);

pub fn setup_inventory_ui(mut commands: Commands) {
    // 1. HOTBAR HUD (In basso al centro, sempre visibile durante il gioco)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-240.0), // Centra i 480px di larghezza
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
        ))
        .with_children(|parent| {
            for i in 0..HOTBAR_SLOTS {
                let is_selected = i == 0;
                let default_block = match i {
                    0 => BlockType::Grass,
                    1 => BlockType::Dirt,
                    2 => BlockType::Stone,
                    3 => BlockType::Cobblestone,
                    4 => BlockType::Wood,
                    5 => BlockType::Planks,
                    6 => BlockType::Glass,
                    7 => BlockType::DiamondOre,
                    _ => BlockType::Water,
                };
                let col = default_block.color(BlockFace::Top);

                parent
                    .spawn((
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(if is_selected { 3.0 } else { 1.5 })),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.9)),
                        BorderColor::all(if is_selected {
                            Color::srgb(1.0, 0.85, 0.2) // Oro se selezionato
                        } else {
                            Color::srgba(0.4, 0.4, 0.45, 0.6)
                        }),
                        HotbarSlotUi(i),
                    ))
                    .with_children(|slot| {
                        // Icona colore blocco
                        slot.spawn((
                            Node {
                                width: Val::Px(26.0),
                                height: Val::Px(26.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(col[0], col[1], col[2])),
                            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                            HotbarIconUi(i),
                        ));

                        // Numero slot (1-9)
                        slot.spawn((
                            Text::new(format!("{}", i + 1)),
                            TextFont {
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });

    // 2. FINESTRA INVENTARIO COMPLETO (Aperta con tasto 'E')
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-270.0),
                    top: Val::Px(-200.0),
                    ..default()
                },
                width: Val::Px(540.0),
                height: Val::Px(400.0),
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
            // Titolo
            parent.spawn((
                Text::new("INVENTARIO - Clicca un blocco per equipaggiarlo nello slot attivo"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(14.0)),
                    ..default()
                },
            ));

            // Griglia di tutti i blocchi disponibili
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    row_gap: Val::Px(10.0),
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|grid| {
                    for &block in ALL_BLOCKS {
                        let col = block.color(BlockFace::Top);
                        let name = block_name(block);

                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(95.0),
                                height: Val::Px(64.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.5)),
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.18, 0.18, 0.24, 0.9)),
                            BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 0.7)),
                            InventorySlotBtn(block),
                        ))
                        .with_children(|btn| {
                            // Anteprima colore blocco
                            btn.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    margin: UiRect::bottom(Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(col[0], col[1], col[2])),
                                BorderColor::all(Color::BLACK),
                            ));

                            // Nome del blocco
                            btn.spawn((
                                Text::new(name),
                                TextFont {
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });

            // Istruzioni in basso
            parent.spawn((
                Text::new("Premi [E] o [ESC] per chiudere l'inventario e tornare al gioco"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.75, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
            ));
        });
}

pub fn inventory_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut inventory: ResMut<Inventory>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    // Tasto E: Apri/Chiudi Inventario
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

    // ESC chiude l'inventario se aperto
    if keys.just_pressed(KeyCode::Escape) && inventory.is_open {
        inventory.is_open = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }

    // Se l'inventario è chiuso, gestisci la selezione Hotbar (1-9 e Rotellina Mouse)
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

        // Rotellina del mouse per scorrere la hotbar
        if scroll.delta.y < -0.1 {
            inventory.selected_slot = (inventory.selected_slot + 1) % HOTBAR_SLOTS;
        } else if scroll.delta.y > 0.1 {
            inventory.selected_slot = (inventory.selected_slot + HOTBAR_SLOTS - 1) % HOTBAR_SLOTS;
        }
    }
}

pub fn inventory_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &InventorySlotBtn, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut inventory: ResMut<Inventory>,
) {
    for (interaction, btn_slot, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Equipaggia il blocco cliccato nello slot hotbar attualmente selezionato!
                let current_slot = inventory.selected_slot;
                inventory.hotbar[current_slot] = btn_slot.0;
                *border = BorderColor::all(Color::srgb(1.0, 0.9, 0.2));
            }
            Interaction::Hovered => {
                *border = BorderColor::all(Color::srgb(0.9, 0.9, 1.0));
            }
            Interaction::None => {
                *border = BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 0.7));
            }
        }
    }
}

pub fn update_inventory_ui_system(
    inventory: Res<Inventory>,
    mut modal_query: Query<&mut Visibility, With<InventoryModal>>,
    mut slots_query: Query<(&HotbarSlotUi, &mut Node, &mut BorderColor), Without<HotbarIconUi>>,
    mut icons_query: Query<(&HotbarIconUi, &mut BackgroundColor)>,
) {
    // 1. Mostra/Nascondi la finestra inventario modale
    if let Ok(mut vis) = modal_query.single_mut() {
        let target_vis = if inventory.is_open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }

    // 2. Aggiorna bordi e stile degli slot della Hotbar
    for (slot, mut node, mut border) in &mut slots_query {
        let is_selected = slot.0 == inventory.selected_slot;
        node.border = UiRect::all(Val::Px(if is_selected { 3.0 } else { 1.5 }));
        *border = if is_selected {
            BorderColor::all(Color::srgb(1.0, 0.85, 0.2)) // Bordo oro brillante
        } else {
            BorderColor::all(Color::srgba(0.4, 0.4, 0.45, 0.6))
        };
    }

    // 3. Aggiorna le icone di colore della Hotbar
    for (icon, mut bg) in &mut icons_query {
        if icon.0 < HOTBAR_SLOTS {
            let block = inventory.hotbar[icon.0];
            let col = block.color(BlockFace::Top);
            bg.0 = Color::srgb(col[0], col[1], col[2]);
        }
    }
}
