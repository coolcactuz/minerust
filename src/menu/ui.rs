use bevy::prelude::*;
use bevy::text::FontSize;

use super::types::{
    BenchmarkAvgFpsText, BenchmarkBannerText, BenchmarkChunksText, BenchmarkFrametimeText,
    BenchmarkMinMaxFrametimeText, BenchmarkOnePercentLowText, BenchmarkP99Text, BenchmarkRamText,
    BenchmarkResultsRoot, BenchmarkRunningBanner, BenchmarkSettingsText,
    BenchmarkVerdictDescText, BenchmarkVerdictTitleText, BenchmarkVertsText, BenchmarkVramText,
    DebugHudBtnText, DistanceFogBtnText, FpsCapBtnText, FpsCapFill, FpsCapThumb, FpsCapTrack,
    FullscreenBtnText, GraphicsGreedyBtnText, GraphicsGreedyFill, GraphicsGreedyThumb,
    GraphicsGreedyTrack, GraphicsLodBtnText, GraphicsLodFill, GraphicsLodThumb, GraphicsLodTrack,
    GraphicsSettings, MainMenuRoot, MenuButtonAction, PauseMenuRoot, ProfilerState, SeedInputBox,
    SeedInputState, SeedInputText, SettingsMenuRoot, ShadowsBtnText, ViewDistanceBtnText,
    ViewDistanceFill, ViewDistanceThumb, ViewDistanceTrack, VsyncBtnText,
};
use super::widgets::{
    spawn_menu_button, spawn_menu_button_sized, spawn_option_tooltip_card, spawn_settings_button,
    spawn_slider_setting,
};

pub fn setup_menu_ui(
    mut commands: Commands,
    graphics_settings: Option<Res<GraphicsSettings>>,
    profiler_state: Option<Res<ProfilerState>>,
    seed_state: Option<Res<SeedInputState>>,
) {
    let initial_seed_str = seed_state
        .as_ref()
        .map_or_else(|| "[Random]".to_string(), |s| s.seed_text.clone());

    spawn_main_menu(&mut commands, &initial_seed_str);
    spawn_pause_menu(&mut commands);
    spawn_settings_menu(
        &mut commands,
        graphics_settings.as_deref(),
        profiler_state.as_deref(),
    );
    spawn_benchmark_results_menu(&mut commands);
    spawn_benchmark_running_banner(&mut commands);
}

fn spawn_main_menu(commands: &mut Commands, initial_seed_str: &str) {
    let maybe_saved_world = crate::save::get_latest_saved_world();

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
            BackgroundColor(Color::srgb(0.06, 0.07, 0.10)),
            Visibility::Inherited,
            MainMenuRoot,
        ))
        .with_children(|parent| {
            // Title Header Panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("MINERUST"),
                        TextFont {
                            font_size: FontSize::Px(48.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    header.spawn((
                        Text::new("High-Performance Voxel Sandbox Engine"),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            // Action Buttons Panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|btn_col| {
                    // 1. Continue Saved World Button (if available)
                    let is_primary_new = if let Some(saved_seed) = maybe_saved_world {
                        let btn_label =
                            format!("Continue Saved World (Seed: {})", saved_seed);
                        spawn_menu_button(
                            btn_col,
                            &btn_label,
                            MenuButtonAction::ContinueGame,
                            true,
                        );
                        false
                    } else {
                        true
                    };

                    // 2. World Seed Customizer Row
                    btn_col
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            margin: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                            ..default()
                        })
                        .with_children(|row| {
                            // Clickable Seed Input Box
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(230.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.13, 0.18, 0.95)),
                                BorderColor::all(Color::srgba(0.35, 0.40, 0.55, 0.8)),
                                MenuButtonAction::ToggleEditSeed,
                                SeedInputBox,
                            ))
                            .with_children(|box_node| {
                                box_node.spawn((
                                    Text::new(format!("Seed: {}", initial_seed_str)),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.92, 0.96)),
                                    SeedInputText,
                                ));
                            });

                            // Dice / Randomize Seed Button
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(42.0),
                                    height: Val::Px(40.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
                                BorderColor::all(Color::srgba(0.45, 0.45, 0.55, 0.8)),
                                MenuButtonAction::RandomizeSeed,
                            ))
                            .with_children(|dice_btn| {
                                dice_btn.spawn((
                                    Text::new("🎲"),
                                    TextFont {
                                        font_size: FontSize::Px(16.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                                ));
                            });
                        });

                    spawn_menu_button(
                        btn_col,
                        if maybe_saved_world.is_some() {
                            "Start New World (Overwrite)"
                        } else {
                            "Start New World"
                        },
                        MenuButtonAction::NewGame,
                        is_primary_new,
                    );

                    // 3. Settings & Exit Buttons
                    spawn_menu_button(
                        btn_col,
                        "Graphics Settings",
                        MenuButtonAction::OpenSettings,
                        false,
                    );
                    spawn_menu_button(btn_col, "Quit Game", MenuButtonAction::QuitGame, false);
                });
        });
}

fn spawn_pause_menu(commands: &mut Commands) {
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
                    spawn_menu_button(btn_col, "Resume Game", MenuButtonAction::ResumeGame, true);
                    spawn_menu_button(
                        btn_col,
                        "Graphics Settings",
                        MenuButtonAction::OpenSettings,
                        false,
                    );
                    spawn_menu_button(
                        btn_col,
                        "Return to Main Menu",
                        MenuButtonAction::BackToMain,
                        false,
                    );
                });
        });
}

fn spawn_settings_menu(
    commands: &mut Commands,
    graphics_settings: Option<&GraphicsSettings>,
    profiler_state: Option<&ProfilerState>,
) {
    let (fps_label, fps_ratio) = graphics_settings.map_or_else(
        || ("FPS Limit: Uncapped (Max FPS)".to_string(), 1.0),
        |g| (g.fps_cap_label(), g.fps_cap_ratio()),
    );
    let (dist_label, dist_ratio) = graphics_settings.map_or_else(
        || ("Render Distance: 16 Chunks (256m)".to_string(), 6.0 / 15.0),
        |g| (g.view_distance_label(), g.view_distance_ratio()),
    );
    let (greedy_label, greedy_ratio) = graphics_settings.map_or_else(
        || ("Greedy Distance: > 2 Chunks (32m)".to_string(), 0.6),
        |g| (g.greedy_label(), g.greedy_ratio()),
    );
    let (lod_label, lod_ratio) = graphics_settings.map_or_else(
        || ("Distant Sloped LOD: > 8 Chunks (128m)".to_string(), 7.0 / 15.0),
        |g| (g.lod_label(), g.lod_ratio()),
    );
    let fog_label = graphics_settings.map_or_else(
        || "Distance Fog: ON (Blended)".to_string(),
        |g| {
            if g.distance_fog {
                "Distance Fog: ON (Blended)".to_string()
            } else {
                "Distance Fog: OFF (Harsh Edge)".to_string()
            }
        },
    );
    let shadows_label = graphics_settings.map_or_else(
        || "Shadows: ON (Dynamic)".to_string(),
        |g| {
            if g.shadows {
                "Shadows: ON (Dynamic)".to_string()
            } else {
                "Shadows: OFF (Flat)".to_string()
            }
        },
    );
    let hud_label = profiler_state.map_or_else(
        || "Profiler HUD (F3): OFF".to_string(),
        |p| {
            if p.visible {
                "Profiler HUD (F3): ON".to_string()
            } else {
                "Profiler HUD (F3): OFF".to_string()
            }
        },
    );

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
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("GRAPHICS SETTINGS"),
                        TextFont {
                            font_size: FontSize::Px(32.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    header.spawn((
                        Text::new(
                            "Configure display preferences, frame rate limits, fog, and visuals",
                        ),
                        TextFont {
                            font_size: FontSize::Px(13.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(24.0),
                    ..default()
                })
                .with_children(|row| {
                    // Controls Column
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|btn_col| {
                        // VSync Button
                        spawn_settings_button(
                            btn_col,
                            "VSync: ON (Smooth)",
                            MenuButtonAction::ToggleVsync,
                            VsyncBtnText,
                            320.0,
                            36.0,
                            13.5,
                        );

                        // Fullscreen Button
                        spawn_settings_button(
                            btn_col,
                            "Display: Windowed (1280x720)",
                            MenuButtonAction::ToggleFullscreen,
                            FullscreenBtnText,
                            320.0,
                            36.0,
                            13.5,
                        );

                        // Shadows Button
                        spawn_settings_button(
                            btn_col,
                            &shadows_label,
                            MenuButtonAction::ToggleShadows,
                            ShadowsBtnText,
                            320.0,
                            36.0,
                            13.5,
                        );

                        // Distance Fog Button
                        spawn_settings_button(
                            btn_col,
                            &fog_label,
                            MenuButtonAction::ToggleDistanceFog,
                            DistanceFogBtnText,
                            320.0,
                            36.0,
                            13.5,
                        );

                        // Real-time Profiler HUD Button (also F3 in-game)
                        spawn_settings_button(
                            btn_col,
                            &hud_label,
                            MenuButtonAction::ToggleDebugHud,
                            DebugHudBtnText,
                            320.0,
                            36.0,
                            13.5,
                        );

                        // FPS Cap Slider
                        spawn_slider_setting(
                            btn_col,
                            FpsCapBtnText,
                            FpsCapTrack,
                            FpsCapFill,
                            FpsCapThumb,
                            MenuButtonAction::StepFpsCapLeft,
                            MenuButtonAction::StepFpsCapRight,
                            MenuButtonAction::SlideFpsCap,
                            MenuButtonAction::CycleFpsCap,
                            &fps_label,
                            fps_ratio,
                            320.0,
                            46.0,
                        );

                        // Render Distance Slider
                        spawn_slider_setting(
                            btn_col,
                            ViewDistanceBtnText,
                            ViewDistanceTrack,
                            ViewDistanceFill,
                            ViewDistanceThumb,
                            MenuButtonAction::StepViewDistanceLeft,
                            MenuButtonAction::StepViewDistanceRight,
                            MenuButtonAction::SlideViewDistance,
                            MenuButtonAction::CycleViewDistance,
                            &dist_label,
                            dist_ratio,
                            320.0,
                            42.0,
                        );

                        // Greedy Meshing Distance Slider
                        spawn_slider_setting(
                            btn_col,
                            GraphicsGreedyBtnText,
                            GraphicsGreedyTrack,
                            GraphicsGreedyFill,
                            GraphicsGreedyThumb,
                            MenuButtonAction::StepGreedyMeshingLeft,
                            MenuButtonAction::StepGreedyMeshingRight,
                            MenuButtonAction::SlideGreedyMeshing,
                            MenuButtonAction::CycleGreedyMeshing,
                            &greedy_label,
                            greedy_ratio,
                            320.0,
                            42.0,
                        );

                        // Distant Sloped LOD Slider
                        spawn_slider_setting(
                            btn_col,
                            GraphicsLodBtnText,
                            GraphicsLodTrack,
                            GraphicsLodFill,
                            GraphicsLodThumb,
                            MenuButtonAction::StepDistanceLodLeft,
                            MenuButtonAction::StepDistanceLodRight,
                            MenuButtonAction::SlideDistanceLod,
                            MenuButtonAction::CycleDistanceLod,
                            &lod_label,
                            lod_ratio,
                            320.0,
                            42.0,
                        );

                        // Run Hardware Benchmark Button
                        spawn_menu_button_sized(
                            btn_col,
                            "⚡ Run Hardware Benchmark",
                            MenuButtonAction::StartBenchmark,
                            true,
                            320.0,
                            34.0,
                            14.0,
                        );

                        // Back Button
                        spawn_menu_button_sized(
                            btn_col,
                            "Back / Done",
                            MenuButtonAction::BackFromSettings,
                            false,
                            320.0,
                            34.0,
                            14.0,
                        );
                    });

                    // Right Info Popup Card
                    spawn_option_tooltip_card(row, 380.0, 380.0);
                });
        });
}

fn spawn_benchmark_results_menu(commands: &mut Commands) {
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
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.93)),
            Visibility::Hidden,
            BenchmarkResultsRoot,
        ))
        .with_children(|parent| {
            // Container modal
            parent
                .spawn((
                    Node {
                        width: Val::Px(840.0),
                        max_width: Val::Percent(96.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.08, 0.12, 0.98)),
                    BorderColor::all(Color::srgb(1.0, 0.85, 0.2)), // Gold border
                ))
                .with_children(|card| {
                    // Header title
                    card.spawn((
                        Text::new("BENCHMARK RESULTS & HARDWARE TELEMETRY"),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    card.spawn((
                        Text::new("Standardized 1,000m high-speed streaming trajectory flight on seed 'BENCHMARK'"),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                    ));

                    // Row with 3 Cards
                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    })
                    .with_children(|row| {
                        // Card 1: Framerate & Pacing
                        row.spawn((
                            Node {
                                width: Val::Px(256.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(12.0)),
                                row_gap: Val::Px(6.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.12, 0.13, 0.18, 0.9)),
                            BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.5)),
                        ))
                        .with_children(|c| {
                            c.spawn((
                                Text::new("[ FRAMERATE & PACING ]"),
                                TextFont { font_size: FontSize::Px(11.5), ..default() },
                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                            ));
                            c.spawn((
                                Text::new("Avg: -- FPS"),
                                TextFont { font_size: FontSize::Px(24.0), ..default() },
                                TextColor(Color::srgb(0.35, 1.0, 0.55)),
                                BenchmarkAvgFpsText,
                            ));
                            c.spawn((
                                Text::new("1% Low: -- FPS"),
                                TextFont { font_size: FontSize::Px(14.0), ..default() },
                                TextColor(Color::srgb(0.9, 0.95, 0.4)),
                                BenchmarkOnePercentLowText,
                            ));
                            c.spawn((
                                Text::new("99th %: -- ms"),
                                TextFont { font_size: FontSize::Px(12.0), ..default() },
                                TextColor(Color::srgb(0.8, 0.85, 0.9)),
                                BenchmarkP99Text,
                            ));
                            c.spawn((
                                Text::new("Frametime: -- ms"),
                                TextFont { font_size: FontSize::Px(12.0), ..default() },
                                TextColor(Color::srgb(0.8, 0.85, 0.9)),
                                BenchmarkFrametimeText,
                            ));
                            c.spawn((
                                Text::new("Min/Max: -- / --"),
                                TextFont { font_size: FontSize::Px(11.5), ..default() },
                                TextColor(Color::srgb(0.65, 0.7, 0.8)),
                                BenchmarkMinMaxFrametimeText,
                            ));
                        });

                        // Card 2: Memory & Hardware
                        row.spawn((
                            Node {
                                width: Val::Px(256.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(12.0)),
                                row_gap: Val::Px(6.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.12, 0.13, 0.18, 0.9)),
                            BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.5)),
                        ))
                        .with_children(|c| {
                            c.spawn((
                                Text::new("[ MEMORY & HARDWARE ]"),
                                TextFont { font_size: FontSize::Px(11.5), ..default() },
                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                            ));
                            c.spawn((
                                Text::new("RAM (RSS): -- MB"),
                                TextFont { font_size: FontSize::Px(14.5), ..default() },
                                TextColor(Color::srgb(0.4, 0.85, 1.0)),
                                BenchmarkRamText,
                            ));
                            c.spawn((
                                Text::new("VRAM: -- MB"),
                                TextFont { font_size: FontSize::Px(14.5), ..default() },
                                TextColor(Color::srgb(0.4, 0.85, 1.0)),
                                BenchmarkVramText,
                            ));
                            c.spawn((
                                Text::new("Active Chunks: --"),
                                TextFont { font_size: FontSize::Px(12.0), ..default() },
                                TextColor(Color::srgb(0.8, 0.85, 0.9)),
                                BenchmarkChunksText,
                            ));
                            c.spawn((
                                Text::new("Peak Geometry: --"),
                                TextFont { font_size: FontSize::Px(12.0), ..default() },
                                TextColor(Color::srgb(0.8, 0.85, 0.9)),
                                BenchmarkVertsText,
                            ));
                            c.spawn((
                                Text::new("Flight: 1,000m @ 50m/s"),
                                TextFont { font_size: FontSize::Px(11.5), ..default() },
                                TextColor(Color::srgb(0.65, 0.7, 0.8)),
                            ));
                        });

                        // Card 3: Evaluated Settings
                        row.spawn((
                            Node {
                                width: Val::Px(256.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(12.0)),
                                row_gap: Val::Px(6.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.12, 0.13, 0.18, 0.9)),
                            BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.5)),
                        ))
                        .with_children(|c| {
                            c.spawn((
                                Text::new("[ TESTED SETTINGS ]"),
                                TextFont { font_size: FontSize::Px(11.5), ..default() },
                                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                            ));
                            c.spawn((
                                Text::new("Loading settings..."),
                                TextFont { font_size: FontSize::Px(12.0), ..default() },
                                TextColor(Color::srgb(0.85, 0.9, 0.95)),
                                BenchmarkSettingsText,
                            ));
                        });
                    });

                    // Verdict Card
                    card.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.12, 0.18, 0.95)),
                        BorderColor::all(Color::srgb(0.35, 0.75, 1.0)),
                    ))
                    .with_children(|v| {
                        v.spawn((
                            Text::new("[ HARDWARE VERDICT ] Analyzing..."),
                            TextFont { font_size: FontSize::Px(14.0), ..default() },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                            BenchmarkVerdictTitleText,
                        ));
                        v.spawn((
                            Text::new("Optimization advice will appear here once the benchmark completes."),
                            TextFont { font_size: FontSize::Px(12.5), ..default() },
                            TextColor(Color::srgb(0.85, 0.9, 0.95)),
                            BenchmarkVerdictDescText,
                        ));
                    });

                    // Action Buttons Row
                    card.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    })
                    .with_children(|btn_row| {
                        spawn_menu_button_sized(
                            btn_row,
                            "⚡ Rerun Benchmark",
                            MenuButtonAction::StartBenchmark,
                            true,
                            220.0,
                            36.0,
                            13.5,
                        );
                        spawn_menu_button_sized(
                            btn_row,
                            "Adjust Graphics Settings",
                            MenuButtonAction::BackFromBenchmark,
                            false,
                            230.0,
                            36.0,
                            13.5,
                        );
                        spawn_menu_button_sized(
                            btn_row,
                            "Return to Main Menu",
                            MenuButtonAction::BackToMain,
                            false,
                            200.0,
                            36.0,
                            13.5,
                        );
                    });
                });
        });
}

fn spawn_benchmark_running_banner(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Px(16.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            BenchmarkRunningBanner,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.04, 0.05, 0.09, 0.92)),
                    BorderColor::all(Color::srgb(1.0, 0.85, 0.2)),
                ))
                .with_children(|box_node| {
                    box_node.spawn((
                        Text::new("[ BENCHMARK IN PROGRESS ] Distance: 0m / 1,000m (0%) | Press [ESC] to Cancel"),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.35, 1.0, 0.55)),
                        BenchmarkBannerText,
                    ));
                });
        });
}
