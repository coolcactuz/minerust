mod block;
mod camera;
mod chunk;
mod interaction;
mod mesher;
mod noise;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use camera::{camera_look_system, camera_move_system, cursor_grab_system, FpsCamera};
use interaction::{block_interaction_system, player_hand_input_system, PlayerHand};
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
        .init_resource::<PlayerHand>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                cursor_grab_system,
                camera_look_system,
                camera_move_system,
                player_hand_input_system,
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
) {
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
    println!("⛏️  MINERUST: MOTORE DI GENERAZIONE PROCEDURALE DEFINITIVO");
    println!("=======================================================");
    println!("* SEED DEL MONDO: {}", seed);
    println!("* BIOMA DI SPAWN: {:?}", spawn_biome);
    println!("* NOTA: Puoi avviare un mondo specifico con:");
    println!("        cargo run -- --seed <numero_o_testo>");
    println!("* BIOMI: Pianure, Foreste, Deserti con Cactus, Tundra Innevata,");
    println!("         Montagne con cime innevate, Oceani/Laghi, Fiumi");
    println!("* SOTTOSUOLO: Caverne 3D e vene di Carbone, Ferro, Oro, Diamante");
    println!("* CONTROLLI:");
    println!("  - WASD + Mouse (clicca per bloccare, ESC per sbloccare)");
    println!("  - Spazio/Shift: Vola su/giù | Ctrl: Scatto");
    println!("  - Click Sinistro: Spacca blocco | Click Destro: Piazza blocco");
    println!("  - Tasti 1-0, -, =: Seleziona blocco (inclusi Vetro, Diamante, Cactus)");
    println!("=======================================================\n");
}
