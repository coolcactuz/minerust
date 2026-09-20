use std::collections::{HashSet, VecDeque};
use bevy::prelude::*;

use crate::block::BlockType;
use crate::camera::FpsCamera;
use crate::chunk::CHUNK_HEIGHT;
use crate::world::{chunk_distance_sq_to_player, update_chunk_mesh, WorldGrid, SEA_LEVEL};

pub const MAX_FLUID_TICKS_PER_FRAME: usize = 48;

#[derive(Resource)]
pub struct FluidSimulation {
    pub queue: VecDeque<IVec3>,
    pub queued: HashSet<IVec3>,
    pub sources: HashSet<IVec3>,
    pub timer: Timer,
}

impl Default for FluidSimulation {
    fn default() -> Self {
        Self {
            queue: VecDeque::with_capacity(256),
            queued: HashSet::with_capacity(256),
            sources: HashSet::with_capacity(256),
            // Run fluid ticks every 1.0 second (1 mesh/block advance per second)
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

impl FluidSimulation {
    pub fn schedule(&mut self, pos: IVec3) {
        if pos.y <= 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return;
        }
        if self.queued.insert(pos) {
            self.queue.push_back(pos);
        }
    }

    pub fn schedule_neighbors(&mut self, pos: IVec3) {
        self.schedule(pos);
        self.schedule(pos + IVec3::Y);
        self.schedule(pos - IVec3::Y);
        self.schedule(pos + IVec3::X);
        self.schedule(pos - IVec3::X);
        self.schedule(pos + IVec3::Z);
        self.schedule(pos - IVec3::Z);
    }
}

/// Executes a single batch of fluid simulation ticks, propagating water downwards
/// (waterfalls), horizontally into canals and excavated seabed holes, and drying up disconnected water.
pub fn process_fluid_step(
    world: &mut WorldGrid,
    fluid_sim: &mut FluidSimulation,
) -> Vec<IVec2> {
    let mut dirty_coords = Vec::new();
    let batch_size = fluid_sim.queue.len().min(MAX_FLUID_TICKS_PER_FRAME);

    for _ in 0..batch_size {
        let pos = match fluid_sim.queue.pop_front() {
            Some(p) => p,
            None => break,
        };
        fluid_sim.queued.remove(&pos);

        let current_block = world.get_block(pos);

        // CASE 1: Position is AIR - check if water should flow into it
        if current_block == BlockType::Air {
            let mut should_fill_water = false;

            // 1. Water directly above: falls down by gravity (waterfall!)
            let above_pos = pos + IVec3::Y;
            if world.get_block(above_pos) == BlockType::Water {
                should_fill_water = true;
            }

            // 2. Horizontal neighbors with water
            if !should_fill_water {
                let neighbors = [
                    pos + IVec3::X,
                    pos - IVec3::X,
                    pos + IVec3::Z,
                    pos - IVec3::Z,
                ];

                for npos in neighbors {
                    if world.get_block(npos) == BlockType::Water {
                        // At or below sea level: natural hole filling (seabed, lakes, excavated channels)
                        if pos.y <= SEA_LEVEL as i32 {
                            should_fill_water = true;
                            break;
                        } else {
                            // Above sea level: water flows horizontally if resting on a solid surface
                            let below_npos = npos - IVec3::Y;
                            if world.is_solid_at(below_npos) || below_npos.y <= 0 {
                                should_fill_water = true;
                                break;
                            }
                        }
                    }
                }
            }

            if should_fill_water {
                let affected = world.set_block(pos, BlockType::Water);
                dirty_coords.extend(affected);

                // Water just filled this block:
                // Check if it can cascade downward to create a waterfall!
                let below_pos = pos - IVec3::Y;
                if below_pos.y > 0 && world.get_block(below_pos) == BlockType::Air {
                    fluid_sim.schedule(below_pos);
                }

                // Also check if horizontal neighbors can now be filled
                let neighbors = [
                    pos + IVec3::X,
                    pos - IVec3::X,
                    pos + IVec3::Z,
                    pos - IVec3::Z,
                ];
                for npos in neighbors {
                    if world.get_block(npos) == BlockType::Air {
                        fluid_sim.schedule(npos);
                    }
                }
            }
        }
        // CASE 2: Position is WATER - check downstream cascade and drying up if supply is cut
        else if current_block == BlockType::Water {
            // A. Check if water can fall downward into air
            let below_pos = pos - IVec3::Y;
            if below_pos.y > 0 && world.get_block(below_pos) == BlockType::Air {
                fluid_sim.schedule(below_pos);
            }

            // B. Above sea level, check if supply was cut off (e.g. player plugged the source with a block)
            if pos.y > SEA_LEVEL as i32 && !fluid_sim.sources.contains(&pos) {
                let above_pos = pos + IVec3::Y;
                let has_water_above = world.get_block(above_pos) == BlockType::Water;

                let has_horizontal_water = [
                    pos + IVec3::X,
                    pos - IVec3::X,
                    pos + IVec3::Z,
                    pos - IVec3::Z,
                ]
                .iter()
                .any(|&np| world.get_block(np) == BlockType::Water);

                // If disconnected from any water above or horizontally, waterfall dries up
                if !has_water_above && !has_horizontal_water {
                    fluid_sim.sources.remove(&pos);
                    let affected = world.set_block(pos, BlockType::Air);
                    dirty_coords.extend(affected);

                    // Notify block below that water supply has dried up
                    fluid_sim.schedule(below_pos);
                    for &np in &[
                        pos + IVec3::X,
                        pos - IVec3::X,
                        pos + IVec3::Z,
                        pos - IVec3::Z,
                    ] {
                        if world.get_block(np) == BlockType::Water {
                            fluid_sim.schedule(np);
                        }
                    }
                }
            }
        }
    }

    dirty_coords
}

/// Bevy system that periodically runs fluid simulation ticks and updates dirty chunk meshes
pub fn fluid_simulation_system(
    time: Res<Time>,
    mut fluid_sim: ResMut<FluidSimulation>,
    mut world: ResMut<WorldGrid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dev_settings: Option<Res<crate::menu::DevSettings>>,
    camera_query: Query<&Transform, With<FpsCamera>>,
) {
    fluid_sim.timer.tick(time.delta());
    if !fluid_sim.timer.just_finished() || fluid_sim.queue.is_empty() {
        return;
    }

    let mut dirty_coords = process_fluid_step(&mut world, &mut fluid_sim);

    // Re-mesh any chunks modified by fluid simulation
    if !dirty_coords.is_empty() {
        dirty_coords.sort_unstable_by_key(|c| (c.x, c.y));
        dirty_coords.dedup();

        let player_pos = camera_query.single().ok().map_or(Vec3::ZERO, |t| t.translation);
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
            update_chunk_mesh(
                &coord,
                &mut commands,
                &mut world,
                &mut meshes,
                &mut materials,
                max_y_skip,
                greedy,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    use crate::world::WorldSeed;

    #[test]
    fn test_seabed_hole_filling() {
        let mut world = WorldGrid::new(WorldSeed(12345));
        let chunk_coord = IVec2::new(0, 0);
        let mut chunk = Chunk::new();

        // Seabed at y = 30 with Sand, and Water at y = 31
        chunk.set(5, 30, 5, BlockType::Sand);
        chunk.set(5, 31, 5, BlockType::Water);
        world.chunks.insert(chunk_coord, chunk);

        let mut fluid_sim = FluidSimulation::default();

        // Player breaks the sand block at (5, 30, 5)
        let hit_block = IVec3::new(5, 30, 5);
        world.set_block(hit_block, BlockType::Air);
        fluid_sim.schedule_neighbors(hit_block);

        assert_eq!(world.get_block(hit_block), BlockType::Air);

        // Run fluid simulation step
        process_fluid_step(&mut world, &mut fluid_sim);

        // Water directly above must have filled the excavated seabed hole!
        assert_eq!(world.get_block(hit_block), BlockType::Water);
    }

    #[test]
    fn test_waterfall_cascades_downward() {
        let mut world = WorldGrid::new(WorldSeed(12345));
        let chunk_coord = IVec2::new(0, 0);
        let mut chunk = Chunk::new();

        // Place stone floor at y = 70, air from y = 71 to 75
        chunk.set(5, 70, 5, BlockType::Stone);
        world.chunks.insert(chunk_coord, chunk);

        let mut fluid_sim = FluidSimulation::default();

        // Water source placed at cliff top (5, 75, 5)
        let source_pos = IVec3::new(5, 75, 5);
        world.set_block(source_pos, BlockType::Water);
        fluid_sim.sources.insert(source_pos);
        fluid_sim.schedule(source_pos);

        // Process fluid ticks to let the waterfall cascade down (1 block per tick)
        for _ in 0..10 {
            process_fluid_step(&mut world, &mut fluid_sim);
        }

        // Entire vertical column from 75 down to 71 must now be filled with cascading water!
        for y in 71..=75 {
            assert_eq!(
                world.get_block(IVec3::new(5, y, 5)),
                BlockType::Water,
                "Waterfall should be water at y = {}",
                y
            );
        }
        // Floor at 70 remains solid Stone
        assert_eq!(world.get_block(IVec3::new(5, 70, 5)), BlockType::Stone);
    }

    #[test]
    fn test_waterfall_dries_up_when_plugged() {
        let mut world = WorldGrid::new(WorldSeed(12345));
        let chunk_coord = IVec2::new(0, 0);
        let mut chunk = Chunk::new();

        // Create an active waterfall column above sea level (y = 70..=72 > SEA_LEVEL = 64)
        for y in 70..=72 {
            chunk.set(5, y, 5, BlockType::Water);
        }
        world.chunks.insert(chunk_coord, chunk);

        let mut fluid_sim = FluidSimulation::default();
        fluid_sim.sources.insert(IVec3::new(5, 72, 5));

        // Player plugs the waterfall source at (5, 72, 5) with a Stone block
        let source_pos = IVec3::new(5, 72, 5);
        world.set_block(source_pos, BlockType::Stone);
        fluid_sim.sources.remove(&source_pos);
        fluid_sim.schedule_neighbors(source_pos);

        // Process fluid ticks: without a water supply above sea level, the waterfall must dry up
        for _ in 0..10 {
            process_fluid_step(&mut world, &mut fluid_sim);
        }

        // Waterfall stream below the plug dried up back to Air!
        assert_eq!(world.get_block(IVec3::new(5, 71, 5)), BlockType::Air);
        assert_eq!(world.get_block(IVec3::new(5, 70, 5)), BlockType::Air);
    }

    #[test]
    fn test_water_advances_strictly_one_block_per_tick() {
        let mut world = WorldGrid::new(WorldSeed(12345));
        let chunk_coord = IVec2::new(0, 0);
        let mut chunk = Chunk::new();
        chunk.set(5, 70, 5, BlockType::Stone);
        world.chunks.insert(chunk_coord, chunk);

        let mut fluid_sim = FluidSimulation::default();
        let source_pos = IVec3::new(5, 75, 5);
        world.set_block(source_pos, BlockType::Water);
        fluid_sim.sources.insert(source_pos);
        fluid_sim.schedule(source_pos);

        // Tick 1: source schedules below_pos (74)
        process_fluid_step(&mut world, &mut fluid_sim);
        assert_eq!(world.get_block(IVec3::new(5, 74, 5)), BlockType::Air);

        // Tick 2: 74 becomes Water, schedules 73 (73 remains Air in this tick)
        process_fluid_step(&mut world, &mut fluid_sim);
        assert_eq!(world.get_block(IVec3::new(5, 74, 5)), BlockType::Water);
        assert_eq!(world.get_block(IVec3::new(5, 73, 5)), BlockType::Air);

        // Tick 3: 73 becomes Water, schedules 72 (72 remains Air in this tick)
        process_fluid_step(&mut world, &mut fluid_sim);
        assert_eq!(world.get_block(IVec3::new(5, 73, 5)), BlockType::Water);
        assert_eq!(world.get_block(IVec3::new(5, 72, 5)), BlockType::Air);

        // Tick 4: 72 becomes Water, schedules 71 (71 remains Air in this tick)
        process_fluid_step(&mut world, &mut fluid_sim);
        assert_eq!(world.get_block(IVec3::new(5, 72, 5)), BlockType::Water);
        assert_eq!(world.get_block(IVec3::new(5, 71, 5)), BlockType::Air);

        // Tick 5: 71 becomes Water
        process_fluid_step(&mut world, &mut fluid_sim);
        assert_eq!(world.get_block(IVec3::new(5, 71, 5)), BlockType::Water);
    }
}
