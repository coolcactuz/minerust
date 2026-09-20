use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, WindowMode,
};

use crate::inventory::Inventory;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuScreen {
    Main,
    Settings,
    DevSettings,
    Pause,
    None, // In-game gameplay
}

#[derive(Resource)]
pub struct MenuState {
    pub screen: MenuScreen,
    pub previous_screen: MenuScreen,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            screen: MenuScreen::Main, // Start in Main Menu!
            previous_screen: MenuScreen::Main,
        }
    }
}

impl MenuState {
    #[inline]
    pub fn is_open(&self) -> bool {
        self.screen != MenuScreen::None
    }
}

#[derive(Resource, Clone, Debug)]
pub struct DevSettings {
    pub backface_culling: bool,
    pub shadows_enabled: bool,
    pub max_y_skip: bool,
    pub distance_fog: bool,
    pub mesh_budget: bool,
    pub async_meshing: bool,
    pub greedy_meshing: bool,
    pub distance_lod: bool,
    pub lod_threshold: i32,
    pub cull_submerged: bool,
    pub show_debug_hud: bool,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            backface_culling: true,
            shadows_enabled: true,
            max_y_skip: true,
            distance_fog: true,
            mesh_budget: true,
            async_meshing: true,
            greedy_meshing: true,
            distance_lod: true,
            lod_threshold: 4,
            cull_submerged: true,
            show_debug_hud: true,
        }
    }
}

#[derive(Resource)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub fps_cap: Option<u32>, // None = Uncapped, Some(60), Some(120), Some(144)
    pub view_distance: i32,   // 8, 16, 24, 32, 64
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            fps_cap: None,
            view_distance: 16,
        }
    }
}

#[derive(Resource, Default)]
pub struct FpsLimiter {
    pub last_frame_instant: Option<std::time::Instant>,
}

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    ResumeGame,
    OpenSettings,
    OpenDevSettings,
    BackFromSettings,
    BackFromDevSettings,
    BackToMain,
    QuitGame,
    ToggleVsync,
    ToggleFullscreen,
    CycleFpsCap,
    CycleViewDistance,
    ToggleBackfaceCulling,
    ToggleShadows,
    ToggleMaxYSkip,
    ToggleDistanceFog,
    ToggleMeshBudget,
    ToggleAsyncMeshing,
    ToggleGreedyMeshing,
    ToggleDistanceLod,
    CycleLodThreshold,
    ToggleCullSubmerged,
    ToggleDebugHud,
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SettingsMenuRoot;

#[derive(Component)]
pub struct DevSettingsMenuRoot;

#[derive(Component)]
pub struct VsyncBtnText;

#[derive(Component)]
pub struct FullscreenBtnText;

#[derive(Component)]
pub struct FpsCapBtnText;

#[derive(Component)]
pub struct ViewDistanceBtnText;

#[derive(Component)]
pub struct BackfaceCullingBtnText;

#[derive(Component)]
pub struct ShadowsBtnText;

#[derive(Component)]
pub struct MaxYSkipBtnText;

#[derive(Component)]
pub struct DistanceFogBtnText;

#[derive(Component)]
pub struct MeshBudgetBtnText;

#[derive(Component)]
pub struct AsyncMeshingBtnText;

#[derive(Component)]
pub struct GreedyMeshingBtnText;

#[derive(Component)]
pub struct DistanceLodBtnText;

#[derive(Component)]
pub struct LodThresholdBtnText;

#[derive(Component)]
pub struct CullSubmergedBtnText;

#[derive(Component)]
pub struct DebugHudBtnText;

pub fn setup_menu_ui(mut commands: Commands) {
    // 1. MAIN MENU SCREEN
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.85)),
            Visibility::Inherited,
            MainMenuRoot,
        ))
        .with_children(|parent| {
            // Title Header Panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("MINERUST ⛏️🦀"),
                        TextFont {
                            font_size: FontSize::Px(44.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    header.spawn((
                        Text::new("A High-Performance Voxel Sandbox in Rust"),
                        TextFont {
                            font_size: FontSize::Px(16.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                        Node {
                            margin: UiRect::top(Val::Px(6.0)),
                            ..default()
                        },
                    ));
                });

            // Main Menu Buttons Panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|btn_col| {
                    spawn_menu_button(btn_col, "▶ Play Game", MenuButtonAction::Play, true);
                    spawn_menu_button(btn_col, "⚙ Graphics Settings", MenuButtonAction::OpenSettings, false);
                    spawn_menu_button(btn_col, "🛠 Dev & Benchmarks", MenuButtonAction::OpenDevSettings, false);
                    spawn_menu_button(btn_col, "✕ Quit Game", MenuButtonAction::QuitGame, false);
                });
        });

    // 2. PAUSE MENU SCREEN
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.85)),
            Visibility::Hidden,
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME PAUSED"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|btn_col| {
                    spawn_menu_button(btn_col, "▶ Resume Game", MenuButtonAction::ResumeGame, true);
                    spawn_menu_button(btn_col, "⚙ Graphics Settings", MenuButtonAction::OpenSettings, false);
                    spawn_menu_button(btn_col, "🛠 Dev & Benchmarks", MenuButtonAction::OpenDevSettings, false);
                    spawn_menu_button(btn_col, "⌂ Return to Main Menu", MenuButtonAction::BackToMain, false);
                });
        });

    // 3. GRAPHICS SETTINGS SCREEN
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.90)),
            Visibility::Hidden,
            SettingsMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GRAPHICS SETTINGS"),
                TextFont {
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|btn_col| {
                    // VSync Button
                    spawn_settings_button(btn_col, "VSync: ON (Smooth)", MenuButtonAction::ToggleVsync, VsyncBtnText);

                    // Fullscreen Button
                    spawn_settings_button(btn_col, "Display: Windowed (1280x720)", MenuButtonAction::ToggleFullscreen, FullscreenBtnText);

                    // FPS Cap Button
                    spawn_settings_button(btn_col, "FPS Limit: Uncapped", MenuButtonAction::CycleFpsCap, FpsCapBtnText);

                    // Render Distance Button
                    spawn_settings_button(btn_col, "Render Distance: 16 Chunks", MenuButtonAction::CycleViewDistance, ViewDistanceBtnText);

                    // Back Button
                    spawn_menu_button(btn_col, "◀ Back / Done", MenuButtonAction::BackFromSettings, true);
                });
        });

    // 4. DEV & BENCHMARK SETTINGS SCREEN
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.92)),
            Visibility::Hidden,
            DevSettingsMenuRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("🛠 DEV & BENCHMARK SETTINGS"),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.9, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Benchmark real-time performance impacts of each optimization technique"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.75, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|btn_col| {
                    spawn_settings_button(btn_col, "Backface Culling: ON", MenuButtonAction::ToggleBackfaceCulling, BackfaceCullingBtnText);
                    spawn_settings_button(btn_col, "Dynamic Shadows: ON", MenuButtonAction::ToggleShadows, ShadowsBtnText);
                    spawn_settings_button(btn_col, "Mesher max_y Skip: ON", MenuButtonAction::ToggleMaxYSkip, MaxYSkipBtnText);
                    spawn_settings_button(btn_col, "Distance Fog: ON", MenuButtonAction::ToggleDistanceFog, DistanceFogBtnText);
                    spawn_settings_button(btn_col, "Mesh Budget: ON (6/frame)", MenuButtonAction::ToggleMeshBudget, MeshBudgetBtnText);
                    spawn_settings_button(btn_col, "Async Meshing: ON (0ms main thread)", MenuButtonAction::ToggleAsyncMeshing, AsyncMeshingBtnText);
                    spawn_settings_button(btn_col, "Greedy Meshing: ON (-75% verts)", MenuButtonAction::ToggleGreedyMeshing, GreedyMeshingBtnText);
                    spawn_settings_button(btn_col, "Distance LOD: ON (Dynamic detail)", MenuButtonAction::ToggleDistanceLod, DistanceLodBtnText);
                    spawn_settings_button(btn_col, "LOD Distance: 4 Chunks (64m)", MenuButtonAction::CycleLodThreshold, LodThresholdBtnText);
                    spawn_settings_button(btn_col, "Water Culls Seabed: ON", MenuButtonAction::ToggleCullSubmerged, CullSubmergedBtnText);
                    spawn_settings_button(btn_col, "Dev HUD (F3): ON", MenuButtonAction::ToggleDebugHud, DebugHudBtnText);
                    spawn_menu_button(btn_col, "◀ Back / Done", MenuButtonAction::BackFromDevSettings, true);
                });
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    highlight: bool,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(if highlight { 2.5 } else { 1.5 })),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
            BorderColor::all(if highlight {
                Color::srgb(1.0, 0.85, 0.2)
            } else {
                Color::srgba(0.45, 0.45, 0.55, 0.8)
            }),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_settings_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    text_marker: T,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(320.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.5)),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
            BorderColor::all(Color::srgba(0.45, 0.45, 0.55, 0.8)),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
                text_marker,
            ));
        });
}

pub fn menu_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    inventory: Option<Res<Inventory>>,
    mut dev_settings: Option<ResMut<DevSettings>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::F3) {
        if let Some(ref mut dev) = dev_settings {
            dev.show_debug_hud = !dev.show_debug_hud;
        }
    }

    // If inventory is open, let inventory handle ESC
    if let Some(inv) = inventory {
        if inv.is_open {
            return;
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        match menu.screen {
            MenuScreen::None => {
                // In game -> Open pause menu and unlock cursor
                menu.screen = MenuScreen::Pause;
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
            MenuScreen::Pause => {
                // In pause menu -> Resume game and lock cursor
                menu.screen = MenuScreen::None;
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
            MenuScreen::Settings | MenuScreen::DevSettings => {
                // Return to previous screen (Main or Pause)
                menu.screen = menu.previous_screen;
            }
            MenuScreen::Main => {
                // In main menu -> Do nothing on ESC
            }
        }
    }
}

pub fn update_menu_visibility_system(
    menu: Res<MenuState>,
    mut main_query: Query<&mut Visibility, (With<MainMenuRoot>, Without<PauseMenuRoot>, Without<SettingsMenuRoot>, Without<DevSettingsMenuRoot>)>,
    mut pause_query: Query<&mut Visibility, (With<PauseMenuRoot>, Without<MainMenuRoot>, Without<SettingsMenuRoot>, Without<DevSettingsMenuRoot>)>,
    mut settings_query: Query<&mut Visibility, (With<SettingsMenuRoot>, Without<MainMenuRoot>, Without<PauseMenuRoot>, Without<DevSettingsMenuRoot>)>,
    mut dev_query: Query<&mut Visibility, (With<DevSettingsMenuRoot>, Without<MainMenuRoot>, Without<PauseMenuRoot>, Without<SettingsMenuRoot>)>,
) {
    if let Ok(mut vis) = main_query.single_mut() {
        let target = if menu.screen == MenuScreen::Main { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != target { *vis = target; }
    }
    if let Ok(mut vis) = pause_query.single_mut() {
        let target = if menu.screen == MenuScreen::Pause { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != target { *vis = target; }
    }
    if let Ok(mut vis) = settings_query.single_mut() {
        let target = if menu.screen == MenuScreen::Settings { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != target { *vis = target; }
    }
    if let Ok(mut vis) = dev_query.single_mut() {
        let target = if menu.screen == MenuScreen::DevSettings { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != target { *vis = target; }
    }
}

pub fn menu_button_hover_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg, mut border) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.26, 0.26, 0.34, 0.95));
                *border = BorderColor::all(Color::srgb(1.0, 0.9, 0.3));
            }
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.36, 0.36, 0.46, 1.0));
                *border = BorderColor::all(Color::srgb(1.0, 1.0, 0.6));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9));
                *border = BorderColor::all(Color::srgba(0.45, 0.45, 0.55, 0.8));
            }
        }
    }
}

pub fn menu_button_click_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu: ResMut<MenuState>,
    mut settings: ResMut<GraphicsSettings>,
    mut dev_settings: Option<ResMut<DevSettings>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    let Ok(mut window) = window_query.single_mut() else {
        return;
    };
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    for (interaction, action) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Play | MenuButtonAction::ResumeGame => {
                    menu.screen = MenuScreen::None;
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
                MenuButtonAction::OpenSettings => {
                    menu.previous_screen = menu.screen;
                    menu.screen = MenuScreen::Settings;
                }
                MenuButtonAction::OpenDevSettings => {
                    menu.previous_screen = menu.screen;
                    menu.screen = MenuScreen::DevSettings;
                }
                MenuButtonAction::BackFromSettings | MenuButtonAction::BackFromDevSettings => {
                    menu.screen = menu.previous_screen;
                }
                MenuButtonAction::BackToMain => {
                    menu.screen = MenuScreen::Main;
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
                MenuButtonAction::QuitGame => {
                    exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::ToggleVsync => {
                    settings.vsync = !settings.vsync;
                    window.present_mode = if settings.vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    };
                }
                MenuButtonAction::ToggleFullscreen => {
                    settings.fullscreen = !settings.fullscreen;
                    window.mode = if settings.fullscreen {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                    } else {
                        WindowMode::Windowed
                    };
                }
                MenuButtonAction::CycleFpsCap => {
                    settings.fps_cap = match settings.fps_cap {
                        None => Some(60),
                        Some(60) => Some(120),
                        Some(120) => Some(144),
                        Some(144) => None,
                        _ => None,
                    };
                }
                MenuButtonAction::CycleViewDistance => {
                    settings.view_distance = match settings.view_distance {
                        8 => 16,
                        16 => 24,
                        24 => 32,
                        32 => 64,
                        64 => 8,
                        _ => 16,
                    };
                }
                MenuButtonAction::ToggleBackfaceCulling => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.backface_culling = !dev.backface_culling;
                    }
                }
                MenuButtonAction::ToggleShadows => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.shadows_enabled = !dev.shadows_enabled;
                    }
                }
                MenuButtonAction::ToggleMaxYSkip => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.max_y_skip = !dev.max_y_skip;
                    }
                }
                MenuButtonAction::ToggleDistanceFog => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.distance_fog = !dev.distance_fog;
                    }
                }
                MenuButtonAction::ToggleMeshBudget => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.mesh_budget = !dev.mesh_budget;
                    }
                }
                MenuButtonAction::ToggleAsyncMeshing => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.async_meshing = !dev.async_meshing;
                    }
                }
                MenuButtonAction::ToggleGreedyMeshing => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.greedy_meshing = !dev.greedy_meshing;
                    }
                }
                MenuButtonAction::ToggleDistanceLod => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.distance_lod = !dev.distance_lod;
                    }
                }
                MenuButtonAction::CycleLodThreshold => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.lod_threshold = match dev.lod_threshold {
                            2 => 3,
                            3 => 4,
                            4 => 6,
                            6 => 8,
                            8 => 2,
                            _ => 4,
                        };
                    }
                }
                MenuButtonAction::ToggleCullSubmerged => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.cull_submerged = !dev.cull_submerged;
                    }
                }
                MenuButtonAction::ToggleDebugHud => {
                    if let Some(ref mut dev) = dev_settings {
                        dev.show_debug_hud = !dev.show_debug_hud;
                    }
                }
            }
        }
    }
}

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    mut vsync_text_query: Query<&mut Text, (With<VsyncBtnText>, Without<FullscreenBtnText>, Without<FpsCapBtnText>, Without<ViewDistanceBtnText>)>,
    mut fs_text_query: Query<&mut Text, (With<FullscreenBtnText>, Without<VsyncBtnText>, Without<FpsCapBtnText>, Without<ViewDistanceBtnText>)>,
    mut fps_text_query: Query<&mut Text, (With<FpsCapBtnText>, Without<VsyncBtnText>, Without<FullscreenBtnText>, Without<ViewDistanceBtnText>)>,
    mut dist_text_query: Query<&mut Text, (With<ViewDistanceBtnText>, Without<VsyncBtnText>, Without<FullscreenBtnText>, Without<FpsCapBtnText>)>,
) {
    if settings.is_changed() {
        if let Ok(mut text) = vsync_text_query.single_mut() {
            *text = Text::new(format!(
                "VSync: {}",
                if settings.vsync { "ON (Smooth)" } else { "OFF (Uncapped)" }
            ));
        }
        if let Ok(mut text) = fs_text_query.single_mut() {
            *text = Text::new(format!(
                "Display: {}",
                if settings.fullscreen { "Fullscreen" } else { "Windowed (1280x720)" }
            ));
        }
        if let Ok(mut text) = fps_text_query.single_mut() {
            *text = Text::new(format!(
                "FPS Limit: {}",
                match settings.fps_cap {
                    None => "Uncapped".to_string(),
                    Some(cap) => format!("{} FPS", cap),
                }
            ));
        }
        if let Ok(mut text) = dist_text_query.single_mut() {
            *text = Text::new(format!(
                "Render Distance: {} Chunks",
                settings.view_distance
            ));
        }
    }
}

pub fn update_dev_button_text_system(
    dev_settings: Option<Res<DevSettings>>,
    mut cull_text_query: Query<&mut Text, (With<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut shadow_text_query: Query<&mut Text, (With<ShadowsBtnText>, Without<BackfaceCullingBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut max_y_text_query: Query<&mut Text, (With<MaxYSkipBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut fog_text_query: Query<&mut Text, (With<DistanceFogBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut budget_text_query: Query<&mut Text, (With<MeshBudgetBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut async_text_query: Query<&mut Text, (With<AsyncMeshingBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut greedy_text_query: Query<&mut Text, (With<GreedyMeshingBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut lod_text_query: Query<&mut Text, (With<DistanceLodBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut thresh_text_query: Query<&mut Text, (With<LodThresholdBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<CullSubmergedBtnText>, Without<DebugHudBtnText>)>,
    mut cull_submerged_text_query: Query<&mut Text, (With<CullSubmergedBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<DebugHudBtnText>)>,
    mut hud_text_query: Query<&mut Text, (With<DebugHudBtnText>, Without<BackfaceCullingBtnText>, Without<ShadowsBtnText>, Without<MaxYSkipBtnText>, Without<DistanceFogBtnText>, Without<MeshBudgetBtnText>, Without<AsyncMeshingBtnText>, Without<GreedyMeshingBtnText>, Without<DistanceLodBtnText>, Without<LodThresholdBtnText>, Without<CullSubmergedBtnText>)>,
) {
    let Some(dev) = dev_settings else { return; };
    if dev.is_changed() {
        if let Ok(mut text) = cull_text_query.single_mut() {
            *text = Text::new(format!(
                "Backface Culling: {}",
                if dev.backface_culling { "ON (GPU -50%)" } else { "OFF (Draw front & back)" }
            ));
        }
        if let Ok(mut text) = shadow_text_query.single_mut() {
            *text = Text::new(format!(
                "Dynamic Shadows: {}",
                if dev.shadows_enabled { "ON (120m Cascades)" } else { "OFF (Zero shadow passes)" }
            ));
        }
        if let Ok(mut text) = max_y_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesher max_y Skip: {}",
                if dev.max_y_skip { "ON (2x faster meshing)" } else { "OFF (Loop all 384 layers)" }
            ));
        }
        if let Ok(mut text) = fog_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if dev.distance_fog { "ON (Blended horizon)" } else { "OFF (Harsh edge)" }
            ));
        }
        if let Ok(mut text) = budget_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesh Budget: {}",
                if dev.mesh_budget { "ON (6/frame smooth)" } else { "OFF (Spike benchmark)" }
            ));
        }
        if let Ok(mut text) = async_text_query.single_mut() {
            *text = Text::new(format!(
                "Async Meshing: {}",
                if dev.async_meshing { "ON (0ms main thread)" } else { "OFF (Sync frame spikes)" }
            ));
        }
        if let Ok(mut text) = greedy_text_query.single_mut() {
            *text = Text::new(format!(
                "Greedy Meshing: {}",
                if dev.greedy_meshing { "ON (-75% verts)" } else { "OFF (1x1 block quads)" }
            ));
        }
        if let Ok(mut text) = lod_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance LOD: {}",
                if dev.distance_lod { "ON (Dynamic detail)" } else { "OFF (Uniform meshing)" }
            ));
        }
        if let Ok(mut text) = thresh_text_query.single_mut() {
            *text = Text::new(format!(
                "LOD Distance: {} Chunks ({}m)",
                dev.lod_threshold,
                dev.lod_threshold * 16
            ));
        }
        if let Ok(mut text) = cull_submerged_text_query.single_mut() {
            *text = Text::new(format!(
                "Water Culls Seabed: {}",
                if dev.cull_submerged { "ON (Hide underwater terrain)" } else { "OFF (Render full seabed)" }
            ));
        }
        if let Ok(mut text) = hud_text_query.single_mut() {
            *text = Text::new(format!(
                "Dev HUD (F3): {}",
                if dev.show_debug_hud { "ON (Visible)" } else { "OFF (Hidden)" }
            ));
        }
    }
}

pub fn update_dev_settings_system(
    dev_settings: Option<Res<DevSettings>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<crate::world::WorldGrid>,
    mut dir_lights: Query<&mut DirectionalLight>,
    mut fog_query: Query<&mut bevy::pbr::DistanceFog>,
    mut last_greedy: Local<Option<bool>>,
    mut last_lod: Local<Option<(bool, i32)>>,
    mut last_cull_submerged: Local<Option<bool>>,
) {
    let Some(dev) = dev_settings else { return; };
    if dev.is_changed() {
        // 1. If greedy meshing, LOD settings, or water submerged culling changed, re-queue all loaded chunks for re-meshing
        let lod_config = (dev.distance_lod, dev.lod_threshold);
        if last_greedy.map_or(false, |last| last != dev.greedy_meshing)
            || last_lod.map_or(false, |last| last != lod_config)
            || last_cull_submerged.map_or(false, |last| last != dev.cull_submerged)
        {
            let coords: Vec<_> = world.chunks.keys().copied().collect();
            for coord in coords {
                world.queue_mesh(coord);
            }
        }
        *last_greedy = Some(dev.greedy_meshing);
        *last_lod = Some(lod_config);
        *last_cull_submerged = Some(dev.cull_submerged);

        // 2. Update Backface Culling in real-time across ALL chunks
        if let Some(ref mat_handle) = world.block_material {
            if let Some(mut mat) = materials.get_mut(mat_handle) {
                mat.cull_mode = if dev.backface_culling {
                    Some(bevy::render::render_resource::Face::Back)
                } else {
                    None
                };
            }
        }

        // 3. Update Directional Light Shadows in real-time
        for mut light in &mut dir_lights {
            light.shadow_maps_enabled = dev.shadows_enabled;
        }

        // 4. Update Distance Fog in real-time
        for mut fog in &mut fog_query {
            if dev.distance_fog {
                fog.falloff = bevy::pbr::FogFalloff::Linear {
                    start: 180.0,
                    end: 255.0,
                };
            } else {
                fog.falloff = bevy::pbr::FogFalloff::Linear {
                    start: 99999.0,
                    end: 100000.0,
                };
            }
        }
    }
}

pub fn fps_limiter_system(
    settings: Res<GraphicsSettings>,
    mut limiter: ResMut<FpsLimiter>,
) {
    if let Some(cap) = settings.fps_cap {
        let target_frame_duration = std::time::Duration::from_secs_f64(1.0 / cap as f64);
        if let Some(last) = limiter.last_frame_instant {
            let elapsed = last.elapsed();
            if elapsed < target_frame_duration {
                std::thread::sleep(target_frame_duration - elapsed);
            }
        }
        limiter.last_frame_instant = Some(std::time::Instant::now());
    } else {
        limiter.last_frame_instant = None;
    }
}
