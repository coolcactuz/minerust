mod block;
mod camera;
mod chunk;
pub mod coords;
pub mod error;
mod fluid;
mod interaction;
mod inventory;
mod menu;
mod mesher;
mod noise;
mod physics;
pub mod stage;
mod texture;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use camera::{CameraPlugin, FpsCamera};
use chunk::Chunk;
use fluid::FluidPlugin;
use interaction::InteractionPlugin;
use inventory::InventoryPlugin;
use menu::MenuPlugin;
use physics::{PhysicsPlugin, PlayerPhysics};
use stage::VoxelStage;
use world::{
    SEA_LEVEL, WorldGrid, WorldPlugin, WorldSeed, calculate_biome_and_height, generate_chunk,
    update_chunk_mesh,
};

fn main() {
    // 1. Parse World Seed from command line (e.g.: cargo run -- --seed "minecraft" or -s 123456)
    let mut seed = WorldSeed::default();
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if (args[i] == "--seed" || args[i] == "-s") && i + 1 < args.len() {
            seed = WorldSeed::from_str(&args[i + 1]);
        }
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("MineRust ⛏️🦀 - Seed: {}", seed.0),
                resolution: WindowResolution::new(1280, 720),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .insert_resource(WorldGrid::new(seed))
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
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut world: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
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

    // 1. Pre-generate initial 9x9 chunk grid around spawn (0, 0) in parallel across all CPU cores
    let initial_coords: Vec<IVec2> = (-4..=4)
        .flat_map(|cx| (-4..=4).map(move |cz| IVec2::new(cx, cz)))
        .collect();

    let mut generated_chunks: Vec<(IVec2, Chunk)> = Vec::with_capacity(initial_coords.len());

    std::thread::scope(|s| {
        let mut handles = Vec::with_capacity(initial_coords.len());
        for coord in &initial_coords {
            let noise_ref = &noise;
            handles.push(s.spawn(move || {
                let chunk = generate_chunk(coord.x, coord.y, noise_ref, seed);
                (*coord, chunk)
            }));
        }
        for handle in handles {
            if let Ok(res) = handle.join() {
                generated_chunks.push(res);
            }
        }
    });

    for (coord, chunk) in generated_chunks {
        world.chunks.insert(coord, chunk);
    }

    for coord in &initial_coords {
        update_chunk_mesh(
            coord,
            &mut commands,
            &mut world,
            &mut meshes,
            &mut materials,
            true,
            true,
        );
    }

    // 2. Calculate terrain height at spawn to position the player naturally
    let (spawn_biome, spawn_y, _) = calculate_biome_and_height(0.0, 0.0, &noise);
    let player_y = (spawn_y as f32 + 4.0).max((SEA_LEVEL + 4) as f32);

    // 3. Spawn FPS camera with integrated AmbientLight, DistanceFog, and PlayerPhysics component
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
        Transform::from_xyz(0.0, player_y, 0.0)
            .looking_at(Vec3::new(20.0, player_y - 2.0, 20.0), Vec3::Y),
        FpsCamera::default(),
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

    println!("\n=======================================================");
    println!("⛏️  MINERUST: FULL VOXEL ENGINE ACTIVE");
    println!("=======================================================");
    println!("* WORLD SEED: {seed}");
    println!("* SPAWN BIOME: {spawn_biome:?}");
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
    println!("=======================================================\n");
}
