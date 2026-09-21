use bevy::prelude::*;

/// Ordered execution stages for game simulation and rendering systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelStage {
    InputHandling,
    PlayerPhysics,
    FluidSimulation,
    WorldStreaming,
    MeshBuilding,
}
