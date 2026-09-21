use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, WindowMode,
};

use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::physics::PlayerPhysics;
use crate::save::load_player_from_disk;
use crate::world::{SEA_LEVEL, WorldGrid, WorldSeed, calculate_biome_and_height};

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
    pub dev_mode: bool,
    pub backface_culling: bool,
    pub shadows_enabled: bool,
    pub max_y_skip: bool,
    pub distance_fog: bool,
    pub mesh_budget: bool,
    pub async_meshing: bool,
    pub greedy_meshing: bool,
    pub distance_lod: bool,
    pub lod_threshold: i32,
    pub show_debug_hud: bool,
    pub pregen_margin: i32,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            dev_mode: false,
            backface_culling: true,
            shadows_enabled: true,
            max_y_skip: true,
            distance_fog: true,
            mesh_budget: true,
            async_meshing: true,
            greedy_meshing: true,
            distance_lod: true,
            lod_threshold: 4,
            show_debug_hud: false,
            pregen_margin: 2,
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

#[derive(Resource, Debug, Clone, Default)]
pub struct SeedInputState {
    pub seed_text: String,
    pub is_editing: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuButtonAction {
    Play,
    ResumeGame,
    OpenSettings,
    OpenDevSettings,
    BackFromSettings,
    BackFromDevSettings,
    BackToMain,
    QuitGame,
    ToggleEditSeed,
    RandomizeSeed,
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
    ToggleDebugHud,
    CyclePregenMargin,
}

#[derive(Component)]
pub struct SeedInputBox;

#[derive(Component)]
pub struct SeedInputText;

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
pub struct DebugHudBtnText;

#[derive(Component)]
pub struct PregenMarginBtnText;

#[derive(Component)]
pub struct OptionTooltipCard;

#[derive(Component)]
pub struct OptionTooltipHeader;

#[derive(Component)]
pub struct OptionTooltipTitle;

#[derive(Component)]
pub struct OptionTooltipDesc;

#[derive(Component)]
pub struct OptionTooltipImpact;

#[derive(Clone, Copy, Debug)]
pub struct OptionDescription {
    pub header: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub impact: &'static str,
}

#[must_use]
pub const fn get_option_description(action: &MenuButtonAction) -> Option<OptionDescription> {
    match action {
        MenuButtonAction::ToggleVsync => Some(OptionDescription {
            header: "DISPLAY & SYNC",
            title: "Vertical Synchronization (VSync)",
            description: "Synchronizes the game's rendered frame rate with your monitor's physical refresh rate to prevent screen tearing.",
            impact: "• ON: Smooth frame pacing, zero screen tearing.\n• OFF: Lowest input latency, uncapped frame rate.",
        }),
        MenuButtonAction::ToggleFullscreen => Some(OptionDescription {
            header: "DISPLAY MODE",
            title: "Display Mode (Fullscreen / Windowed)",
            description: "Switches between Borderless Fullscreen (native monitor resolution) and Windowed mode (1280x720).",
            impact: "• Fullscreen: Immersive edge-to-edge display.\n• Windowed: Convenient multitasking and window positioning.",
        }),
        MenuButtonAction::CycleFpsCap => Some(OptionDescription {
            header: "PERFORMANCE & THERMALS",
            title: "Frame Rate Limiter",
            description: "Limits maximum frames rendered per second (Uncapped, 60, 120, 144 FPS) using microsecond sleep pacing.",
            impact: "• Capping FPS significantly lowers GPU temperature, power consumption, and fan noise.",
        }),
        MenuButtonAction::CycleViewDistance => Some(OptionDescription {
            header: "WORLD GENERATION & RENDER RADIUS",
            title: "Render Distance",
            description: "Sets the horizontal radius of chunks loaded and rendered around the player (8 to 64 chunks = 128m to 1024m).",
            impact: "• 16 Chunks (256m): Recommended balance of horizon view and performance.\n• 32-64 Chunks: Sweeping vistas; higher RAM/VRAM load.",
        }),
        MenuButtonAction::ToggleBackfaceCulling => Some(OptionDescription {
            header: "GPU PIPELINE BENCHMARK",
            title: "Backface Culling",
            description: "Discards triangles facing away from the camera in the GPU rasterizer. Solid voxel blocks never expose interior faces.",
            impact: "• ON: Cuts rasterizer fragment load and fill-rate by ~50%.\n• OFF: Forces GPU to rasterize front and back faces of every quad.",
        }),
        MenuButtonAction::ToggleShadows => Some(OptionDescription {
            header: "LIGHTING & SHADOWS",
            title: "Dynamic Cascaded Shadows",
            description: "Toggles real-time directional sunlight shadow cascades spanning up to 120 meters from the camera.",
            impact: "• ON: Realistic depth, tree canopy shadows, and terrain self-shadowing.\n• OFF: Skips shadow passes, yielding a large FPS boost on iGPUs.",
        }),
        MenuButtonAction::ToggleMaxYSkip => Some(OptionDescription {
            header: "MESHING BENCHMARK",
            title: "Mesher max_y Air Skipping",
            description: "Tracks the highest solid block per chunk during generation, allowing the mesher to skip empty sky layers up to Y=384.",
            impact: "• ON: ~2x faster chunk meshing, preventing CPU stutters.\n• OFF: Scans all 384 vertical Y layers even if 250 are empty sky.",
        }),
        MenuButtonAction::ToggleDistanceFog => Some(OptionDescription {
            header: "ATMOSPHERE & BLENDING",
            title: "Distance Fog",
            description: "Applies linear atmospheric distance fog that gracefully blends distant terrain into the sky before chunk boundaries.",
            impact: "• ON: Smooth, immersive horizon that hides chunk loading boundaries.\n• OFF: Sharp cutoff edge at the boundary of loaded chunks.",
        }),
        MenuButtonAction::ToggleMeshBudget => Some(OptionDescription {
            header: "FRAME PACING BENCHMARK",
            title: "Frame Mesh Upload Budget",
            description: "Limits GPU buffer uploads of newly meshed chunks to a maximum of 6 chunks per frame.",
            impact: "• ON: Smooth, consistent frame times when flying rapidly.\n• OFF: Uploads all meshes simultaneously, causing micro-stutters.",
        }),
        MenuButtonAction::ToggleAsyncMeshing => Some(OptionDescription {
            header: "MULTITHREADING BENCHMARK",
            title: "Async Multi-Threaded Meshing",
            description: "Dispatches chunk greedy meshing computations to background worker threads across all available CPU cores.",
            impact: "• ON: Zero main-thread lag (0ms) during terrain meshing.\n• OFF: Synchronous meshing on the render thread, causing frame drops.",
        }),
        MenuButtonAction::ToggleGreedyMeshing => Some(OptionDescription {
            header: "GEOMETRY OPTIMIZATION BENCHMARK",
            title: "Greedy Meshing Algorithm",
            description: "Iteratively merges adjacent coplanar block faces sharing the same voxel type into large single rectangular quads.",
            impact: "• ON: Reduces chunk vertex and triangle counts by ~75%.\n• OFF: Emits separate 1x1 quads for every exposed block face.",
        }),
        MenuButtonAction::ToggleDistanceLod => Some(OptionDescription {
            header: "LOD BENCHMARK",
            title: "Distance Level of Detail (LOD)",
            description: "Dynamically switches chunk meshing to a 2x2 simplified voxel grid for chunks located beyond the LOD threshold distance.",
            impact: "• ON: Reduces distant geometry complexity by another 50-75%.\n• OFF: Renders distant chunks with uniform 1:1 full-resolution geometry.",
        }),
        MenuButtonAction::CycleLodThreshold => Some(OptionDescription {
            header: "LOD DISTANCE TUNING",
            title: "LOD Distance Threshold",
            description: "Distance in chunks (2, 4, 6, 8 chunks = 32m to 128m) at which chunk geometry transitions to simplified Level 2 LOD.",
            impact: "• Shorter distance = higher frame rates at the cost of closer visual simplification.\n• Longer distance = full detail preserved further out.",
        }),
        MenuButtonAction::CyclePregenMargin => Some(OptionDescription {
            header: "MEMORY & STREAMING BENCHMARK",
            title: "Lookahead Pregen Buffer",
            description: "Pre-calculates chunk voxel data in RAM just outside the camera's visual view distance (0 to 4 chunks = 0m to 64m margin).",
            impact: "• Eliminates pop-in stutter when walking forward.\n• 0 Chunks: Disabled (benchmark raw generation latency).\n• 2-4 Chunks: Seamless walking buffer.",
        }),
        MenuButtonAction::ToggleDebugHud => Some(OptionDescription {
            header: "DIAGNOSTICS",
            title: "Debug Diagnostics Overlay (F3)",
            description: "Displays in-game real-time FPS, frame timing, player coordinates, active chunk count, triangle counts, and biome data.",
            impact: "• Essential for profiling performance impacts while playing.\n• Can also be toggled anytime in-game with the F3 key.",
        }),
        MenuButtonAction::BackFromSettings | MenuButtonAction::BackFromDevSettings => {
            Some(OptionDescription {
                header: "NAVIGATION",
                title: "◀ Return / Done",
                description: "Save configuration changes and return to the previous menu screen.",
                impact: "• All graphical and benchmark adjustments apply immediately in real-time.",
            })
        }
        MenuButtonAction::Play => Some(OptionDescription {
            header: "GAMEPLAY",
            title: "▶ Play Game",
            description: "Generate or load the voxel world and enter gameplay.",
            impact: "• Uses the current world seed and graphics settings.",
        }),
        MenuButtonAction::ResumeGame => Some(OptionDescription {
            header: "GAMEPLAY",
            title: "▶ Resume Game",
            description: "Unpause and return to the active game world.",
            impact: "• Restores mouse capture and camera controls.",
        }),
        MenuButtonAction::OpenSettings => Some(OptionDescription {
            header: "CONFIGURATION",
            title: "⚙ Graphics Settings",
            description: "Configure display mode, VSync, frame rate limit, and view distance.",
            impact: "• Adjust visuals and performance for your hardware.",
        }),
        MenuButtonAction::OpenDevSettings => Some(OptionDescription {
            header: "BENCHMARK TOOLS",
            title: "🛠 Dev & Benchmark Settings",
            description: "Toggle internal engine optimizations to measure performance impacts.",
            impact: "• Available only in Developer mode.",
        }),
        MenuButtonAction::BackToMain => Some(OptionDescription {
            header: "NAVIGATION",
            title: "⌂ Return to Main Menu",
            description: "Save player data and chunk modifications to disk, then return to the main title screen.",
            impact: "• World progress is safely saved.",
        }),
        MenuButtonAction::QuitGame => Some(OptionDescription {
            header: "NAVIGATION",
            title: "✕ Quit Game",
            description: "Close MineRust and return to desktop.",
            impact: "• All world modifications and player inventory are saved.",
        }),
        MenuButtonAction::ToggleEditSeed => Some(OptionDescription {
            header: "WORLD GENERATION",
            title: "Custom World Seed Input",
            description: "Type any alphanumeric string or number to generate a unique procedural world.",
            impact: "• Supports full alphanumeric seed strings.",
        }),
        MenuButtonAction::RandomizeSeed => Some(OptionDescription {
            header: "WORLD GENERATION",
            title: "🎲 Randomize World Seed",
            description: "Generates a fresh random 64-bit seed using high-resolution entropy.",
            impact: "• Each click generates a brand new terrain layout.",
        }),
    }
}

pub fn setup_menu_ui(
    mut commands: Commands,
    dev_settings: Option<Res<DevSettings>>,
    seed_state: Option<Res<SeedInputState>>,
) {
    let is_dev = dev_settings.as_ref().is_some_and(|d| d.dev_mode);
    let initial_seed_str = seed_state
        .as_ref()
        .map_or_else(|| "[Random]".to_string(), |s| s.seed_text.clone());

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
                    margin: UiRect::bottom(Val::Px(30.0)),
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
                        Text::new("WORLD SEED (CLICK TO EDIT / TYPE):"),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                    ));

                    seed_col
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Seed text box button
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(260.0),
                                    height: Val::Px(38.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.14, 0.20, 0.9)),
                                BorderColor::all(Color::srgba(0.35, 0.4, 0.55, 0.8)),
                                MenuButtonAction::ToggleEditSeed,
                                SeedInputBox,
                            ))
                            .with_children(|box_parent| {
                                box_parent.spawn((
                                    Text::new(format!("Seed: {initial_seed_str}")),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.95, 0.95, 0.95)),
                                    SeedInputText,
                                ));
                            });

                            // Randomize Seed Button
                            row.spawn((
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
                                    Text::new("🎲 Random"),
                                    TextFont {
                                        font_size: FontSize::Px(14.0),
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
                    spawn_menu_button(btn_col, "▶ Play Game", MenuButtonAction::Play, true);
                    spawn_menu_button(
                        btn_col,
                        "⚙ Graphics Settings",
                        MenuButtonAction::OpenSettings,
                        false,
                    );
                    if is_dev {
                        spawn_menu_button(
                            btn_col,
                            "🛠 Dev & Benchmarks",
                            MenuButtonAction::OpenDevSettings,
                            false,
                        );
                    }
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
                    spawn_menu_button(
                        btn_col,
                        "⚙ Graphics Settings",
                        MenuButtonAction::OpenSettings,
                        false,
                    );
                    if is_dev {
                        spawn_menu_button(
                            btn_col,
                            "🛠 Dev & Benchmarks",
                            MenuButtonAction::OpenDevSettings,
                            false,
                        );
                    }
                    spawn_menu_button(
                        btn_col,
                        "⌂ Return to Main Menu",
                        MenuButtonAction::BackToMain,
                        false,
                    );
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
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("GRAPHICS SETTINGS"),
                        TextFont {
                            font_size: FontSize::Px(34.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    header.spawn((
                        Text::new(
                            "Configure display preferences, frame rate limits, and visual distance",
                        ),
                        TextFont {
                            font_size: FontSize::Px(14.0),
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
                    column_gap: Val::Px(28.0),
                    ..default()
                })
                .with_children(|row| {
                    // Buttons Column
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
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
                            48.0,
                            15.0,
                        );

                        // Fullscreen Button
                        spawn_settings_button(
                            btn_col,
                            "Display: Windowed (1280x720)",
                            MenuButtonAction::ToggleFullscreen,
                            FullscreenBtnText,
                            320.0,
                            48.0,
                            15.0,
                        );

                        // FPS Cap Button
                        spawn_settings_button(
                            btn_col,
                            "FPS Limit: Uncapped",
                            MenuButtonAction::CycleFpsCap,
                            FpsCapBtnText,
                            320.0,
                            48.0,
                            15.0,
                        );

                        // Render Distance Button
                        spawn_settings_button(
                            btn_col,
                            "Render Distance: 16 Chunks",
                            MenuButtonAction::CycleViewDistance,
                            ViewDistanceBtnText,
                            320.0,
                            48.0,
                            15.0,
                        );

                        // Back Button
                        spawn_menu_button_sized(
                            btn_col,
                            "◀ Back / Done",
                            MenuButtonAction::BackFromSettings,
                            true,
                            320.0,
                            48.0,
                            16.0,
                        );
                    });

                    // Right Info Popup Card
                    spawn_option_tooltip_card(row, 380.0, 288.0);
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
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("🛠 DEV & BENCHMARK SETTINGS"),
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
                    // Column 1 (6 buttons)
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
                            "Distance Fog: ON",
                            MenuButtonAction::ToggleDistanceFog,
                            DistanceFogBtnText,
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

                    // Column 2 (6 buttons)
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
                            "LOD Distance: 4 Chunks (64m)",
                            MenuButtonAction::CycleLodThreshold,
                            LodThresholdBtnText,
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
                            "◀ Back / Done",
                            MenuButtonAction::BackFromDevSettings,
                            true,
                            280.0,
                            44.0,
                            14.5,
                        );
                    });

                    // Right Info Popup Card
                    spawn_option_tooltip_card(row, 370.0, 304.0);
                });
        });
}

fn spawn_option_tooltip_card(parent: &mut ChildSpawnerCommands, width_px: f32, height_px: f32) {
    parent
        .spawn((
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.09, 0.15, 0.95)),
            BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8)),
            OptionTooltipCard,
        ))
        .with_children(|card| {
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|top| {
                top.spawn((
                    Text::new("[ ℹ️ SETTING INFO POPUP ]"),
                    TextFont {
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.35, 0.8, 1.0)),
                    OptionTooltipHeader,
                ));

                top.spawn((
                    Text::new("Hover over any setting"),
                    TextFont {
                        font_size: FontSize::Px(16.5),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    OptionTooltipTitle,
                ));

                top.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.5),
                        margin: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.4, 0.5, 0.7, 0.35)),
                ));

                top.spawn((
                    Text::new(
                        "Move your mouse over any graphic or performance setting on the left to inspect its technical details, rendering behavior, and performance impact.",
                    ),
                    TextFont {
                        font_size: FontSize::Px(12.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.88, 0.93)),
                    OptionTooltipDesc,
                ));
            });

            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.16, 0.25, 0.85)),
                BorderColor::all(Color::srgba(0.3, 0.5, 0.7, 0.5)),
            ))
            .with_children(|impact_box| {
                impact_box.spawn((
                    Text::new("• All MineRust optimizations are tuned for maximum 60+ FPS stability."),
                    TextFont {
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.95, 0.65)),
                    OptionTooltipImpact,
                ));
            });
        });
}

fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    highlight: bool,
) {
    spawn_menu_button_sized(parent, label, action, highlight, 320.0, 50.0, 16.0);
}

fn spawn_menu_button_sized(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    highlight: bool,
    width_px: f32,
    height_px: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
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
                    font_size: FontSize::Px(font_size),
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
    width_px: f32,
    height_px: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
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
                    font_size: FontSize::Px(font_size),
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
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::F3) {
        if let Some(ref mut dev) = dev_settings {
            if dev.dev_mode {
                dev.show_debug_hud = !dev.show_debug_hud;
            }
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

fn keycode_to_char(key: KeyCode, shift: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Numpad0 => Some('0'),
        KeyCode::Numpad1 => Some('1'),
        KeyCode::Numpad2 => Some('2'),
        KeyCode::Numpad3 => Some('3'),
        KeyCode::Numpad4 => Some('4'),
        KeyCode::Numpad5 => Some('5'),
        KeyCode::Numpad6 => Some('6'),
        KeyCode::Numpad7 => Some('7'),
        KeyCode::Numpad8 => Some('8'),
        KeyCode::Numpad9 => Some('9'),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

pub fn update_seed_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<MenuState>,
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut box_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let Some(ref mut state) = seed_state else {
        return;
    };

    if menu.screen != MenuScreen::Main {
        if state.is_editing {
            state.is_editing = false;
        }
        return;
    }

    let mut state_changed = false;

    if state.is_editing {
        if keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || keys.just_pressed(KeyCode::Escape)
        {
            state.is_editing = false;
            state_changed = true;
        } else if keys.just_pressed(KeyCode::Backspace) {
            state.seed_text.pop();
            state_changed = true;
        } else {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            for &key in keys.get_just_pressed() {
                if let Some(ch) = keycode_to_char(key, shift) {
                    if state.seed_text.len() < 32 {
                        state.seed_text.push(ch);
                        state_changed = true;
                    }
                }
            }
        }
    }

    if state_changed || state.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            *text = Text::new(if state.is_editing {
                if state.seed_text.is_empty() {
                    "Seed: ▌".to_string()
                } else {
                    format!("Seed: {}▌", state.seed_text)
                }
            } else if state.seed_text.is_empty() {
                "Seed: [Random Seed]".to_string()
            } else {
                format!("Seed: {}", state.seed_text)
            });
        }

        if let Ok(mut border) = box_query.single_mut() {
            *border = if state.is_editing {
                BorderColor::all(Color::srgb(0.2, 0.8, 1.0))
            } else {
                BorderColor::all(Color::srgba(0.35, 0.4, 0.55, 0.8))
            };
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
                                            let (_, spawn_y, _) =
                                                calculate_biome_and_height(0.0, 0.0, &w.noise);
                                            let player_y =
                                                (spawn_y as f32 + 4.0).max((SEA_LEVEL + 4) as f32);
                                            if let Some(ref mut inv) = inventory {
                                                **inv = Inventory::default();
                                            }
                                            (
                                                Vec3::new(0.0, player_y, 0.0),
                                                -std::f32::consts::FRAC_PI_2,
                                                -0.3,
                                            )
                                        }
                                    } else {
                                        let (_, spawn_y, _) =
                                            calculate_biome_and_height(0.0, 0.0, &w.noise);
                                        let player_y =
                                            (spawn_y as f32 + 4.0).max((SEA_LEVEL + 4) as f32);
                                        if let Some(ref mut inv) = inventory {
                                            **inv = Inventory::default();
                                        }
                                        (
                                            Vec3::new(0.0, player_y, 0.0),
                                            -std::f32::consts::FRAC_PI_2,
                                            -0.3,
                                        )
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
                                    format!("MineRust ⛏️🦀 [DEV MODE] - Seed: {}", target_seed.0)
                                } else {
                                    format!("MineRust ⛏️🦀 - Seed: {}", target_seed.0)
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
                    *text = Text::new(format!("[ ℹ️ {} ]", desc.header));
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
                *text = Text::new("[ ℹ️ SETTING INFO POPUP ]");
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
                    "• All MineRust optimizations are tuned for maximum 60+ FPS stability.",
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

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<GraphicsSettings>()
            .init_resource::<DevSettings>()
            .init_resource::<FpsLimiter>()
            .init_resource::<SeedInputState>()
            .add_systems(Startup, setup_menu_ui)
            .add_systems(
                Update,
                (
                    menu_input_system.in_set(crate::stage::VoxelStage::InputHandling),
                    update_seed_input_system.in_set(crate::stage::VoxelStage::InputHandling),
                    update_menu_visibility_system,
                    menu_button_hover_system,
                    menu_button_click_system,
                    update_settings_button_text_system,
                    update_dev_button_text_system,
                    update_dev_settings_system,
                    update_option_tooltip_system,
                    fps_limiter_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_settings_production_defaults() {
        let dev = DevSettings::default();

        // Production mode must be default
        assert!(
            !dev.dev_mode,
            "dev_mode must default to false for public release"
        );
        assert!(
            !dev.show_debug_hud,
            "debug HUD must default to false for public release"
        );

        // All performance optimizations must be locked ON by default
        assert!(dev.backface_culling, "backface culling must be enabled");
        assert!(dev.shadows_enabled, "shadows must be enabled");
        assert!(dev.max_y_skip, "max_y skip must be enabled");
        assert!(dev.distance_fog, "distance fog must be enabled");
        assert!(dev.mesh_budget, "mesh budget must be enabled");
        assert!(dev.async_meshing, "async meshing must be enabled");
        assert!(dev.greedy_meshing, "greedy meshing must be enabled");
        assert!(dev.distance_lod, "distance LOD must be enabled");
        assert_eq!(
            dev.lod_threshold, 4,
            "default LOD threshold should be 4 chunks"
        );
        assert_eq!(
            dev.pregen_margin, 2,
            "default lookahead pregen margin should be 2 chunks"
        );
    }

    #[test]
    fn test_dev_settings_custom_dev_mode() {
        let dev = DevSettings {
            dev_mode: true,
            show_debug_hud: true,
            ..Default::default()
        };

        assert!(dev.dev_mode);
        assert!(dev.show_debug_hud);
        assert!(dev.greedy_meshing);
    }

    #[test]
    fn test_seed_input_state_and_keycode_to_char() {
        let default_state = SeedInputState::default();
        assert!(default_state.seed_text.is_empty());
        assert!(!default_state.is_editing);

        // Test character conversion
        assert_eq!(keycode_to_char(KeyCode::KeyA, false), Some('a'));
        assert_eq!(keycode_to_char(KeyCode::KeyA, true), Some('A'));
        assert_eq!(keycode_to_char(KeyCode::Digit7, false), Some('7'));
        assert_eq!(keycode_to_char(KeyCode::Minus, false), Some('-'));
        assert_eq!(keycode_to_char(KeyCode::Minus, true), Some('_'));
    }

    #[test]
    fn test_get_option_description_all_actions() {
        let actions = [
            MenuButtonAction::ToggleVsync,
            MenuButtonAction::ToggleFullscreen,
            MenuButtonAction::CycleFpsCap,
            MenuButtonAction::CycleViewDistance,
            MenuButtonAction::ToggleBackfaceCulling,
            MenuButtonAction::ToggleShadows,
            MenuButtonAction::ToggleMaxYSkip,
            MenuButtonAction::ToggleDistanceFog,
            MenuButtonAction::ToggleMeshBudget,
            MenuButtonAction::ToggleAsyncMeshing,
            MenuButtonAction::ToggleGreedyMeshing,
            MenuButtonAction::ToggleDistanceLod,
            MenuButtonAction::CycleLodThreshold,
            MenuButtonAction::CyclePregenMargin,
            MenuButtonAction::ToggleDebugHud,
            MenuButtonAction::BackFromSettings,
            MenuButtonAction::BackFromDevSettings,
            MenuButtonAction::Play,
            MenuButtonAction::ResumeGame,
            MenuButtonAction::OpenSettings,
            MenuButtonAction::OpenDevSettings,
            MenuButtonAction::BackToMain,
            MenuButtonAction::QuitGame,
            MenuButtonAction::ToggleEditSeed,
            MenuButtonAction::RandomizeSeed,
        ];

        for action in actions {
            let desc = get_option_description(&action);
            assert!(desc.is_some(), "Every action must have a valid description");
            let d = desc.unwrap();
            assert!(!d.header.is_empty(), "Header must not be empty");
            assert!(!d.title.is_empty(), "Title must not be empty");
            assert!(!d.description.is_empty(), "Description must not be empty");
            assert!(!d.impact.is_empty(), "Impact must not be empty");
        }
    }
}
