use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::window::WindowResolution;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use minerust::camera::{CameraPlugin, FpsCamera};
use minerust::fluid::FluidPlugin;
use minerust::interaction::InteractionPlugin;
use minerust::inventory::InventoryPlugin;
use minerust::menu::{DevSettings, GraphicsSettings, MenuPlugin, SeedInputState};
use minerust::physics::{PhysicsPlugin, PlayerPhysics};
use minerust::profile::ProfilePlugin;
use minerust::save::SavePlugin;
use minerust::stage::VoxelStage;
use minerust::texture;
use minerust::voxel_material::{VoxelBlockMaterial, VoxelExtension};
use minerust::world::{WorldGrid, WorldPlugin, WorldSeed};

fn main() {
    // 1. Parse World Seed and Dev flags from command line
    let mut seed = WorldSeed::default();
    let mut seed_specified = false;
    let mut dev_mode = false;
    let mut profile_mode = false;
    let mut quickstart = false;
    let mut benchmark_mode = false;
    let mut selected_preset: Option<minerust::benchmark::BenchmarkPreset> = None;
    let mut benchmark_distance = 1000.0f32; // Default 1 km (1000 blocks)
    let mut benchmark_speed = 50.0f32; // Default 50 m/s (180 km/h) = 20s for 1km
    let mut benchmark_warmup = 2.5f32;
    let mut benchmark_output = Some("benchmark_results.json".to_string());
    let mut view_dist_override: Option<i32> = None;
    let mut override_greedy: Option<bool> = None;
    let mut override_lod: Option<bool> = None;
    let mut override_culling: Option<bool> = None;
    let mut override_max_y: Option<bool> = None;
    let mut override_fog: Option<bool> = None;

    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if (args[i] == "--seed" || args[i] == "-s") && i + 1 < args.len() {
            seed = WorldSeed::from_seed_str(&args[i + 1]);
            seed_specified = true;
        }
        if args[i] == "--dev" || args[i] == "--debug" || args[i] == "-d" {
            dev_mode = true;
        }
        if args[i] == "--profile" || args[i] == "-p" {
            profile_mode = true;
        }
        if args[i] == "--quickstart" || args[i] == "-q" {
            quickstart = true;
        }
        if args[i] == "--benchmark" || args[i] == "-b" {
            benchmark_mode = true;
            quickstart = true;
        }
        if (args[i] == "--benchmark-preset" || args[i] == "-bp") && i + 1 < args.len() {
            selected_preset = minerust::benchmark::BenchmarkPreset::from_str_name(&args[i + 1]);
        }
        if (args[i] == "--benchmark-distance" || args[i] == "-bd") && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<f32>() {
                benchmark_distance = val;
            }
        }
        if (args[i] == "--benchmark-speed" || args[i] == "-bs") && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<f32>() {
                benchmark_speed = val;
            }
        }
        if args[i] == "--benchmark-warmup" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<f32>() {
                benchmark_warmup = val;
            }
        }
        if args[i] == "--benchmark-output" && i + 1 < args.len() {
            benchmark_output = Some(args[i + 1].clone());
        }
        if args[i] == "--view-distance" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<i32>() {
                view_dist_override = Some(val.clamp(2, 64));
            }
        }
        if args[i] == "--no-greedy" {
            override_greedy = Some(false);
        } else if args[i] == "--enable-greedy" {
            override_greedy = Some(true);
        }
        if args[i] == "--no-lod" {
            override_lod = Some(false);
        } else if args[i] == "--enable-lod" {
            override_lod = Some(true);
        }
        if args[i] == "--no-culling" {
            override_culling = Some(false);
        } else if args[i] == "--enable-culling" {
            override_culling = Some(true);
        }
        if args[i] == "--no-max-y-skip" {
            override_max_y = Some(false);
        } else if args[i] == "--enable-max-y-skip" {
            override_max_y = Some(true);
        }
        if args[i] == "--no-fog" {
            override_fog = Some(false);
        } else if args[i] == "--enable-fog" {
            override_fog = Some(true);
        }
        if args[i] == "--help" || args[i] == "-h" {
            println!("MineRust - A High-Performance Voxel Sandbox in Rust");
            println!("Usage: minerust [OPTIONS]");
            println!("\nOptions:");
            println!("  -s, --seed <SEED>              Set world generation seed (string or integer)");
            println!(
                "  -d, --dev, --debug             Enable Developer Mode (benchmarks & debug settings)"
            );
            println!("  -p, --profile                  Enable Real-time Performance Profiler HUD");
            println!("  -q, --quickstart               Start directly in-game bypassing the main menu");
            println!("  -b, --benchmark                Run automated reproducible benchmark (uncapped FPS)");
            println!("  -bp, --benchmark-preset <name> Preset: baseline, culling, greedy, sloped_lod, production");
            println!("  -bd, --benchmark-distance <m>  Target flight distance in meters (default: 1000m / 1km)");
            println!("  -bs, --benchmark-speed <m/s>   Flight speed in m/s (default: 50 m/s = 180 km/h)");
            println!("       --benchmark-warmup <s>    Warmup period before recording in seconds (default: 2.5)");
            println!("       --benchmark-output <path> JSON file output path (default: benchmark_results.json)");
            println!("       --view-distance <chunks>  Override render distance (4 to 64 chunks)");
            println!("       --no-greedy               Disable greedy meshing (use naive meshing)");
            println!("       --no-lod                  Disable distance-based sloped LOD");
            println!("       --no-culling              Disable backface culling");
            println!("       --no-max-y-skip           Disable empty atmosphere scanning skip");
            println!("       --no-fog                  Disable atmospheric distance fog");
            println!("  -h, --help                     Print help information");
            return;
        }
    }

    // Benchmark mode defaults to a deterministic fixed seed (133742) for reproducible tests
    if benchmark_mode && !seed_specified {
        seed = WorldSeed(133742);
    } else if !seed_specified {
        seed = WorldSeed::random();
    }

    let seed_state = SeedInputState {
        seed_text: seed.0.to_string(),
        is_editing: false,
    };

    let menu_state = if quickstart {
        minerust::menu::MenuState {
            screen: minerust::menu::MenuScreen::None,
            previous_screen: minerust::menu::MenuScreen::None,
            world_active: true,
        }
    } else {
        minerust::menu::MenuState::default()
    };

    let mut dev_settings = DevSettings {
        dev_mode,
        profile_mode,
        show_debug_hud: dev_mode || profile_mode,
        ..default()
    };

    let mut graphics_settings = GraphicsSettings::load_or_default();

    // Apply benchmark preset if selected
    if let Some(preset) = selected_preset {
        preset.apply(&mut dev_settings, &mut graphics_settings);
    }

    // Apply granular overrides if specified
    if let Some(vd) = view_dist_override {
        graphics_settings.view_distance = vd;
    }
    if let Some(g) = override_greedy {
        graphics_settings.greedy_meshing = g;
        dev_settings.greedy_meshing = g;
    }
    if let Some(lod) = override_lod {
        graphics_settings.distance_lod = lod;
        dev_settings.distance_lod = lod;
    }
    if let Some(cull) = override_culling {
        dev_settings.backface_culling = cull;
    }
    if let Some(my) = override_max_y {
        dev_settings.max_y_skip = my;
    }
    if let Some(fog) = override_fog {
        graphics_settings.distance_fog = fog;
        dev_settings.distance_fog = fog;
    }

    let present_mode = if benchmark_mode {
        bevy::window::PresentMode::AutoNoVsync // Force un-capped framerate during benchmarks
    } else if graphics_settings.vsync {
        bevy::window::PresentMode::AutoVsync
    } else {
        bevy::window::PresentMode::AutoNoVsync
    };
    let mode = if graphics_settings.fullscreen {
        bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };

    let benchmark_config = minerust::benchmark::BenchmarkConfig {
        enabled: benchmark_mode,
        preset_name: selected_preset.map_or("Custom".to_string(), |p| p.name().to_string()),
        seed: seed.0,
        view_distance: graphics_settings.view_distance,
        warmup_duration_secs: benchmark_warmup,
        target_distance_meters: benchmark_distance,
        flight_speed: benchmark_speed,
        output_path: benchmark_output,
        ..default()
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: if benchmark_mode {
                            format!("MineRust [BENCHMARK MODE] - Seed: {}", seed.0)
                        } else if dev_mode {
                            format!("MineRust [DEV MODE] - Seed: {}", seed.0)
                        } else if profile_mode {
                            format!("MineRust [PROFILE MODE] - Seed: {}", seed.0)
                        } else {
                            format!("MineRust - Seed: {}", seed.0)
                        },
                        resolution: WindowResolution::new(1280, 720),
                        present_mode,
                        mode,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::audio::AudioPlugin>(),
        )
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .insert_resource(WorldGrid::new(seed))
        .insert_resource(graphics_settings)
        .insert_resource(dev_settings)
        .insert_resource(seed_state)
        .insert_resource(menu_state)
        .insert_resource(benchmark_config)
        .configure_sets(
            Update,
            (
                VoxelStage::InputHandling,
                VoxelStage::PlayerPhysics,
                VoxelStage::FluidSimulation,
                VoxelStage::WorldStreaming,
                VoxelStage::MeshBuilding,
            )
                .chain(),
        )
        .add_plugins(MaterialPlugin::<VoxelBlockMaterial>::default())
        .add_plugins((
            CameraPlugin,
            PhysicsPlugin,
            FluidPlugin,
            WorldPlugin,
            InteractionPlugin,
            InventoryPlugin,
            MenuPlugin,
            SavePlugin,
            ProfilePlugin,
            minerust::benchmark::BenchmarkPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut world: ResMut<WorldGrid>,
    mut materials: ResMut<Assets<VoxelBlockMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    graphics_settings: Res<GraphicsSettings>,
    menu_state: Res<minerust::menu::MenuState>,
    dev_settings: Res<DevSettings>,
) {
    // 0. 2D Texture Array (25 layers of 16x16 pixel-art) and shared ExtendedMaterial with Alpha Mask for transparency (Glass)
    let array_image = texture::create_texture_array();
    let array_handle = images.add(array_image);

    let cull_mode = if dev_settings.backface_culling {
        Some(bevy::render::render_resource::Face::Back)
    } else {
        None
    };

    let block_mat = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            cull_mode,
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        },
        extension: VoxelExtension {
            array_texture: array_handle,
        },
    });
    world.block_material = Some(block_mat);

    if menu_state.world_active {
        let center_chunk = WorldGrid::world_to_chunk_coord(0, 0).0;
        let max_y_skip = dev_settings.max_y_skip;
        let greedy = dev_settings.greedy_meshing && graphics_settings.greedy_meshing;
        world.pregenerate_spawn_grid(
            center_chunk,
            &mut commands,
            &mut meshes,
            &mut materials,
            max_y_skip,
            greedy,
        );
    }

    // 1. Spawn FPS camera with integrated AmbientLight, PlayerPhysics component, and DistanceFog (if enabled in settings)
    let fps_camera = FpsCamera::default();

    let mut cam_builder = commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            far: 2500.0,
            ..default()
        }),
        AmbientLight {
            color: Color::WHITE,
            brightness: 320.0,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_y(fps_camera.yaw)
                * Quat::from_rotation_x(fps_camera.pitch),
            ..default()
        },
        fps_camera,
        PlayerPhysics::default(),
    ));

    if graphics_settings.distance_fog {
        let max_dist = (graphics_settings.view_distance as f32 * 16.0).max(64.0);
        cam_builder.insert(DistanceFog {
            color: Color::srgb(0.53, 0.81, 0.98),
            falloff: FogFalloff::Linear {
                start: (max_dist * 0.70).max(48.0),
                end: (max_dist - 2.0).max(64.0),
            },
            ..default()
        });
    }

    // 2. Sun light (Directional Light) with optimized shadow distance
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        bevy::light::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 30.0,
            maximum_distance: 120.0,
            num_cascades: 2,
            ..default()
        }
        .build(),
        Transform::from_xyz(200.0, 450.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
