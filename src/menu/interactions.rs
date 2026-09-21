use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, WindowMode,
};

use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::physics::PlayerPhysics;
use crate::save::load_player_from_disk;
use crate::world::{WorldGrid, WorldSeed, find_safe_surface_spawn};

use super::descriptions::get_option_description;
use super::types::{
    AsyncMeshingBtnText, BackfaceCullingBtnText, DebugHudBtnText, DevSettings, DevSettingsMenuRoot,
    DistanceFogBtnText, DistanceLodBtnText, FpsCapBtnText, FpsLimiter, FullscreenBtnText,
    GraphicsSettings, GreedyMeshingBtnText, LodThresholdBtnText, MainMenuRoot, MaxYSkipBtnText,
    MenuButtonAction, MenuScreen, MenuState, MeshBudgetBtnText, OptionTooltipCard,
    OptionTooltipDesc, OptionTooltipHeader, OptionTooltipImpact, OptionTooltipTitle, PauseMenuRoot,
    PregenMarginBtnText, SeedInputBox, SeedInputState, SettingsMenuRoot, ShadowsBtnText,
    ViewDistanceBtnText, VsyncBtnText,
};

pub fn menu_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    inventory: Option<Res<Inventory>>,
    mut dev_settings: Option<ResMut<DevSettings>>,
    mut seed_state: Option<ResMut<SeedInputState>>,
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
        if let Some(ref mut state) = seed_state {
            if state.is_editing {
                state.is_editing = false;
                return;
            }
        }

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
    dev_settings: Option<Res<DevSettings>>,
    mut main_query: Query<
        &mut Visibility,
        (
            With<MainMenuRoot>,
            Without<PauseMenuRoot>,
            Without<SettingsMenuRoot>,
            Without<DevSettingsMenuRoot>,
        ),
    >,
    mut pause_query: Query<
        &mut Visibility,
        (
            With<PauseMenuRoot>,
            Without<MainMenuRoot>,
            Without<SettingsMenuRoot>,
            Without<DevSettingsMenuRoot>,
        ),
    >,
    mut settings_query: Query<
        &mut Visibility,
        (
            With<SettingsMenuRoot>,
            Without<MainMenuRoot>,
            Without<PauseMenuRoot>,
            Without<DevSettingsMenuRoot>,
        ),
    >,
    mut dev_query: Query<
        &mut Visibility,
        (
            With<DevSettingsMenuRoot>,
            Without<MainMenuRoot>,
            Without<PauseMenuRoot>,
            Without<SettingsMenuRoot>,
        ),
    >,
) {
    if let Ok(mut vis) = main_query.single_mut() {
        let target = if menu.screen == MenuScreen::Main {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }
    if let Ok(mut vis) = pause_query.single_mut() {
        let target = if menu.screen == MenuScreen::Pause {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }
    if let Ok(mut vis) = settings_query.single_mut() {
        let target = if menu.screen == MenuScreen::Settings {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }
    if let Ok(mut vis) = dev_query.single_mut() {
        let is_dev = dev_settings.as_ref().is_some_and(|d| d.dev_mode);
        let target = if is_dev && menu.screen == MenuScreen::DevSettings {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }
}

pub fn menu_button_hover_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>, Without<SeedInputBox>),
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
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu: ResMut<MenuState>,
    mut settings: ResMut<GraphicsSettings>,
    mut dev_settings: Option<ResMut<DevSettings>>,
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut world: Option<ResMut<WorldGrid>>,
    mut inventory: Option<ResMut<Inventory>>,
    mut player_query: Query<(&mut Transform, &mut FpsCamera, &mut PlayerPhysics)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                MenuButtonAction::ToggleEditSeed => {
                    if let Some(ref mut state) = seed_state {
                        state.is_editing = !state.is_editing;
                    }
                }
                MenuButtonAction::RandomizeSeed => {
                    if let Some(ref mut state) = seed_state {
                        let new_seed = WorldSeed::random();
                        state.seed_text = new_seed.0.to_string();
                        state.is_editing = false;
                    }
                }
                MenuButtonAction::Play => {
                    if let Some(ref mut state) = seed_state {
                        state.is_editing = false;
                        let target_seed = if state.seed_text.trim().is_empty() {
                            WorldSeed::random()
                        } else {
                            WorldSeed::from_seed_str(&state.seed_text)
                        };

                        if let Some(ref mut w) = world {
                            if w.seed != target_seed {
                                w.reinitialize_with_seed(target_seed, &mut commands);

                                let player_save_file = w.player_save_path();
                                let (player_pos, player_yaw, player_pitch) =
                                    if player_save_file.exists() {
                                        if let Ok(data) = load_player_from_disk(&player_save_file) {
                                            if let Some(ref mut inv) = inventory {
                                                inv.hotbar = data.hotbar;
                                                inv.main = data.main;
                                                inv.selected_slot = data.selected_slot;
                                            }
                                            (Vec3::from_array(data.position), data.yaw, data.pitch)
                                        } else {
                                            let spawn_pos =
                                                find_safe_surface_spawn(&w.noise, target_seed.0);
                                            if let Some(ref mut inv) = inventory {
                                                **inv = Inventory::default();
                                            }
                                            (spawn_pos, -std::f32::consts::FRAC_PI_2, -0.3)
                                        }
                                    } else {
                                        let spawn_pos =
                                            find_safe_surface_spawn(&w.noise, target_seed.0);
                                        if let Some(ref mut inv) = inventory {
                                            **inv = Inventory::default();
                                        }
                                        (spawn_pos, -std::f32::consts::FRAC_PI_2, -0.3)
                                    };

                                if let Ok((mut transform, mut fps_cam, mut physics)) =
                                    player_query.single_mut()
                                {
                                    transform.translation = player_pos;
                                    transform.rotation = Quat::from_rotation_y(player_yaw)
                                        * Quat::from_rotation_x(player_pitch);
                                    fps_cam.yaw = player_yaw;
                                    fps_cam.pitch = player_pitch;
                                    physics.velocity = Vec3::ZERO;
                                }

                                let center_chunk = WorldGrid::world_to_chunk_coord(
                                    player_pos.x.floor() as i32,
                                    player_pos.z.floor() as i32,
                                )
                                .0;

                                w.pregenerate_spawn_grid(
                                    center_chunk,
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                );

                                window.title = if dev_settings.as_ref().is_some_and(|d| d.dev_mode)
                                {
                                    format!("MineRust [DEV MODE] - Seed: {}", target_seed.0)
                                } else {
                                    format!("MineRust - Seed: {}", target_seed.0)
                                };
                            }
                        }
                    }
                    menu.screen = MenuScreen::None;
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
                MenuButtonAction::ResumeGame => {
                    menu.screen = MenuScreen::None;
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
                MenuButtonAction::OpenSettings => {
                    menu.previous_screen = menu.screen;
                    menu.screen = MenuScreen::Settings;
                }
                MenuButtonAction::OpenDevSettings => {
                    if dev_settings.as_ref().is_some_and(|d| d.dev_mode) {
                        menu.previous_screen = menu.screen;
                        menu.screen = MenuScreen::DevSettings;
                    }
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
                        _ => None,
                    };
                }
                MenuButtonAction::CycleViewDistance => {
                    settings.view_distance = match settings.view_distance {
                        16 => 24,
                        24 => 32,
                        32 => 64,
                        64 => 8,
                        _ => 16,
                    };
                }
                MenuButtonAction::ToggleBackfaceCulling => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.backface_culling = !dev.backface_culling;
                        }
                    }
                }
                MenuButtonAction::ToggleShadows => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.shadows_enabled = !dev.shadows_enabled;
                        }
                    }
                }
                MenuButtonAction::ToggleMaxYSkip => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.max_y_skip = !dev.max_y_skip;
                        }
                    }
                }
                MenuButtonAction::ToggleDistanceFog => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.distance_fog = !dev.distance_fog;
                        }
                    }
                }
                MenuButtonAction::ToggleMeshBudget => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.mesh_budget = !dev.mesh_budget;
                        }
                    }
                }
                MenuButtonAction::ToggleAsyncMeshing => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.async_meshing = !dev.async_meshing;
                        }
                    }
                }
                MenuButtonAction::ToggleGreedyMeshing => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.greedy_meshing = !dev.greedy_meshing;
                        }
                    }
                }
                MenuButtonAction::ToggleDistanceLod => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.distance_lod = !dev.distance_lod;
                        }
                    }
                }
                MenuButtonAction::CycleLodThreshold => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.lod_threshold = match dev.lod_threshold {
                                2 => 3,
                                4 => 6,
                                6 => 8,
                                8 => 2,
                                _ => 4,
                            };
                        }
                    }
                }
                MenuButtonAction::CyclePregenMargin => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.pregen_margin = match dev.pregen_margin {
                                0 => 1,
                                1 => 2,
                                2 => 3,
                                3 => 4,
                                _ => 0,
                            };
                        }
                    }
                }
                MenuButtonAction::ToggleDebugHud => {
                    if let Some(ref mut dev) = dev_settings {
                        if dev.dev_mode {
                            dev.show_debug_hud = !dev.show_debug_hud;
                        }
                    }
                }
            }
        }
    }
}

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    mut vsync_text_query: Query<
        &mut Text,
        (
            With<VsyncBtnText>,
            Without<FullscreenBtnText>,
            Without<FpsCapBtnText>,
            Without<ViewDistanceBtnText>,
        ),
    >,
    mut fs_text_query: Query<
        &mut Text,
        (
            With<FullscreenBtnText>,
            Without<VsyncBtnText>,
            Without<FpsCapBtnText>,
            Without<ViewDistanceBtnText>,
        ),
    >,
    mut fps_text_query: Query<
        &mut Text,
        (
            With<FpsCapBtnText>,
            Without<VsyncBtnText>,
            Without<FullscreenBtnText>,
            Without<ViewDistanceBtnText>,
        ),
    >,
    mut dist_text_query: Query<
        &mut Text,
        (
            With<ViewDistanceBtnText>,
            Without<VsyncBtnText>,
            Without<FullscreenBtnText>,
            Without<FpsCapBtnText>,
        ),
    >,
) {
    if settings.is_changed() {
        if let Ok(mut text) = vsync_text_query.single_mut() {
            *text = Text::new(format!(
                "VSync: {}",
                if settings.vsync {
                    "ON (Smooth)"
                } else {
                    "OFF (Uncapped)"
                }
            ));
        }
        if let Ok(mut text) = fs_text_query.single_mut() {
            *text = Text::new(format!(
                "Display: {}",
                if settings.fullscreen {
                    "Fullscreen"
                } else {
                    "Windowed (1280x720)"
                }
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
    mut cull_text_query: Query<
        &mut Text,
        (
            With<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut shadow_text_query: Query<
        &mut Text,
        (
            With<ShadowsBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut max_y_text_query: Query<
        &mut Text,
        (
            With<MaxYSkipBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut fog_text_query: Query<
        &mut Text,
        (
            With<DistanceFogBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut budget_text_query: Query<
        &mut Text,
        (
            With<MeshBudgetBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut async_text_query: Query<
        &mut Text,
        (
            With<AsyncMeshingBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut greedy_text_query: Query<
        &mut Text,
        (
            With<GreedyMeshingBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut lod_text_query: Query<
        &mut Text,
        (
            With<DistanceLodBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut thresh_text_query: Query<
        &mut Text,
        (
            With<LodThresholdBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut hud_text_query: Query<
        &mut Text,
        (
            With<DebugHudBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut margin_text_query: Query<
        &mut Text,
        (
            With<PregenMarginBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
        ),
    >,
) {
    let Some(dev) = dev_settings else {
        return;
    };
    if dev.is_changed() {
        if let Ok(mut text) = cull_text_query.single_mut() {
            *text = Text::new(format!(
                "Backface Culling: {}",
                if dev.backface_culling {
                    "ON (GPU -50%)"
                } else {
                    "OFF (Draw front & back)"
                }
            ));
        }
        if let Ok(mut text) = shadow_text_query.single_mut() {
            *text = Text::new(format!(
                "Dynamic Shadows: {}",
                if dev.shadows_enabled {
                    "ON (120m Cascades)"
                } else {
                    "OFF (Zero shadow passes)"
                }
            ));
        }
        if let Ok(mut text) = max_y_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesher max_y Skip: {}",
                if dev.max_y_skip {
                    "ON (2x faster meshing)"
                } else {
                    "OFF (Loop all 384 layers)"
                }
            ));
        }
        if let Ok(mut text) = fog_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if dev.distance_fog {
                    "ON (Blended horizon)"
                } else {
                    "OFF (Harsh edge)"
                }
            ));
        }
        if let Ok(mut text) = budget_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesh Budget: {}",
                if dev.mesh_budget {
                    "ON (6/frame smooth)"
                } else {
                    "OFF (Spike benchmark)"
                }
            ));
        }
        if let Ok(mut text) = async_text_query.single_mut() {
            *text = Text::new(format!(
                "Async Meshing: {}",
                if dev.async_meshing {
                    "ON (0ms main thread)"
                } else {
                    "OFF (Sync frame spikes)"
                }
            ));
        }
        if let Ok(mut text) = greedy_text_query.single_mut() {
            *text = Text::new(format!(
                "Greedy Meshing: {}",
                if dev.greedy_meshing {
                    "ON (-75% verts)"
                } else {
                    "OFF (1x1 block quads)"
                }
            ));
        }
        if let Ok(mut text) = lod_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance LOD: {}",
                if dev.distance_lod {
                    "ON (Dynamic detail)"
                } else {
                    "OFF (Uniform meshing)"
                }
            ));
        }
        if let Ok(mut text) = thresh_text_query.single_mut() {
            *text = Text::new(format!(
                "LOD Distance: {} Chunks ({}m)",
                dev.lod_threshold,
                dev.lod_threshold * 16
            ));
        }
        if let Ok(mut text) = margin_text_query.single_mut() {
            *text = Text::new(if dev.pregen_margin == 0 {
                "Lookahead Buffer: 0 (Disabled / Stutter prone)".to_string()
            } else {
                format!(
                    "Lookahead Buffer: {} Chunks (+{}m RAM cache)",
                    dev.pregen_margin,
                    dev.pregen_margin * 16
                )
            });
        }
        if let Ok(mut text) = hud_text_query.single_mut() {
            *text = Text::new(format!(
                "Dev HUD (F3): {}",
                if dev.show_debug_hud {
                    "ON (Visible)"
                } else {
                    "OFF (Hidden)"
                }
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
) {
    let Some(dev) = dev_settings else {
        return;
    };
    if dev.is_changed() {
        // 1. If greedy meshing or LOD settings changed, re-queue all loaded chunks for re-meshing
        let lod_config = (dev.distance_lod, dev.lod_threshold);
        if last_greedy.map_or(false, |last| last != dev.greedy_meshing)
            || last_lod.map_or(false, |last| last != lod_config)
        {
            let coords: Vec<_> = world.chunks.keys().copied().collect();
            for coord in coords {
                world.queue_mesh(coord);
            }
        }
        *last_greedy = Some(dev.greedy_meshing);
        *last_lod = Some(lod_config);

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

pub fn update_option_tooltip_system(
    interaction_query: Query<(&Interaction, &MenuButtonAction), With<Button>>,
    mut last_hovered: Local<Option<MenuButtonAction>>,
    mut header_query: Query<
        &mut Text,
        (
            With<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipDesc>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut title_query: Query<
        &mut Text,
        (
            With<OptionTooltipTitle>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipDesc>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<OptionTooltipDesc>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut impact_query: Query<
        &mut Text,
        (
            With<OptionTooltipImpact>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipDesc>,
        ),
    >,
    mut card_query: Query<&mut BorderColor, With<OptionTooltipCard>>,
) {
    let currently_hovered = interaction_query
        .iter()
        .find(|(interaction, _)| {
            **interaction == Interaction::Hovered || **interaction == Interaction::Pressed
        })
        .map(|(_, action)| *action);

    if *last_hovered != currently_hovered {
        *last_hovered = currently_hovered;

        if let Some(action) = currently_hovered {
            if let Some(desc) = get_option_description(&action) {
                for mut text in &mut header_query {
                    *text = Text::new(format!("[ {} ]", desc.header));
                }
                for mut text in &mut title_query {
                    *text = Text::new(desc.title);
                }
                for mut text in &mut desc_query {
                    *text = Text::new(desc.description);
                }
                for mut text in &mut impact_query {
                    *text = Text::new(desc.impact);
                }
                for mut border in &mut card_query {
                    *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.2));
                }
            }
        } else {
            for mut text in &mut header_query {
                *text = Text::new("[ SETTING INFO ]");
            }
            for mut text in &mut title_query {
                *text = Text::new("Hover over any setting");
            }
            for mut text in &mut desc_query {
                *text = Text::new(
                    "Move your mouse over any graphic or performance setting on the left to inspect its technical details, rendering behavior, and performance impact.",
                );
            }
            for mut text in &mut impact_query {
                *text = Text::new(
                    "- All MineRust optimizations are tuned for maximum 60+ FPS stability.",
                );
            }
            for mut border in &mut card_query {
                *border = BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8));
            }
        }
    }
}

pub fn fps_limiter_system(settings: Res<GraphicsSettings>, mut limiter: ResMut<FpsLimiter>) {
    if let Some(cap) = settings.fps_cap {
        let target_frame_duration = std::time::Duration::from_secs_f64(1.0 / cap as f64);
        if let Some(last) = limiter.last_frame_instant {
            let elapsed = last.elapsed();
            if let Some(remaining) = target_frame_duration.checked_sub(elapsed) {
                std::thread::sleep(remaining);
            }
        }
        limiter.last_frame_instant = Some(std::time::Instant::now());
    } else {
        limiter.last_frame_instant = None;
    }
}
