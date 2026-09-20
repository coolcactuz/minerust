mod block;
mod camera;
mod chunk;
mod interaction;
mod inventory;
mod mesher;
mod noise;
mod texture;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use camera::{camera_look_system, camera_move_system, cursor_grab_system, FpsCamera};
use interaction::block_interaction_system;
use inventory::{
    inventory_input_system, inventory_interaction_system, setup_inventory_ui,
    update_inventory_ui_system, Inventory,
};
use world::{
    calculate_biome_and_height, generate_chunk, update_chunk_mesh, world_streaming_system,
    WorldGrid, WorldSeed,
};

fn main() {
    // 1. Parsing del Seed da riga di comando (es: cargo run -- --seed "minecraft" oppure -s 123456)
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
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .insert_resource(WorldGrid::new(seed))
        .init_resource::<Inventory>()
        .add_systems(Startup, (setup, setup_inventory_ui))
        .add_systems(
            Update,
            (
                cursor_grab_system,
                camera_look_system,
                camera_move_system,
                inventory_input_system,
                inventory_interaction_system,
                update_inventory_ui_system,
                block_interaction_system,
                world_streaming_system,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut world: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // 0. Texture Atlas 128x128 pixel-art e Materiale PBR condiviso con Alpha Mask per trasparenze (Vetro)
    let atlas_image = texture::create_texture_atlas();
    let atlas_handle = images.add(atlas_image);

    let block_mat = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        perceptual_roughness: 0.85,
        reflectance: 0.15,
        cull_mode: None,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });
    world.block_material = Some(block_mat);

    let seed = world.seed.0;
    let noise = world.noise.clone();

    // 1. Pre-genera una griglia iniziale 5x5 di chunk attorno allo spawn (0, 0)
    for cx in -2..=2 {
        for cz in -2..=2 {
            let coord = IVec2::new(cx, cz);
            let chunk = generate_chunk(cx, cz, &noise, seed);
            world.chunks.insert(coord, chunk);
            update_chunk_mesh(&coord, &mut commands, &mut world, &mut meshes, &mut materials);
        }
    }

    // 2. Calcola la quota del terreno al punto di spawn per posizionare il giocatore in modo naturale
    let (spawn_biome, spawn_y, _) = calculate_biome_and_height(0.0, 0.0, &noise);
    let player_y = (spawn_y as f32 + 4.0).max(28.0);

    // 3. Spawna la camera FPS con AmbientLight integrata
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            color: Color::WHITE,
            brightness: 320.0,
            ..default()
        },
        Transform::from_xyz(0.0, player_y, 0.0).looking_at(Vec3::new(20.0, player_y - 2.0, 20.0), Vec3::Y),
        FpsCamera::default(),
    ));

    // 4. Luce del Sole (Directional Light) con ombre
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(60.0, 100.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("\n=======================================================");
    println!("⛏️  MINERUST: TEXTURE ATLAS & INVENTARIO COMPLETATI");
    println!("=======================================================");
    println!("* SEED DEL MONDO: {}", seed);
    println!("* BIOMA DI SPAWN: {:?}", spawn_biome);
    println!("* CONTROLLI:");
    println!("  - WASD + Mouse: Movimento e visuale");
    println!("  - Click Sinistro: Spacca blocco (o blocca cursore se sbloccato)");
    println!("  - Click Destro: Piazza blocco selezionato");
    println!("  - Tasti 1-9 o Rotellina del Mouse: Seleziona slot rapido nella Hotbar");
    println!("  - Tasto 'E': Apri / Chiudi Inventario Completo (clicca per equipaggiare)");
    println!("  - Tasto ESC: Chiudi inventario / Sblocca cursore");
    println!("  - Spazio / Shift: Vola su / giù | Ctrl: Scatto veloce");
    println!("=======================================================\n");
}
