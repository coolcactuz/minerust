use bevy::prelude::*;
use bevy::text::FontSize;

use super::types::{
    AsyncMeshingBtnText, BackfaceCullingBtnText, DebugHudBtnText, DevSettings, DevSettingsMenuRoot,
    DistanceFogBtnText, DistanceLodBtnText, FpsCapBtnText, FpsCapFill, FpsCapThumb, FpsCapTrack,
    FullscreenBtnText, GraphicsGreedyBtnText, GraphicsGreedyFill, GraphicsGreedyThumb,
    GraphicsGreedyTrack, GraphicsLodBtnText, GraphicsLodFill, GraphicsLodThumb, GraphicsLodTrack,
    GraphicsSettings, GreedyMeshingBtnText, MainMenuRoot, MaxYSkipBtnText, MenuButtonAction,
    MeshBudgetBtnText, PauseMenuRoot, PregenMarginBtnText, SeedInputBox, SeedInputState,
    SeedInputText, SettingsMenuRoot, ShadowsBtnText, ViewDistanceBtnText, ViewDistanceFill,
    ViewDistanceThumb, ViewDistanceTrack, VsyncBtnText,
};
use super::widgets::{
    spawn_menu_button, spawn_menu_button_sized, spawn_option_tooltip_card, spawn_settings_button,
    spawn_slider_setting,
};

pub fn setup_menu_ui(
    mut commands: Commands,
    dev_settings: Option<Res<DevSettings>>,
    graphics_settings: Option<Res<GraphicsSettings>>,
    seed_state: Option<Res<SeedInputState>>,
) {
    let is_dev = dev_settings.as_ref().is_some_and(|d| d.dev_mode);
    let initial_seed_str = seed_state
        .as_ref()
        .map_or_else(|| "[Random]".to_string(), |s| s.seed_text.clone());

    spawn_main_menu(&mut commands, is_dev, &initial_seed_str);
    spawn_pause_menu(&mut commands, is_dev);
    spawn_settings_menu(&mut commands, graphics_settings.as_deref());
    spawn_dev_settings_menu(&mut commands);
}

fn spawn_main_menu(commands: &mut Commands, is_dev: bool, initial_seed_str: &str) {
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
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("MINERUST"),
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

            // Seed Configuration Panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(24.0)),
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|seed_col| {
                    seed_col.spawn((
                        Text::new("WORLD GENERATION SEED"),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.75, 1.0)),
                    ));

                    seed_col
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|seed_row| {
                            seed_row
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(38.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.5)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.1, 0.12, 0.18, 0.95)),
                                    BorderColor::all(Color::srgba(0.35, 0.4, 0.55, 0.8)),
                                    SeedInputBox,
                                    MenuButtonAction::ToggleEditSeed,
                                ))
                                .with_children(|input_box| {
                                    input_box.spawn((
                                        Text::new(if initial_seed_str.is_empty() {
                                            "Seed: [Random Seed]".to_string()
                                        } else {
                                            format!("Seed: {}", initial_seed_str)
                                        }),
                                        TextFont {
                                            font_size: FontSize::Px(13.0),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                        SeedInputText,
                                    ));
                                });

                            seed_row
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(110.0),
                                        height: Val::Px(38.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.18, 0.25, 0.38, 0.9)),
                                    BorderColor::all(Color::srgba(0.3, 0.5, 0.8, 0.8)),
                                    MenuButtonAction::RandomizeSeed,
                                ))
                                .with_children(|rand_parent| {
                                    rand_parent.spawn((
                                        Text::new("Random Seed"),
                                        TextFont {
                                            font_size: FontSize::Px(13.5),
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                    ));
                                });
                        });
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
                    spawn_menu_button(btn_col, "Play Game", MenuButtonAction::Play, true);
                    spawn_menu_button(
                        btn_col,
                        "Graphics Settings",
                        MenuButtonAction::OpenSettings,
                        false,
                    );
                    if is_dev {
                        spawn_menu_button(
                            btn_col,
                            "Dev & Benchmarks",
                            MenuButtonAction::OpenDevSettings,
                            false,
                        );
                    }
                    spawn_menu_button(btn_col, "Quit Game", MenuButtonAction::QuitGame, false);
                });
        });
}

fn spawn_pause_menu(commands: &mut Commands, is_dev: bool) {
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
                    if is_dev {
                        spawn_menu_button(
                            btn_col,
                            "Dev & Benchmarks",
                            MenuButtonAction::OpenDevSettings,
                            false,
                        );
                    }
                    spawn_menu_button(
                        btn_col,
                        "Return to Main Menu",
                        MenuButtonAction::BackToMain,
                        false,
                    );
                });
        });
}

fn spawn_settings_menu(commands: &mut Commands, graphics_settings: Option<&GraphicsSettings>) {
    let (fps_label, fps_ratio) = graphics_settings.map_or_else(
        || ("FPS Limit: Uncapped (Max FPS)".to_string(), 1.0),
        |g| (g.fps_cap_label(), g.fps_cap_ratio()),
    );
    let (dist_label, dist_ratio) = graphics_settings.map_or_else(
        || ("Render Distance: 16 Chunks (256m)".to_string(), 6.0 / 15.0),
        |g| (g.view_distance_label(), g.view_distance_ratio()),
    );
    let (greedy_label, greedy_ratio) = graphics_settings.map_or_else(
        || ("Greedy Distance: > 2 Chunks (32m)".to_string(), 0.2),
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
                            "Configure display preferences, frame rate limits, fog, and visual detail",
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

                        // Distance Fog Button (Moved from Dev Settings)
                        spawn_settings_button(
                            btn_col,
                            &fog_label,
                            MenuButtonAction::ToggleDistanceFog,
                            DistanceFogBtnText,
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
                            46.0,
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
                            46.0,
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
                            46.0,
                        );

                        // Back Button
                        spawn_menu_button_sized(
                            btn_col,
                            "Back / Done",
                            MenuButtonAction::BackFromSettings,
                            true,
                            320.0,
                            38.0,
                            14.5,
                        );
                    });

                    // Right Info Popup Card
                    spawn_option_tooltip_card(row, 380.0, 380.0);
                });
        });
}

fn spawn_dev_settings_menu(commands: &mut Commands) {
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
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("DEV & BENCHMARK SETTINGS"),
                        TextFont {
                            font_size: FontSize::Px(30.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.9, 1.0)),
                    ));
                    header.spawn((
                        Text::new(
                            "Benchmark real-time performance impacts of each optimization technique",
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
                    column_gap: Val::Px(18.0),
                    ..default()
                })
                .with_children(|row| {
                    // Column 1 (5 buttons - Distance Fog moved to Graphics Settings)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|col1| {
                        spawn_settings_button(
                            col1,
                            "Backface Culling: ON",
                            MenuButtonAction::ToggleBackfaceCulling,
                            BackfaceCullingBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col1,
                            "Dynamic Shadows: ON",
                            MenuButtonAction::ToggleShadows,
                            ShadowsBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col1,
                            "Mesher max_y Skip: ON",
                            MenuButtonAction::ToggleMaxYSkip,
                            MaxYSkipBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col1,
                            "Mesh Budget: ON (6/frame)",
                            MenuButtonAction::ToggleMeshBudget,
                            MeshBudgetBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col1,
                            "Async Meshing: ON (0ms main thread)",
                            MenuButtonAction::ToggleAsyncMeshing,
                            AsyncMeshingBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                    });

                    // Column 2 (5 buttons - LOD Distance moved to Graphics Settings)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|col2| {
                        spawn_settings_button(
                            col2,
                            "Greedy Meshing: ON (-75% verts)",
                            MenuButtonAction::ToggleGreedyMeshing,
                            GreedyMeshingBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col2,
                            "Distance LOD: ON (Dynamic detail)",
                            MenuButtonAction::ToggleDistanceLod,
                            DistanceLodBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col2,
                            "Lookahead Buffer: 2 Chunks (+32m)",
                            MenuButtonAction::CyclePregenMargin,
                            PregenMarginBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_settings_button(
                            col2,
                            "Dev HUD (F3): ON",
                            MenuButtonAction::ToggleDebugHud,
                            DebugHudBtnText,
                            280.0,
                            44.0,
                            13.5,
                        );
                        spawn_menu_button_sized(
                            col2,
                            "Back / Done",
                            MenuButtonAction::BackFromDevSettings,
                            true,
                            280.0,
                            44.0,
                            14.5,
                        );
                    });

                    // Right Info Popup Card
                    spawn_option_tooltip_card(row, 370.0, 252.0);
                });
        });
}

