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
    calculate_surface_height, generate_chunk, update_chunk_mesh, world_streaming_system,
    WorldGrid,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MineRust ⛏️🦀 - Procedural Infinite Voxel World".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.98))) // Sky blue
        .init_resource::<WorldGrid>()
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
    // 1. Pre-genera una griglia iniziale 3x3 di chunk attorno allo spawn (0, 0)
    for cx in -1..=1 {
        for cz in -1..=1 {
            let coord = IVec2::new(cx, cz);
            let chunk = generate_chunk(cx, cz);
            world.chunks.insert(coord, chunk);
            update_chunk_mesh(&coord, &mut commands, &mut world, &mut meshes, &mut materials);
        }
    }

    // 2. Calcola la quota del terreno al punto di spawn per posizionare il giocatore in modo naturale
    let (spawn_y, _) = calculate_surface_height(0.0, 0.0);
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
        Transform::from_xyz(50.0, 80.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("\n=== MINERUST: MONDO PROCEDURALE INFINITO AVVIATO ===");
    println!("* Biomi: Montagne con cime innevate, Mare, Fiumi, Laghi, Caverne 3D sotterranee");
    println!("* Generazione procedurale e streaming in realtime attivo mentre ti muovi!");
    println!("* Clicca sulla finestra per catturare il cursore (ESC per rilasciare)");
    println!("* WASD + Spazio/Shift: Movimento / Volo");
    println!("* Ctrl sinistro: Scatto rapido");
    println!("* Mouse: Visuale libera");
    println!("* Click Sinistro: Spacca blocco puntato");
    println!("* Click Destro: Piazza blocco");
    println!("* Tasti 1-9: Seleziona blocco (1: Pietrisco, 2: Terra, 3: Legno, 4: Foglie, 5: Assi, 6: Pietra, 7: Sabbia, 8: Acqua, 9: Neve)\n");
}
