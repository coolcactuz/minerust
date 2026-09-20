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
    BackFromSettings,
    BackToMain,
    QuitGame,
    ToggleVsync,
    ToggleFullscreen,
    CycleFpsCap,
    CycleViewDistance,
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SettingsMenuRoot;

#[derive(Component)]
pub struct VsyncBtnText;

#[derive(Component)]
pub struct FullscreenBtnText;

#[derive(Component)]
pub struct FpsCapBtnText;

#[derive(Component)]
pub struct ViewDistanceBtnText;

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
                    spawn_settings_button(btn_col, "Render Distance: 8 Chunks", MenuButtonAction::CycleViewDistance, ViewDistanceBtnText);

                    // Back Button
                    spawn_menu_button(btn_col, "◀ Back / Done", MenuButtonAction::BackFromSettings, true);
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
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

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
            MenuScreen::Settings => {
                // In settings -> Return to previous screen (Main or Pause)
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
    mut main_query: Query<&mut Visibility, (With<MainMenuRoot>, Without<PauseMenuRoot>, Without<SettingsMenuRoot>)>,
    mut pause_query: Query<&mut Visibility, (With<PauseMenuRoot>, Without<MainMenuRoot>, Without<SettingsMenuRoot>)>,
    mut settings_query: Query<&mut Visibility, (With<SettingsMenuRoot>, Without<MainMenuRoot>, Without<PauseMenuRoot>)>,
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
                MenuButtonAction::BackFromSettings => {
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
