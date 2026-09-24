use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::inventory::Inventory;
use crate::menu::{GraphicsSettings, MenuState};
use crate::voxel_material::VoxelBlockMaterial;
use crate::world::{
    WorldGrid, chunk_distance_sq_to_player, determine_chunk_tier, update_chunk_mesh,
};

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

    let delta_x = if dir.x != 0.0 {
        (1.0 / dir.x).abs()
    } else {
        f32::INFINITY
    };
    let delta_y = if dir.y != 0.0 {
        (1.0 / dir.y).abs()
    } else {
        f32::INFINITY
    };
    let delta_z = if dir.z != 0.0 {
        (1.0 / dir.z).abs()
    } else {
        f32::INFINITY
    };

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

#[derive(SystemParam)]
pub struct InteractionAssets<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<VoxelBlockMaterial>>,
}

#[derive(SystemParam)]
pub struct InteractionContext<'w> {
    pub mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    pub inventory: ResMut<'w, Inventory>,
    pub menu: Option<Res<'w, MenuState>>,
    pub graphics: Option<Res<'w, GraphicsSettings>>,
}

pub fn block_interaction_system(
    mut commands: Commands,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &FpsCamera)>,
    mut context: InteractionContext,
    mut world: ResMut<WorldGrid>,
    mut fluid_sim: ResMut<crate::fluid::FluidSimulation>,
    mut assets: InteractionAssets,
    mut gizmos: Gizmos,
) {
    if let Some(menu) = context.menu.as_ref() {
        if menu.is_open() {
            return;
        }
    }
    if context.inventory.is_open {
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

        // Left click: Break block (except indestructible Bedrock) and collect resource
        if context.mouse_buttons.just_pressed(MouseButton::Left) {
            if hit.hit_block.y > 0 && world.get_block(hit.hit_block) != BlockType::Bedrock {
                let target_block = world.get_block(hit.hit_block);
                if target_block != BlockType::Air {
                    let affected = world.set_block(hit.hit_block, BlockType::Air);
                    dirty_coords.extend(affected);
                    fluid_sim.sources.remove(&hit.hit_block);
                    fluid_sim.schedule_neighbors(hit.hit_block);

                    // Yield mined resource directly into player's inventory
                    if let Some(drop) = target_block.drop_item() {
                        let leftover = context.inventory.add_item(drop, 1);
                        if leftover > 0 {
                            tracing::info!("Inventory full! Could not store mined item {drop:?}");
                        }
                    }
                }
            }
        }
        // Right click: Place block (if an item is equipped in the active hotbar slot)
        else if context.mouse_buttons.just_pressed(MouseButton::Right) {
            if let Some(selected_stack) = context.inventory.selected_item() {
                let block_to_place = selected_stack.block_type;
                if block_to_place != BlockType::Air {
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

                        // Consume 1 block from active hotbar slot
                        context.inventory.consume_selected(1);
                    }
                }
            }
        }

        // If blocks were modified, regenerate meshes for all affected chunks
        if !dirty_coords.is_empty() {
            dirty_coords.sort_unstable_by_key(|c| (c.x, c.y));
            dirty_coords.dedup();

            let player_pos = cam_transform.translation;

            let (distance_lod, lod_threshold_sq, greedy_meshing, greedy_threshold, greedy_threshold_sq) =
                if let Some(ref g) = context.graphics {
                    let l_sq = (g.lod_threshold as f32 * 16.0).powi(2);
                    let g_sq = if g.greedy_threshold <= 0 {
                        0.0
                    } else {
                        (g.greedy_threshold as f32 * 16.0).powi(2)
                    };
                    (g.distance_lod, l_sq, g.greedy_meshing, g.greedy_threshold, g_sq)
                } else {
                    (true, 128.0 * 128.0, true, 2, 32.0 * 32.0)
                };

            for coord in dirty_coords {
                let chunk_opt = world.chunks.get(&coord);
                let dist_sq = chunk_distance_sq_to_player(coord, player_pos, chunk_opt);
                let (tier, _, _) = determine_chunk_tier(
                    dist_sq,
                    distance_lod,
                    lod_threshold_sq,
                    greedy_meshing,
                    greedy_threshold,
                    greedy_threshold_sq,
                );
                update_chunk_mesh(
                    &coord,
                    &mut commands,
                    &mut world,
                    &mut assets.meshes,
                    &mut assets.materials,
                    true,
                    tier,
                );
            }
        }
    }
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            block_interaction_system.in_set(crate::stage::VoxelStage::InputHandling),
        );
    }
}
