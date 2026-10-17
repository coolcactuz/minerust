#![forbid(unsafe_code)]

use std::collections::HashMap;

use bevy::prelude::*;

use crate::block::BlockType;
use crate::inventory::{ItemStack, ItemType, MaterialType};
use crate::world::WorldGrid;

pub mod ui;

pub const COOK_DURATION: f32 = 10.0;

/// Returns the smelting result for a given ingredient item.
#[must_use]
pub fn get_smelt_result(item: ItemType) -> Option<ItemType> {
    match item {
        ItemType::Block(BlockType::IronOre) => Some(ItemType::Material(MaterialType::IronIngot)),
        ItemType::Block(BlockType::GoldOre) => Some(ItemType::Material(MaterialType::GoldIngot)),
        ItemType::Block(BlockType::Sand) => Some(ItemType::Block(BlockType::Glass)),
        ItemType::Block(BlockType::Cobblestone) => Some(ItemType::Block(BlockType::Stone)),
        ItemType::Block(BlockType::Wood) => Some(ItemType::Material(MaterialType::Coal)),
        _ => None,
    }
}

/// Returns the burn duration in seconds that an item provides as fuel.
#[must_use]
pub fn get_fuel_burn_time(item: ItemType) -> Option<f32> {
    match item {
        ItemType::Material(MaterialType::Coal) => Some(80.0), // Smelts 8 items
        ItemType::Block(BlockType::Wood | BlockType::Planks) => Some(15.0), // Smelts 1.5 items
        ItemType::Material(MaterialType::Stick) => Some(5.0), // Smelts 0.5 items
        _ => None,
    }
}

/// Simulation state of an active furnace.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FurnaceData {
    pub input: Option<ItemStack>,
    pub fuel: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub burn_time_remaining: f32,
    pub burn_time_total: f32,
    pub cook_progress: f32,
}

impl FurnaceData {
    #[must_use]
    pub fn is_burning(&self) -> bool {
        self.burn_time_remaining > 0.0
    }

    /// Progress normalized from 0.0 to 1.0 of the current fuel consumption.
    #[must_use]
    pub fn flame_ratio(&self) -> f32 {
        if self.burn_time_total <= 0.0 {
            0.0
        } else {
            (self.burn_time_remaining / self.burn_time_total).clamp(0.0, 1.0)
        }
    }

    /// Progress normalized from 0.0 to 1.0 of the item smelting cycle.
    #[must_use]
    pub fn cook_ratio(&self) -> f32 {
        (self.cook_progress / COOK_DURATION).clamp(0.0, 1.0)
    }

    /// Checks if the furnace can currently smelt its input into its output slot.
    #[must_use]
    pub fn can_smelt(&self) -> bool {
        let Some(input_stack) = self.input else {
            return false;
        };
        let Some(result_item) = get_smelt_result(input_stack.item) else {
            return false;
        };

        match self.output {
            None => true,
            Some(out) => out.item == result_item && out.count < out.item.max_stack_size(),
        }
    }
}

/// World-wide container for all furnaces keyed by their 3D voxel position.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct FurnaceStorage {
    pub furnaces: HashMap<IVec3, FurnaceData>,
}

impl FurnaceStorage {
    pub fn get_or_create(&mut self, pos: IVec3) -> &mut FurnaceData {
        self.furnaces.entry(pos).or_default()
    }

    pub fn remove_and_dump(&mut self, pos: IVec3) -> Vec<ItemStack> {
        self.furnaces.remove(&pos).map_or_else(Vec::new, |f| {
            [f.input, f.fuel, f.output].into_iter().flatten().collect()
        })
    }
}

/// Tracks which furnace GUI is currently opened by the player.
#[derive(Resource, Default, Debug, Clone)]
pub struct FurnaceOpenState {
    pub open_pos: Option<IVec3>,
}

impl FurnaceOpenState {
    pub fn is_open(&self) -> bool {
        self.open_pos.is_some()
    }

    pub fn close(&mut self) {
        self.open_pos = None;
    }
}

/// Simulates fuel burning, item cooking, and block appearance updates for all active furnaces.
pub fn furnace_simulation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut storage: ResMut<FurnaceStorage>,
    mut world: Option<ResMut<WorldGrid>>,
    sfx: Option<Res<crate::audio::SoundEffects>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let mut state_changes: Vec<(IVec3, BlockType)> = Vec::new();

    for (&pos, furnace) in &mut storage.furnaces {
        let can_cook = furnace.can_smelt();

        // 1. If currently burning, consume burn time
        if furnace.burn_time_remaining > 0.0 {
            furnace.burn_time_remaining = (furnace.burn_time_remaining - dt).max(0.0);

            if can_cook {
                furnace.cook_progress += dt;
                if furnace.cook_progress >= COOK_DURATION {
                    furnace.cook_progress = 0.0;
                    if let Some(ref mut input_stack) = furnace.input {
                        if let Some(result_item) = get_smelt_result(input_stack.item) {
                            if input_stack.count <= 1 {
                                furnace.input = None;
                            } else {
                                input_stack.count -= 1;
                            }

                            if let Some(ref mut out) = furnace.output {
                                out.count += 1;
                            } else {
                                furnace.output = Some(ItemStack::new(result_item, 1));
                            }
                        }
                    }
                }
            } else {
                furnace.cook_progress = (furnace.cook_progress - dt * 2.0).max(0.0);
            }
        } else {
            // Not currently burning
            furnace.cook_progress = (furnace.cook_progress - dt * 2.0).max(0.0);

            // If we have items ready to smelt and fuel available, ignite!
            if can_cook {
                if let Some(ref mut fuel_stack) = furnace.fuel {
                    if let Some(burn_duration) = get_fuel_burn_time(fuel_stack.item) {
                        furnace.burn_time_remaining = burn_duration;
                        furnace.burn_time_total = burn_duration;
                        if fuel_stack.count <= 1 {
                            furnace.fuel = None;
                        } else {
                            fuel_stack.count -= 1;
                        }
                        if let Some(ref sfx) = sfx {
                            crate::audio::play_sound(&mut commands, &sfx.furnace_crackle, 0.75);
                        }
                    }
                }
            }
        }

        // 2. Synchronize voxel world block state (Furnace vs LitFurnace)
        let is_now_burning = furnace.burn_time_remaining > 0.0;
        let expected_block = if is_now_burning {
            BlockType::LitFurnace
        } else {
            BlockType::Furnace
        };
        state_changes.push((pos, expected_block));
    }

    if let Some(ref mut wg) = world {
        for (pos, expected) in state_changes {
            let current = wg.get_block(pos);
            if (current == BlockType::Furnace || current == BlockType::LitFurnace)
                && current != expected
            {
                wg.set_block(pos, expected);
            }
        }
    }
}

pub struct FurnacePlugin;

impl Plugin for FurnacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FurnaceStorage>()
            .init_resource::<FurnaceOpenState>()
            .add_systems(Startup, ui::setup_furnace_ui)
            .add_systems(
                Update,
                (
                    furnace_simulation_system,
                    ui::furnace_interaction_system,
                    ui::furnace_ui_update_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smelting_recipes_coverage() {
        assert_eq!(
            get_smelt_result(ItemType::Block(BlockType::IronOre)),
            Some(ItemType::Material(MaterialType::IronIngot))
        );
        assert_eq!(
            get_smelt_result(ItemType::Block(BlockType::GoldOre)),
            Some(ItemType::Material(MaterialType::GoldIngot))
        );
        assert_eq!(
            get_smelt_result(ItemType::Block(BlockType::Sand)),
            Some(ItemType::Block(BlockType::Glass))
        );
        assert_eq!(
            get_smelt_result(ItemType::Block(BlockType::Cobblestone)),
            Some(ItemType::Block(BlockType::Stone))
        );
        assert_eq!(
            get_smelt_result(ItemType::Block(BlockType::Wood)),
            Some(ItemType::Material(MaterialType::Coal))
        );
        assert_eq!(get_smelt_result(ItemType::Block(BlockType::Dirt)), None);
    }

    #[test]
    fn test_fuel_burn_times() {
        assert_eq!(
            get_fuel_burn_time(ItemType::Material(MaterialType::Coal)),
            Some(80.0)
        );
        assert_eq!(
            get_fuel_burn_time(ItemType::Block(BlockType::Wood)),
            Some(15.0)
        );
        assert_eq!(
            get_fuel_burn_time(ItemType::Block(BlockType::Planks)),
            Some(15.0)
        );
        assert_eq!(
            get_fuel_burn_time(ItemType::Material(MaterialType::Stick)),
            Some(5.0)
        );
        assert_eq!(get_fuel_burn_time(ItemType::Block(BlockType::Stone)), None);
    }

    #[test]
    fn test_furnace_data_can_smelt() {
        let mut f = FurnaceData::default();
        assert!(!f.can_smelt());

        f.input = Some(ItemStack::block(BlockType::IronOre, 5));
        assert!(f.can_smelt());

        f.output = Some(ItemStack::material(MaterialType::IronIngot, 64));
        assert!(!f.can_smelt(), "Full output slot cannot accept more");

        f.output = Some(ItemStack::material(MaterialType::IronIngot, 63));
        assert!(f.can_smelt());
    }

    #[test]
    fn test_furnace_remove_and_dump() {
        let mut storage = FurnaceStorage::default();
        let pos = IVec3::new(5, 60, 5);
        {
            let f = storage.get_or_create(pos);
            f.input = Some(ItemStack::block(BlockType::Sand, 4));
            f.fuel = Some(ItemStack::material(MaterialType::Coal, 2));
            f.output = Some(ItemStack::block(BlockType::Glass, 1));
        }

        let dumped = storage.remove_and_dump(pos);
        assert_eq!(dumped.len(), 3);
        assert_eq!(dumped[0], ItemStack::block(BlockType::Sand, 4));
        assert_eq!(dumped[1], ItemStack::material(MaterialType::Coal, 2));
        assert_eq!(dumped[2], ItemStack::block(BlockType::Glass, 1));
        assert!(storage.furnaces.is_empty());
    }

    #[test]
    fn test_furnace_simulation_cycle_and_block_state() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<FurnaceStorage>();
        app.insert_resource(WorldGrid::new(crate::world::WorldSeed(12345)));
        app.add_systems(Update, furnace_simulation_system);

        let furnace_pos = IVec3::new(10, 64, 10);
        // Place a furnace block in the world
        {
            let mut world = app.world_mut().resource_mut::<WorldGrid>();
            world.chunks.insert(IVec2::ZERO, crate::chunk::Chunk::new());
            world.set_block(furnace_pos, BlockType::Furnace);
        }

        // Add 2 Iron Ore and 1 Coal in the furnace storage
        {
            let mut storage = app.world_mut().resource_mut::<FurnaceStorage>();
            let furnace = storage.get_or_create(furnace_pos);
            furnace.input = Some(ItemStack::block(BlockType::IronOre, 2));
            furnace.fuel = Some(ItemStack::material(MaterialType::Coal, 1));
        }

        // Advance time: delta 1.0s -> furnace ignites, consumes coal
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(1));
        }
        app.update();

        {
            let storage = app.world().resource::<FurnaceStorage>();
            let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
            assert!(furnace.is_burning());
            assert_eq!(furnace.fuel, None, "Coal consumed for 80s burn time");

            let world = app.world().resource::<WorldGrid>();
            assert_eq!(world.get_block(furnace_pos), BlockType::LitFurnace);
        }

        // Advance time: delta 1.0s -> active cooking progresses
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(1));
        }
        app.update();

        {
            let storage = app.world().resource::<FurnaceStorage>();
            let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
            assert!(furnace.cook_progress > 0.0);
        }

        // Advance time: delta 9.5s -> total cooking > 10.0s -> first smelt completes!
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(9500));
        }
        app.update();

        {
            let storage = app.world().resource::<FurnaceStorage>();
            let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
            assert_eq!(
                furnace.output,
                Some(ItemStack::material(MaterialType::IronIngot, 1))
            );
            assert_eq!(furnace.input, Some(ItemStack::block(BlockType::IronOre, 1)));
            assert!(furnace.is_burning());
        }

        // Advance time: delta 10.0s -> second smelt completes, input becomes None!
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(10));
        }
        app.update();

        {
            let storage = app.world().resource::<FurnaceStorage>();
            let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
            assert_eq!(
                furnace.output,
                Some(ItemStack::material(MaterialType::IronIngot, 2))
            );
            assert_eq!(furnace.input, None);
        }

        // Advance time: delta 100.0s -> coal runs out, furnace cools down and reverts to BlockType::Furnace
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(100));
        }
        app.update();

        {
            let storage = app.world().resource::<FurnaceStorage>();
            let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
            assert!(!furnace.is_burning());
            assert_eq!(furnace.cook_progress, 0.0);

            let world = app.world().resource::<WorldGrid>();
            assert_eq!(world.get_block(furnace_pos), BlockType::Furnace);
        }
    }

    #[test]
    fn test_furnace_simulation_respects_max_stack() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<FurnaceStorage>();
        app.add_systems(Update, furnace_simulation_system);

        let furnace_pos = IVec3::new(0, 50, 0);
        {
            let mut storage = app.world_mut().resource_mut::<FurnaceStorage>();
            let furnace = storage.get_or_create(furnace_pos);
            furnace.input = Some(ItemStack::block(BlockType::Cobblestone, 5));
            furnace.fuel = Some(ItemStack::material(MaterialType::Coal, 1));
            furnace.output = Some(ItemStack::block(BlockType::Stone, 64)); // Full!
        }

        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs(12));
        }
        app.update();

        let storage = app.world().resource::<FurnaceStorage>();
        let furnace = storage.furnaces.get(&furnace_pos).expect("Furnace exists");
        assert_eq!(
            furnace.input,
            Some(ItemStack::block(BlockType::Cobblestone, 5))
        );
        assert_eq!(furnace.output, Some(ItemStack::block(BlockType::Stone, 64)));
        assert_eq!(
            furnace.fuel,
            Some(ItemStack::material(MaterialType::Coal, 1))
        );
        assert_eq!(furnace.cook_progress, 0.0);
    }
}
