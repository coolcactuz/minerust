use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, WindowMode};

use crate::benchmark::{BenchmarkConfig, BenchmarkScenario, BenchmarkState};
use crate::inventory::Inventory;
use crate::physics::PlayerPhysics;
use crate::save::{load_player_from_disk, PlayerSaveData};
use crate::voxel_material::VoxelBlockMaterial;
use crate::world::{find_safe_surface_spawn, WorldGrid, WorldSeed};

use super::types::{
    BenchmarkResultsRoot, FpsCapTrack, GraphicsGreedyTrack, GraphicsLodTrack, GraphicsSettings,
    MainMenuRoot, MenuButtonAction, MenuScreen, MenuState, PauseMenuRoot, ProfilerState,
    SeedInputBox, SeedInputState, SettingsMenuRoot, SliderTrack, ViewDistanceTrack,
};

#[derive(SystemParam)]
pub struct MenuAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<VoxelBlockMaterial>>,
}

#[derive(SystemParam)]
pub struct MenuBenchmarkControl<'w> {
    pub config: Option<ResMut<'w, BenchmarkConfig>>,
    pub state: Option<ResMut<'w, BenchmarkState>>,
}

pub fn menu_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    inventory: Option<Res<Inventory>>,
    mut profiler_state: Option<ResMut<ProfilerState>>,
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut bench_control: MenuBenchmarkControl,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::F3) {
        if let Some(ref mut profiler) = profiler_state {
            profiler.visible = !profiler.visible;
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

        // Cancel in-progress hardware benchmark if currently running
        if let (Some(ref mut config), Some(ref mut state)) = (
            bench_control.config.as_deref_mut(),
            bench_control.state.as_deref_mut(),
        ) {
            if config.enabled && !config.is_cli && !state.completed {
                config.enabled = false;
                state.completed = true;
                menu.screen = MenuScreen::Settings;
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
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
            MenuScreen::Settings => {
                // Return to previous screen (Main or Pause)
                menu.screen = menu.previous_screen;
            }
            MenuScreen::BenchmarkResults => {
                // Return to settings menu from benchmark results screen
                menu.screen = MenuScreen::Settings;
            }
            MenuScreen::Main => {
                // In main menu -> Do nothing on ESC
            }
        }
    }
}

pub fn update_menu_visibility_system(
    menu: Res<MenuState>,
    mut query: Query<(
        &mut Visibility,
        Option<&MainMenuRoot>,
        Option<&PauseMenuRoot>,
        Option<&SettingsMenuRoot>,
        Option<&BenchmarkResultsRoot>,
    )>,
) {
    for (mut vis, main, pause, settings, bench) in &mut query {
        let target = if main.is_some() {
            if menu.screen == MenuScreen::Main {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        } else if pause.is_some() {
            if menu.screen == MenuScreen::Pause {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        } else if settings.is_some() {
            if menu.screen == MenuScreen::Settings {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        } else if bench.is_some() {
            if menu.screen == MenuScreen::BenchmarkResults {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        } else {
            continue;
        };

        if *vis != target {
            *vis = target;
        }
    }
}

pub fn menu_button_hover_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            Changed<Interaction>,
            With<Button>,
            Without<SeedInputBox>,
            Without<SliderTrack>,
        ),
    >,
    mut track_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<SliderTrack>),
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

    for (interaction, mut border) in &mut track_query {
        match *interaction {
            Interaction::Hovered | Interaction::Pressed => {
                *border = BorderColor::all(Color::srgb(1.0, 0.9, 0.3));
            }
            Interaction::None => {
                *border = BorderColor::all(Color::srgba(0.35, 0.40, 0.55, 0.7));
            }
        }
    }
}

pub fn menu_button_click_system(
    mut commands: Commands,
    mut menu: ResMut<MenuState>,
    mut settings: ResMut<GraphicsSettings>,
    mut profiler_state: Option<ResMut<ProfilerState>>,
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut world: Option<ResMut<WorldGrid>>,
    mut assets: MenuAssets,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut exit_writer: MessageWriter<AppExit>,
    mut player_query: Query<(&mut Transform, &mut crate::camera::FpsCamera, &mut PlayerPhysics)>,
    mut inventory: Option<ResMut<Inventory>>,
    mut bench_control: MenuBenchmarkControl,
    interaction_query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
) {
    let Ok(mut window) = window_query.single_mut() else {
        return;
    };
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::ToggleEditSeed => {
                    if let Some(ref mut state) = seed_state {
                        state.is_editing = !state.is_editing;
                    }
                }
                MenuButtonAction::RandomizeSeed => {
                    let new_seed = WorldSeed::random();
                    if let Some(ref mut state) = seed_state {
                        state.seed_text = new_seed.0.to_string();
                    }
                }
                MenuButtonAction::ContinueGame => {
                    if let Some(saved_seed) = crate::save::get_latest_saved_world() {
                        let target_seed = WorldSeed(saved_seed);
                        if let Some(ref mut w) = world {
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
                                &mut assets.meshes,
                                &mut assets.materials,
                            );

                            window.title = format!("MineRust - Seed: {}", target_seed.0);
                            crate::save::set_last_played_world(target_seed.0);
                        }
                    }
                    menu.world_active = true;
                    menu.screen = MenuScreen::None;
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
                MenuButtonAction::Play | MenuButtonAction::NewGame => {
                    let target_seed = if let Some(ref state) = seed_state {
                        WorldSeed::from_seed_str(&state.seed_text)
                    } else {
                        WorldSeed::random()
                    };

                    if let Some(ref mut w) = world {
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
                            &mut assets.meshes,
                            &mut assets.materials,
                        );

                        window.title = format!("MineRust - Seed: {}", target_seed.0);
                        crate::save::set_last_played_world(target_seed.0);
                    }
                    menu.world_active = true;
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
                MenuButtonAction::BackFromSettings => {
                    menu.screen = menu.previous_screen;
                }
                MenuButtonAction::BackToMain => {
                    if let Some(ref mut config) = bench_control.config.as_deref_mut() {
                        config.enabled = false;
                    }
                    if let Some(ref mut w) = world {
                        let _ = w.save_all_modified();
                        if let Ok((transform, fps_cam, _)) = player_query.single() {
                            if let Some(ref inv) = inventory {
                                let data = PlayerSaveData::new(
                                    transform.translation,
                                    fps_cam.yaw,
                                    fps_cam.pitch,
                                    inv.hotbar,
                                    inv.main,
                                    inv.selected_slot,
                                );
                                let _ =
                                    crate::save::save_player_to_disk(&w.player_save_path(), &data);
                            }
                        }
                        w.despawn_all_chunks(&mut commands);
                    }
                    menu.world_active = false;
                    menu.screen = MenuScreen::Main;
                    window.title = "MineRust".to_string();
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
                MenuButtonAction::ToggleShadows => {
                    settings.shadows = !settings.shadows;
                }
                MenuButtonAction::ToggleDistanceFog => {
                    settings.distance_fog = !settings.distance_fog;
                }
                MenuButtonAction::ToggleDebugHud => {
                    if let Some(ref mut profiler) = profiler_state {
                        profiler.visible = !profiler.visible;
                    }
                }
                MenuButtonAction::CycleFpsCap | MenuButtonAction::StepFpsCapRight => {
                    settings.step_fps_cap(1);
                }
                MenuButtonAction::StepFpsCapLeft => {
                    settings.step_fps_cap(-1);
                }
                MenuButtonAction::CycleViewDistance | MenuButtonAction::StepViewDistanceRight => {
                    settings.step_view_distance(1);
                }
                MenuButtonAction::StepViewDistanceLeft => {
                    settings.step_view_distance(-1);
                }
                MenuButtonAction::CycleGreedyMeshing | MenuButtonAction::StepGreedyMeshingRight => {
                    settings.step_greedy(1);
                }
                MenuButtonAction::StepGreedyMeshingLeft => {
                    settings.step_greedy(-1);
                }
                MenuButtonAction::ToggleGreedyMeshing => {
                    settings.greedy_meshing = !settings.greedy_meshing;
                }
                MenuButtonAction::CycleDistanceLod | MenuButtonAction::StepDistanceLodRight => {
                    settings.step_lod(1);
                }
                MenuButtonAction::StepDistanceLodLeft => {
                    settings.step_lod(-1);
                }
                MenuButtonAction::ToggleDistanceLod => {
                    settings.distance_lod = !settings.distance_lod;
                }
                MenuButtonAction::StartBenchmark => {
                    let target_seed = WorldSeed::from_seed_str("BENCHMARK");
                    if let Some(ref mut w) = world {
                        w.reinitialize_with_seed(target_seed, &mut commands);

                        let spawn_pos = Vec3::new(
                            0.0,
                            BenchmarkScenario::DEFAULT_FLIGHT_ALTITUDE,
                            0.0,
                        );
                        let player_yaw = 0.0;
                        let player_pitch = -0.06;

                        if let Ok((mut transform, mut fps_cam, mut physics)) =
                            player_query.single_mut()
                        {
                            transform.translation = spawn_pos;
                            transform.rotation = Quat::from_rotation_y(player_yaw)
                                * Quat::from_rotation_x(player_pitch);
                            fps_cam.yaw = player_yaw;
                            fps_cam.pitch = player_pitch;
                            physics.velocity = Vec3::ZERO;
                            physics.is_flying = true;
                        }

                        let center_chunk = WorldGrid::world_to_chunk_coord(0, 0).0;
                        w.pregenerate_spawn_grid(
                            center_chunk,
                            &mut commands,
                            &mut assets.meshes,
                            &mut assets.materials,
                        );

                        window.title = "MineRust [BENCHMARK RUNNING] - Seed: BENCHMARK".to_string();
                    }

                    if let (Some(ref mut config), Some(ref mut state)) = (
                        bench_control.config.as_deref_mut(),
                        bench_control.state.as_deref_mut(),
                    ) {
                        config.enabled = true;
                        config.is_cli = false;
                        config.seed = target_seed.0;
                        config.scenario.view_distance = settings.view_distance;
                        config.scenario.distance_fog = settings.distance_fog;
                        **state = BenchmarkState::default();
                    }

                    menu.world_active = true;
                    menu.screen = MenuScreen::None;
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
                MenuButtonAction::BackFromBenchmark => {
                    menu.screen = MenuScreen::Settings;
                }
                MenuButtonAction::SlideFpsCap
                | MenuButtonAction::SlideViewDistance
                | MenuButtonAction::SlideGreedyMeshing
                | MenuButtonAction::SlideDistanceLod => {}
            }
        }
    }
}

pub fn slider_interaction_system(
    mut settings: ResMut<GraphicsSettings>,
    track_query: Query<
        (
            &Interaction,
            &bevy::ui::RelativeCursorPosition,
            Option<&FpsCapTrack>,
            Option<&ViewDistanceTrack>,
            Option<&GraphicsGreedyTrack>,
            Option<&GraphicsLodTrack>,
        ),
        With<Button>,
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut dragging_fps: Local<bool>,
    mut dragging_dist: Local<bool>,
    mut dragging_greedy: Local<bool>,
    mut dragging_lod: Local<bool>,
) {
    let mouse_down = mouse_buttons.pressed(MouseButton::Left);
    if !mouse_down {
        *dragging_fps = false;
        *dragging_dist = false;
        *dragging_greedy = false;
        *dragging_lod = false;
    }

    for (interaction, rcp, is_fps, is_dist, is_greedy, is_lod) in &track_query {
        let is_pressed = *interaction == Interaction::Pressed;
        if is_pressed {
            if is_fps.is_some() {
                *dragging_fps = true;
            }
            if is_dist.is_some() {
                *dragging_dist = true;
            }
            if is_greedy.is_some() {
                *dragging_greedy = true;
            }
            if is_lod.is_some() {
                *dragging_lod = true;
            }
        }

        let is_active_fps = is_fps.is_some() && (*dragging_fps || is_pressed);
        let is_active_dist = is_dist.is_some() && (*dragging_dist || is_pressed);
        let is_active_greedy = is_greedy.is_some() && (*dragging_greedy || is_pressed);
        let is_active_lod = is_lod.is_some() && (*dragging_lod || is_pressed);

        if (is_active_fps || is_active_dist || is_active_greedy || is_active_lod) && mouse_down {
            if let Some(norm) = rcp.normalized {
                let ratio = (norm.x + 0.5).clamp(0.0, 1.0);
                if is_active_fps {
                    settings.set_fps_cap_from_ratio(ratio);
                }
                if is_active_dist {
                    settings.set_view_distance_from_ratio(ratio);
                }
                if is_active_greedy {
                    settings.set_greedy_from_ratio(ratio);
                }
                if is_active_lod {
                    settings.set_lod_from_ratio(ratio);
                }
            }
        }
    }
}
