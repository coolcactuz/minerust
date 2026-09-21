use std::path::Path;

use bevy::app::AppExit;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::camera::FpsCamera;
use crate::inventory::{HOTBAR_SLOTS, Inventory, ItemStack, MAIN_INVENTORY_SLOTS};
use crate::world::WorldGrid;

/// Persisted state of the player including 3D position, camera orientation, and inventory slots.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerSaveData {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub hotbar: [Option<ItemStack>; HOTBAR_SLOTS],
    pub main: [Option<ItemStack>; MAIN_INVENTORY_SLOTS],
    pub selected_slot: usize,
}

impl PlayerSaveData {
    pub fn new(
        position: Vec3,
        yaw: f32,
        pitch: f32,
        hotbar: [Option<ItemStack>; HOTBAR_SLOTS],
        main: [Option<ItemStack>; MAIN_INVENTORY_SLOTS],
        selected_slot: usize,
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            yaw,
            pitch,
            hotbar,
            main,
            selected_slot,
        }
    }
}

/// Saves the player's position, camera orientation, and inventory to JSON on disk.
pub fn save_player_to_disk(path: &Path, data: &PlayerSaveData) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Loads the player's saved state from a JSON file on disk.
pub fn load_player_from_disk(path: &Path) -> Result<PlayerSaveData, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Timer for periodic auto-saving during gameplay.
#[derive(Resource)]
pub struct AutoSaveTimer(pub Timer);

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

/// Auto-saves dirty chunks and player position/inventory every 2 seconds during gameplay.
pub fn autosave_system(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    mut world: ResMut<WorldGrid>,
    inventory: Res<Inventory>,
    player_query: Query<(&Transform, &FpsCamera)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // 1. Flush any modified chunks that have not been written to disk yet
        match world.save_all_dirty_chunks() {
            Ok(count) if count > 0 => {
                tracing::debug!("Auto-saved {count} modified chunks to disk");
            }
            Err(e) => {
                tracing::warn!("Failed to auto-save dirty chunks: {e}");
            }
            _ => {}
        }

        // 2. Auto-save player state (position, rotation, inventory)
        if let Ok((transform, fps)) = player_query.single() {
            let data = PlayerSaveData::new(
                transform.translation,
                fps.yaw,
                fps.pitch,
                inventory.hotbar,
                inventory.main,
                inventory.selected_slot,
            );
            let path = world.player_save_path();
            if let Err(e) = save_player_to_disk(&path, &data) {
                tracing::warn!("Failed to auto-save player data to {path:?}: {e}");
            }
        }
    }
}

/// Immediate flush system that saves all modified chunks and player data when the app is exiting.
pub fn save_on_exit_system(
    exit_reader: MessageReader<AppExit>,
    mut world: ResMut<WorldGrid>,
    inventory: Res<Inventory>,
    player_query: Query<(&Transform, &FpsCamera)>,
) {
    if !exit_reader.is_empty() {
        tracing::info!("App exit detected! Flushing all world chunks and player data to disk...");

        if let Ok((transform, fps)) = player_query.single() {
            let data = PlayerSaveData::new(
                transform.translation,
                fps.yaw,
                fps.pitch,
                inventory.hotbar,
                inventory.main,
                inventory.selected_slot,
            );
            let path = world.player_save_path();
            if let Err(e) = save_player_to_disk(&path, &data) {
                tracing::warn!("Failed to save player data on exit: {e}");
            }
        }

        if let Err(e) = world.save_all_modified() {
            tracing::warn!("Failed to save modified chunks on exit: {e}");
        }
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoSaveTimer>()
            .add_systems(PostUpdate, (autosave_system, save_on_exit_system));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_player_save_data_serialization_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "minerust_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let save_path = temp_dir.join("player.json");

        let mut hotbar = [None; HOTBAR_SLOTS];
        hotbar[0] = Some(ItemStack::new(BlockType::Wood, 14));
        hotbar[1] = Some(ItemStack::new(BlockType::Cobblestone, 64));

        let mut main = [None; MAIN_INVENTORY_SLOTS];
        main[5] = Some(ItemStack::new(BlockType::DiamondOre, 3));

        let original =
            PlayerSaveData::new(Vec3::new(12.5, 68.0, -99.2), 1.23, -0.45, hotbar, main, 2);

        save_player_to_disk(&save_path, &original).expect("saving player data should succeed");
        let loaded = load_player_from_disk(&save_path).expect("loading player data should succeed");

        assert_eq!(original, loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_world_dirty_chunks_save_and_reload() {
        use crate::chunk::Chunk;
        use crate::coords::BlockPos;
        use crate::world::WorldSeed;

        let temp_dir = std::env::temp_dir().join(format!(
            "minerust_world_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let mut world = WorldGrid::new(WorldSeed(42));
        world.save_dir = temp_dir.clone();
        world.chunks.insert(IVec2::ZERO, Chunk::new());

        let target_pos = BlockPos::new(5, 50, 5);
        world.set_block(target_pos, BlockType::Glass);

        assert!(world.dirty_chunks.contains(&IVec2::ZERO));
        assert!(world.modified_chunks.contains(&IVec2::ZERO));

        let saved_count = world
            .save_all_dirty_chunks()
            .expect("saving dirty chunks should succeed");
        assert_eq!(saved_count, 1);
        assert!(world.dirty_chunks.is_empty());

        let loaded = WorldGrid::load_chunk_from_disk_path(&temp_dir, IVec2::ZERO)
            .expect("loading chunk from disk should succeed");
        let (_, local) = target_pos.to_chunk_and_local();
        assert_eq!(loaded.get_local(local), BlockType::Glass);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
