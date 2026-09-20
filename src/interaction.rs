use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::{CHUNK_DEPTH, CHUNK_WIDTH};
use crate::mesher::build_chunk_mesh;
use crate::world::WorldGrid;

#[derive(Resource)]
pub struct PlayerHand {
    pub selected_block: BlockType,
}

impl Default for PlayerHand {
    fn default() -> Self {
        Self {
            selected_block: BlockType::Cobblestone,
        }
    }
}

pub struct RaycastHit {
    pub hit_block: IVec3,
    pub place_pos: IVec3,
}

pub fn voxel_raycast(
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
    world: &WorldGrid,
) -> Option<RaycastHit> {
    let dir = direction.normalize();

    let mut current = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );
    let mut previous = current;

    let step_x = if dir.x > 0.0 { 1 } else { -1 };
    let step_y = if dir.y > 0.0 { 1 } else { -1 };
    let step_z = if dir.z > 0.0 { 1 } else { -1 };

    let delta_x = if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::INFINITY };
    let delta_y = if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::INFINITY };
    let delta_z = if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::INFINITY };

    let mut t_max_x = if dir.x > 0.0 {
        (current.x as f32 + 1.0 - origin.x) * delta_x
    } else {
        (origin.x - current.x as f32) * delta_x
    };
    let mut t_max_y = if dir.y > 0.0 {
        (current.y as f32 + 1.0 - origin.y) * delta_y
    } else {
        (origin.y - current.y as f32) * delta_y
    };
    let mut t_max_z = if dir.z > 0.0 {
        (current.z as f32 + 1.0 - origin.z) * delta_z
    } else {
        (origin.z - current.z as f32) * delta_z
    };

    let mut distance = 0.0;

    while distance < max_dist {
        if world.is_solid_at(current) {
            return Some(RaycastHit {
                hit_block: current,
                place_pos: previous,
            });
        }

        previous = current;

        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                current.x += step_x;
                distance = t_max_x;
                t_max_x += delta_x;
            } else {
                current.z += step_z;
                distance = t_max_z;
                t_max_z += delta_z;
            }
        } else if t_max_y < t_max_z {
            current.y += step_y;
            distance = t_max_y;
            t_max_y += delta_y;
        } else {
            current.z += step_z;
            distance = t_max_z;
            t_max_z += delta_z;
        }
    }

    None
}

pub fn player_hand_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut hand: ResMut<PlayerHand>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        hand.selected_block = BlockType::Cobblestone;
    } else if keys.just_pressed(KeyCode::Digit2) {
        hand.selected_block = BlockType::Dirt;
    } else if keys.just_pressed(KeyCode::Digit3) {
        hand.selected_block = BlockType::Wood;
    } else if keys.just_pressed(KeyCode::Digit4) {
        hand.selected_block = BlockType::Leaves;
    } else if keys.just_pressed(KeyCode::Digit5) {
        hand.selected_block = BlockType::Planks;
    } else if keys.just_pressed(KeyCode::Digit6) {
        hand.selected_block = BlockType::Stone;
    }
}

pub fn block_interaction_system(
    mut commands: Commands,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &FpsCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    hand: Res<PlayerHand>,
    mut world: ResMut<WorldGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
) {
    let Ok(cursor) = cursor_options.single() else {
        return;
    };
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let Ok((cam_transform, _)) = camera_query.single() else {
        return;
    };

    let ray_origin = cam_transform.translation;
    let ray_dir = cam_transform.forward();

    if let Some(hit) = voxel_raycast(ray_origin, *ray_dir, 8.0, &world) {
        // Disegna un contorno wireframe attorno al blocco puntato
        let center = hit.hit_block.as_vec3() + Vec3::splat(0.5);
        gizmos.cube(
            Transform::from_translation(center).with_scale(Vec3::splat(1.005)),
            Color::WHITE,
        );

        let mut dirty_coords = Vec::new();

        // Tasto sinistro: Spacca blocco
        if mouse_buttons.just_pressed(MouseButton::Left) {
            let affected = world.set_block(hit.hit_block, BlockType::Air);
            dirty_coords.extend(affected);
        }
        // Tasto destro: Piazza blocco
        else if mouse_buttons.just_pressed(MouseButton::Right) {
            let affected = world.set_block(hit.place_pos, hand.selected_block);
            dirty_coords.extend(affected);
        }

        // Se sono stati modificati blocchi, rigenera le mesh dei chunk coinvolti
        if !dirty_coords.is_empty() {
            dirty_coords.sort_unstable_by_key(|c| (c.x, c.y));
            dirty_coords.dedup();

            for coord in dirty_coords {
                remesh_chunk(&coord, &mut commands, &mut world, &mut meshes, &mut materials);
            }
        }
    }
}

pub fn remesh_chunk(
    coord: &IVec2,
    commands: &mut Commands,
    world: &mut WorldGrid,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let Some(chunk) = world.chunks.get(coord) else {
        return;
    };

    let new_mesh = build_chunk_mesh(chunk, coord.x, coord.y, |wx, wy, wz| {
        world.is_solid_at(IVec3::new(wx, wy, wz))
    });

    let world_pos = Vec3::new(
        (coord.x * CHUNK_WIDTH as i32) as f32,
        0.0,
        (coord.y * CHUNK_DEPTH as i32) as f32,
    );

    if let Some(entity) = world.chunk_entities.get(coord) {
        if let Some(mesh) = new_mesh {
            commands.entity(*entity).insert(Mesh3d(meshes.add(mesh)));
        } else {
            commands.entity(*entity).despawn();
            world.chunk_entities.remove(coord);
        }
    } else if let Some(mesh) = new_mesh {
        let material = materials.add(StandardMaterial {
            perceptual_roughness: 0.9,
            reflectance: 0.1,
            ..default()
        });

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::from_translation(world_pos),
            ))
            .id();

        world.chunk_entities.insert(*coord, entity);
    }
}
