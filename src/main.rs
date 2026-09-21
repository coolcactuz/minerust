use bevy::prelude::*;
use bevy::window::WindowResolution;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use minerust::camera::{CameraPlugin, FpsCamera};
use minerust::fluid::FluidPlugin;
use minerust::interaction::InteractionPlugin;
use minerust::inventory::{Inventory, InventoryPlugin};
use minerust::menu::{DevSettings, MenuPlugin, SeedInputState};
use minerust::physics::{PhysicsPlugin, PlayerPhysics};
use minerust::profile::ProfilePlugin;
use minerust::save::{SavePlugin, load_player_from_disk};
use minerust::stage::VoxelStage;
use minerust::texture;
use minerust::world::{
    WorldGrid, WorldPlugin, WorldSeed, calculate_biome_and_height, find_safe_surface_spawn,
};

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
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::audio::AudioPlugin>(),
        )
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .insert_resource(WorldGrid::new(seed))
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
    mut inventory: ResMut<Inventory>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    dev_settings: Option<Res<DevSettings>>,
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

    let seed = world.seed.0;
    let noise = world.noise.clone();

    // 1. Check for persisted player state (position, camera orientation, and inventory)
    let player_save_file = world.player_save_path();
    let maybe_player_data = if player_save_file.exists() {
        match load_player_from_disk(&player_save_file) {
            Ok(data) => {
                info!(
                    "Loaded saved player data from {}",
                    player_save_file.display()
                );
                Some(data)
            }
            Err(e) => {
                warn!("Failed to load player save file: {e}");
                None
            }
        }
    } else {
        None
    };

    let default_fps = FpsCamera::default();
    let (player_pos, player_yaw, player_pitch) = if let Some(ref data) = maybe_player_data {
        inventory.hotbar = data.hotbar;
        inventory.main = data.main;
        inventory.selected_slot = data.selected_slot;
        (Vec3::from_array(data.position), data.yaw, data.pitch)
    } else {
        let spawn_pos = find_safe_surface_spawn(&noise, seed);
        (spawn_pos, default_fps.yaw, default_fps.pitch)
    };

    let center_chunk =
        WorldGrid::world_to_chunk_coord(player_pos.x.floor() as i32, player_pos.z.floor() as i32).0;

    // 2. Pre-generate initial 9x9 chunk grid around center_chunk in parallel across all CPU cores,
    // checking disk first to restore player modifications.
    world.pregenerate_spawn_grid(center_chunk, &mut commands, &mut meshes, &mut materials);

    // 3. Spawn FPS camera with integrated AmbientLight, DistanceFog, and PlayerPhysics component
    let fps_camera = FpsCamera {
        yaw: player_yaw,
        pitch: player_pitch,
        ..default()
    };

    let rot = Quat::from_rotation_y(player_yaw) * Quat::from_rotation_x(player_pitch);

    commands.spawn((
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
        DistanceFog {
            color: Color::srgb(0.70, 0.82, 0.95),
            falloff: FogFalloff::Linear {
                start: 180.0,
                end: 255.0,
            },
            ..default()
        },
        Transform {
            translation: player_pos,
            rotation: rot,
            ..default()
        },
        fps_camera,
        PlayerPhysics::default(),
    ));

    // 4. Sun light (Directional Light) with optimized shadow distance
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

    let (current_biome, _, _) =
        calculate_biome_and_height(player_pos.x as f64, player_pos.z as f64, &noise);
    let is_dev = dev_settings.as_ref().is_some_and(|d| d.dev_mode);

    println!("\n=======================================================");
    println!("MINERUST: FULL VOXEL ENGINE ACTIVE");
    println!("=======================================================");
    println!("* WORLD SEED: {seed}");
    println!("* CURRENT BIOME: {current_biome:?}");
    if is_dev {
        println!("* DEV MODE: ENABLED (Dev & Benchmark Menu + F3 Debug HUD active)");
    } else {
        println!("* DEV MODE: DISABLED (Production Mode: All optimizations locked ON)");
    }
    println!("* CONTROLS:");
    println!("  - WASD: Horizontal movement with inertia and friction");
    println!("  - Mouse: Free-look FPS (Left-click to lock / ESC to unlock)");
    println!("  - SPACE: Jump (or swim upward in water)");
    println!("  - SHIFT: Sneak (crouch, edge protection prevents falling) / Dive in water");
    println!("  - CTRL: Sprint");
    println!("  - KEY 'F': Toggle Flight Mode (No-Clip / Creative)");
    println!("  - Left Click: Break targeted block");
    println!("  - Right Click: Place selected block (with anti-self-collision protection)");
    println!("  - Keys 1-9 or Mouse Wheel: Select quick slot in Hotbar");
    println!("  - Key 'E': Open / Close Full Inventory");
    if is_dev {
        println!("  - Key 'F3': Toggle Dev & Benchmark HUD");
    }
    println!("=======================================================\n");
}
