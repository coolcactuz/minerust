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
use minerust::world::{WorldGrid, WorldPlugin, WorldSeed};

fn main() {
    // 1. Parse World Seed and Dev flags from command line
    let mut seed = WorldSeed::default();
    let mut seed_specified = false;
    let mut dev_mode = false;
    let mut profile_mode = false;
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
        if args[i] == "--help" || args[i] == "-h" {
            println!("MineRust - A High-Performance Voxel Sandbox in Rust");
            println!("Usage: minerust [OPTIONS]");
            println!("\nOptions:");
            println!("  -s, --seed <SEED>       Set world generation seed (string or integer)");
            println!(
                "  -d, --dev, --debug      Enable Developer Mode (benchmarks & debug settings)"
            );
            println!("  -p, --profile           Enable Real-time Performance Profiler HUD");
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
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.10))) // Clean dark background
        .insert_resource(WorldGrid::new(seed))
        .insert_resource(graphics_settings)
        .insert_resource(dev_settings)
        .insert_resource(seed_state)
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    graphics_settings: Res<GraphicsSettings>,
) {
    // 0. 128x128 pixel-art Texture Atlas and shared PBR material with Alpha Mask for transparency (Glass)
    let atlas_image = texture::create_texture_atlas();
    let atlas_handle = images.add(atlas_image);

    let block_mat = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        perceptual_roughness: 0.85,
        reflectance: 0.15,
        cull_mode: Some(bevy::render::render_resource::Face::Back),
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });
    world.block_material = Some(block_mat);

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
            color: Color::srgb(0.70, 0.82, 0.95),
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
