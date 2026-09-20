use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::menu::MenuState;
use crate::world::{chunk_distance_sq_to_player, update_chunk_mesh, WorldGrid};

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

pub fn block_interaction_system(
    mut commands: Commands,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &FpsCamera)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    inventory: Res<Inventory>,
    menu: Option<Res<MenuState>>,
    dev_settings: Option<Res<crate::menu::DevSettings>>,
    mut world: ResMut<WorldGrid>,
    mut fluid_sim: ResMut<crate::fluid::FluidSimulation>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
) {
    if let Some(menu) = menu {
        if menu.is_open() {
            return;
        }
    }
    if inventory.is_open {
        return;
    }

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
        // Draw wireframe bounding box around targeted block
        let center = hit.hit_block.as_vec3() + Vec3::splat(0.5);
        gizmos.cube(
            Transform::from_translation(center).with_scale(Vec3::splat(1.005)),
            Color::WHITE,
        );

        let mut dirty_coords = Vec::new();

        // Left click: Break block (except indestructible Bedrock)
        if mouse_buttons.just_pressed(MouseButton::Left) {
            if hit.hit_block.y > 0 && world.get_block(hit.hit_block) != BlockType::Bedrock {
                let affected = world.set_block(hit.hit_block, BlockType::Air);
                dirty_coords.extend(affected);
                fluid_sim.sources.remove(&hit.hit_block);
                fluid_sim.schedule_neighbors(hit.hit_block);
            }
        }
        // Right click: Place block (if not colliding with the player's body)
        else if mouse_buttons.just_pressed(MouseButton::Right) {
            let block_to_place = inventory.selected_block();
            let mut can_place = true;

            if block_to_place.is_solid() {
                let feet = cam_transform.translation - Vec3::new(0.0, 1.62, 0.0);
                let p_min = feet + Vec3::new(-0.3, 0.0, -0.3);
                let p_max = feet + Vec3::new(0.3, 1.8, 0.3);

                let b_min = hit.place_pos.as_vec3();
                let b_max = b_min + Vec3::ONE;

                let overlaps = p_min.x < b_max.x
                    && p_max.x > b_min.x
                    && p_min.y < b_max.y
                    && p_max.y > b_min.y
                    && p_min.z < b_max.z
                    && p_max.z > b_min.z;

                if overlaps {
                    can_place = false;
                }
            }

            if can_place {
                let affected = world.set_block(hit.place_pos, block_to_place);
                dirty_coords.extend(affected);
                if block_to_place == BlockType::Water {
                    fluid_sim.sources.insert(hit.place_pos);
                } else {
                    fluid_sim.sources.remove(&hit.place_pos);
                }
                fluid_sim.schedule_neighbors(hit.place_pos);
            }
        }

        // If blocks were modified, regenerate meshes for all affected chunks
        if !dirty_coords.is_empty() {
            dirty_coords.sort_unstable_by_key(|c| (c.x, c.y));
            dirty_coords.dedup();

            let player_pos = cam_transform.translation;
            let max_y_skip = dev_settings.as_ref().map_or(true, |d| d.max_y_skip);
            let distance_lod = dev_settings.as_ref().map_or(true, |d| d.distance_lod);
            let lod_threshold = dev_settings.as_ref().map_or(4, |d| d.lod_threshold);
            let threshold_world = (lod_threshold as f32) * 16.0;
            let threshold_sq = threshold_world * threshold_world;
            let global_greedy = dev_settings.as_ref().map_or(true, |d| d.greedy_meshing);

            for coord in dirty_coords {
                let chunk_opt = world.chunks.get(&coord);
                let dist_sq = chunk_distance_sq_to_player(coord, player_pos, chunk_opt);
                let greedy = if distance_lod {
                    dist_sq > threshold_sq
                } else {
                    global_greedy
                };
                update_chunk_mesh(&coord, &mut commands, &mut world, &mut meshes, &mut materials, max_y_skip, greedy);
            }
        }
    }
}
