mod block;
mod camera;
mod chunk;
mod interaction;
mod mesher;
mod world;

use bevy::prelude::*;
use bevy::window::WindowResolution;

use camera::{camera_look_system, camera_move_system, cursor_grab_system, FpsCamera};
use interaction::{block_interaction_system, player_hand_input_system, remesh_chunk, PlayerHand};
use world::WorldGrid;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MineRust ⛏️🦀 - Voxel Engine MVP".into(),
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
    // 1. Generazione del mondo voxel (raggio 2 = 5x5 chunk, ovvero 80x80 blocchi)
    world.generate_world(2);

    // 2. Generazione delle mesh iniziali per ogni chunk
    let coords: Vec<IVec2> = world.chunks.keys().copied().collect();
    for coord in coords {
        remesh_chunk(&coord, &mut commands, &mut world, &mut meshes, &mut materials);
    }

    // 3. Spawna la camera FPS con AmbientLight integrata
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        },
        Transform::from_xyz(0.0, 22.0, 30.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        FpsCamera::default(),
    ));

    // 4. Luce del Sole (Directional Light) con ombre
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(40.0, 60.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    println!("\n=== MINERUST MVP AVVIATO ===");
    println!("* Clicca sulla finestra per catturare il cursore (ESC per rilasciare)");
    println!("* WASD + Spazio/Shift: Movimento / Volo");
    println!("* Ctrl sinistro: Scatto veloce");
    println!("* Mouse: Guarda intorno");
    println!("* Click Sinistro: Spacca blocco puntato");
    println!("* Click Destro: Piazza blocco");
    println!("* Tasti 1-6: Seleziona blocco (1: Pietrisco, 2: Terra, 3: Legno, 4: Foglie, 5: Assi, 6: Pietra)\n");
}
