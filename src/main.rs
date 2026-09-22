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
        if args[i] == "--help" || args[i] == "-h" {
            println!("MineRust - A High-Performance Voxel Sandbox in Rust");
            println!("Usage: minerust [OPTIONS]");
            println!("\nOptions:");
            println!("  -s, --seed <SEED>       Set world generation seed (string or integer)");
            println!(
                "  -d, --dev, --debug      Enable Developer Mode (benchmarks & debug settings)"
            );
            println!("  -p, --profile           Enable Real-time Performance Profiler HUD");
            println!("  -q, --quickstart        Start directly in-game bypassing the main menu");
            println!("  -h, --help              Print help information");
            return;
        }
    }

    // Default to a fresh random seed for new game if not provided via CLI
    if !seed_specified {
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

    let dev_settings = DevSettings {
        dev_mode,
        profile_mode,
        show_debug_hud: dev_mode || profile_mode,
        ..default()
    };

    let graphics_settings = GraphicsSettings::load_or_default();
    let present_mode = if graphics_settings.vsync {
        bevy::window::PresentMode::AutoVsync
    } else {
        bevy::window::PresentMode::AutoNoVsync
    };
    let mode = if graphics_settings.fullscreen {
        bevy::window::WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current)
    } else {
        bevy::window::WindowMode::Windowed
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: if dev_mode {
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
) {
    // 0. 2D Texture Array (25 layers of 16x16 pixel-art) and shared ExtendedMaterial with Alpha Mask for transparency (Glass)
    let array_image = texture::create_texture_array();
    let array_handle = images.add(array_image);

    let block_mat = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 0.85,
            reflectance: 0.15,
            cull_mode: Some(bevy::render::render_resource::Face::Back),
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
        world.pregenerate_spawn_grid(center_chunk, &mut commands, &mut meshes, &mut materials);
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
