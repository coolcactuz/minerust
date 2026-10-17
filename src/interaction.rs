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

/// Tracks continuous block mining progress and active target.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct MiningState {
    /// Coordinate of the voxel currently being mined, or `None` if idle.
    pub target_block: Option<IVec3>,
    /// Accumulated destruction progress normalized from 0.0 to 1.0.
    pub progress: f32,
    /// Timer for rhythmic punch / digging sound effects while mining.
    pub hit_timer: f32,
}

impl MiningState {
    /// Resets mining progress and target.
    #[inline]
    pub fn reset(&mut self) {
        self.target_block = None;
        self.progress = 0.0;
        self.hit_timer = 0.0;
    }

    /// Sets a new target block and resets progress to 0.
    #[inline]
    pub fn start(&mut self, target: IVec3) {
        self.target_block = Some(target);
        self.progress = 0.0;
        self.hit_timer = 0.0;
    }

    /// Returns true if currently mining at the given block position with progress > 0.
    #[inline]
    pub fn is_mining_at(&self, pos: IVec3) -> bool {
        self.target_block == Some(pos) && self.progress > 0.0
    }

    /// Mining damage stage from 0 to 9 for crack animations and sound effects.
    #[inline]
    pub fn stage(&self) -> u8 {
        ((self.progress * 10.0).floor() as u8).min(9)
    }
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
    pub time: Res<'w, Time>,
    pub mining_state: ResMut<'w, MiningState>,
    pub crafting_state: Option<ResMut<'w, crate::crafting::CraftingState>>,
    pub chest_storage: Option<ResMut<'w, crate::chest::ChestStorage>>,
    pub chest_state: Option<ResMut<'w, crate::chest::ChestOpenState>>,
    pub furnace_storage: Option<ResMut<'w, crate::furnace::FurnaceStorage>>,
    pub furnace_state: Option<ResMut<'w, crate::furnace::FurnaceOpenState>>,
    pub sfx: Option<Res<'w, crate::audio::SoundEffects>>,
}

pub fn block_interaction_system(
    mut commands: Commands,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &FpsCamera)>,
    mut context: InteractionContext,
    mut world: ResMut<WorldGrid>,
    mut fluid_sim: ResMut<crate::fluid::FluidSimulation>,
    mut assets: InteractionAssets,
    mut gizmos: Gizmos,
) {
    if let Some(menu) = context.menu.as_ref() {
        if menu.is_open() {
            context.mining_state.reset();
            return;
        }
    }
    if context.inventory.is_open {
        context.mining_state.reset();
        return;
    }

    let Ok(cursor) = cursor_options.single() else {
        context.mining_state.reset();
        return;
    };
    if cursor.grab_mode != CursorGrabMode::Locked {
        context.mining_state.reset();
        return;
    }

    let Ok((cam_transform, _)) = camera_query.single() else {
        context.mining_state.reset();
        return;
    };

    let ray_origin = cam_transform.translation;
    let ray_dir = cam_transform.forward();

    if let Some(hit) = voxel_raycast(ray_origin, *ray_dir, 8.0, &world) {
        // Draw wireframe bounding box around targeted block (colored reddish if actively being mined)
        let center = hit.hit_block.as_vec3() + Vec3::splat(0.5);
        let mining_this_block = context.mining_state.is_mining_at(hit.hit_block);
        let box_color = if mining_this_block {
            let p = context.mining_state.progress;
            Color::srgb(1.0, 1.0 - p * 0.8, 1.0 - p * 0.8)
        } else {
            Color::WHITE
        };
        gizmos.cube(
            Transform::from_translation(center).with_scale(Vec3::splat(1.005)),
            box_color,
        );

        let mut dirty_coords = Vec::new();

        // Left click: Continuous progressive mining (holding left click accumulates block damage)
        if context.mouse_buttons.pressed(MouseButton::Left) {
            let target_block = world.get_block(hit.hit_block);
            if hit.hit_block.y > 0
                && target_block != BlockType::Air
                && !target_block.hardness().is_infinite()
            {
                let is_new_target = context.mining_state.target_block != Some(hit.hit_block);
                if is_new_target {
                    context.mining_state.start(hit.hit_block);
                    // Immediate punch audio feedback on initial click / hit
                    if let Some(ref sfx) = context.sfx {
                        let sound = sfx.for_block(target_block);
                        crate::audio::play_sound(&mut commands, &sound, 0.85);
                    }
                }

                // Resolve equipped tool from active hotbar slot
                let equipped_tool = match context.inventory.selected_item() {
                    Some(stack) => match stack.item {
                        crate::item::ItemType::Tool { tool, tier } => Some((tool, tier)),
                        _ => None,
                    },
                    None => None,
                };

                let break_time = crate::block::block_break_time(target_block, equipped_tool);
                let dt = context.time.delta_secs();
                let progress_delta = if break_time <= 0.0 {
                    1.0
                } else {
                    dt / break_time
                };

                context.mining_state.progress =
                    (context.mining_state.progress + progress_delta).min(1.0);

                // Continuous rhythmic punch / dig sounds (approx 4.5 hits/sec, synchronized with arm swing)
                context.mining_state.hit_timer += dt;
                if context.mining_state.hit_timer >= 0.22 && context.mining_state.progress < 1.0 {
                    context.mining_state.hit_timer -= 0.22;
                    if let Some(ref sfx) = context.sfx {
                        let sound = sfx.for_block(target_block);
                        crate::audio::play_sound(&mut commands, &sound, 0.80);
                    }
                }

                // When progress reaches 1.0, the block breaks and drops items
                if context.mining_state.progress >= 1.0 {
                    if let Some(ref sfx) = context.sfx {
                        let mat_sound = sfx.for_block(target_block);
                        crate::audio::play_sound(&mut commands, &mat_sound, 1.0);
                    }
                    let affected = world.set_block(hit.hit_block, BlockType::Air);
                    dirty_coords.extend(affected);
                    fluid_sim.sources.remove(&hit.hit_block);
                    fluid_sim.schedule_neighbors(hit.hit_block);

                    // If chest was destroyed, dump its contents into player inventory
                    if target_block == BlockType::Chest {
                        if let Some(ref mut cs) = context.chest_storage {
                            let dumped = cs.remove_and_dump(hit.hit_block);
                            for item in dumped {
                                let leftover = context.inventory.add_item(item.item, item.count);
                                if leftover > 0 {
                                    tracing::info!("Inventory full! Dropped chest item {item:?}");
                                }
                            }
                        }
                    }

                    // If furnace was destroyed, dump its contents into player inventory
                    if target_block == BlockType::Furnace || target_block == BlockType::LitFurnace {
                        if let Some(ref mut fs) = context.furnace_storage {
                            let dumped = fs.remove_and_dump(hit.hit_block);
                            for item in dumped {
                                let leftover = context.inventory.add_item(item.item, item.count);
                                if leftover > 0 {
                                    tracing::info!("Inventory full! Dropped furnace item {item:?}");
                                }
                            }
                        }
                    }

                    // Yield mined resource directly into player's inventory if harvestable
                    if target_block.can_harvest(equipped_tool) {
                        if let Some(drop) = crate::item::block_drop(target_block) {
                            let leftover = context.inventory.add_item(drop, 1);
                            if leftover > 0 {
                                tracing::info!(
                                    "Inventory full! Could not store mined item {drop:?}"
                                );
                            }
                        }
                    }

                    context.mining_state.reset();
                }
            } else {
                context.mining_state.reset();
            }
        } else {
            // Player not holding left click: reset active mining progress
            context.mining_state.reset();
        }

        // Right click: Interact with blocks (e.g. Crafting Table, Chest, Furnace) or place equipped block
        if context.mouse_buttons.just_pressed(MouseButton::Right) {
            let target_block = world.get_block(hit.hit_block);
            if target_block == BlockType::CraftingTable {
                if let Some(ref mut cs) = context.crafting_state {
                    cs.mode = crate::crafting::CraftingMode::Table3x3;
                    cs.is_table_open = true;
                    cs.update_output();
                }
                context.inventory.is_open = true;
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            } else if target_block == BlockType::Chest {
                if let Some(ref mut cs) = context.chest_state {
                    cs.open_pos = Some(hit.hit_block);
                }
                if let Some(ref sfx) = context.sfx {
                    crate::audio::play_sound(&mut commands, &sfx.chest_open, 0.7);
                }
                context.inventory.is_open = true;
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            } else if target_block == BlockType::Furnace || target_block == BlockType::LitFurnace {
                if let Some(ref mut fs) = context.furnace_state {
                    fs.open_pos = Some(hit.hit_block);
                }
                context.inventory.is_open = true;
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.grab_mode = CursorGrabMode::None;
                    cursor.visible = true;
                }
            } else if let Some(selected_stack) = context.inventory.selected_item() {
                if let Some(block_to_place) = selected_stack.item.as_block() {
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

                        if let Some(ref sfx) = context.sfx {
                            let sound = sfx.for_block(block_to_place);
                            crate::audio::play_sound(&mut commands, &sound, 0.85);
                        }

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

            let (
                distance_lod,
                lod_threshold_sq,
                greedy_meshing,
                greedy_threshold,
                greedy_threshold_sq,
            ) = if let Some(ref g) = context.graphics {
                let l_sq = (g.lod_threshold as f32 * 16.0).powi(2);
                let g_sq = if g.greedy_threshold <= 0 {
                    0.0
                } else {
                    (g.greedy_threshold as f32 * 16.0).powi(2)
                };
                (
                    g.distance_lod,
                    l_sq,
                    g.greedy_meshing,
                    g.greedy_threshold,
                    g_sq,
                )
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
    } else {
        context.mining_state.reset();
    }
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiningState>()
            .add_plugins(crate::crack::CrackOverlayPlugin)
            .add_systems(
                Update,
                block_interaction_system.in_set(crate::stage::VoxelStage::InputHandling),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ItemStack;

    #[test]
    fn test_mining_state_lifecycle() {
        let mut state = MiningState::default();
        assert_eq!(state.target_block, None);
        assert_eq!(state.progress, 0.0);
        assert!(!state.is_mining_at(IVec3::new(1, 2, 3)));
        assert_eq!(state.stage(), 0);

        state.start(IVec3::new(10, 20, 30));
        assert_eq!(state.target_block, Some(IVec3::new(10, 20, 30)));
        assert_eq!(state.progress, 0.0);
        assert!(!state.is_mining_at(IVec3::new(10, 20, 30)));

        state.progress = 0.35;
        assert!(state.is_mining_at(IVec3::new(10, 20, 30)));
        assert_eq!(state.stage(), 3);

        state.progress = 0.99;
        assert_eq!(state.stage(), 9);

        state.progress = 1.0;
        assert_eq!(state.stage(), 9);

        state.reset();
        assert_eq!(state.target_block, None);
        assert_eq!(state.progress, 0.0);
        assert!(!state.is_mining_at(IVec3::new(10, 20, 30)));
    }

    #[test]
    fn test_chest_destruction_dumps_items_into_inventory() {
        let mut chest_storage = crate::chest::ChestStorage::default();
        let mut inventory = crate::inventory::Inventory::default();
        let pos = IVec3::new(10, 64, 10);

        {
            let chest = chest_storage.get_or_create(pos);
            chest[0] = Some(ItemStack::block(BlockType::DiamondOre, 5));
            chest[1] = Some(ItemStack::material(
                crate::item::MaterialType::IronIngot,
                12,
            ));
        }

        // Simulate block breaking logic
        let dumped = chest_storage.remove_and_dump(pos);
        for item in dumped {
            let leftover = inventory.add_item(item.item, item.count);
            assert_eq!(leftover, 0);
        }

        assert!(chest_storage.containers.is_empty());
        assert_eq!(
            inventory.hotbar[0],
            Some(ItemStack::block(BlockType::DiamondOre, 5))
        );
        assert_eq!(
            inventory.hotbar[1],
            Some(ItemStack::material(
                crate::item::MaterialType::IronIngot,
                12
            ))
        );
    }

    #[test]
    fn test_furnace_destruction_dumps_items_into_inventory() {
        let mut furnace_storage = crate::furnace::FurnaceStorage::default();
        let mut inventory = crate::inventory::Inventory::default();
        let pos = IVec3::new(10, 64, 10);

        {
            let furnace = furnace_storage.get_or_create(pos);
            furnace.input = Some(ItemStack::block(BlockType::IronOre, 3));
            furnace.fuel = Some(ItemStack::material(crate::item::MaterialType::Coal, 1));
            furnace.output = Some(ItemStack::material(crate::item::MaterialType::IronIngot, 2));
        }

        let dumped = furnace_storage.remove_and_dump(pos);
        for item in dumped {
            let leftover = inventory.add_item(item.item, item.count);
            assert_eq!(leftover, 0);
        }

        assert!(furnace_storage.furnaces.is_empty());
        assert_eq!(
            inventory.hotbar[0],
            Some(ItemStack::block(BlockType::IronOre, 3))
        );
        assert_eq!(
            inventory.hotbar[1],
            Some(ItemStack::material(crate::item::MaterialType::Coal, 1))
        );
        assert_eq!(
            inventory.hotbar[2],
            Some(ItemStack::material(crate::item::MaterialType::IronIngot, 2))
        );
    }
}
