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

    pub fn from_player(transform: &Transform, fps: &FpsCamera, inventory: &Inventory) -> Self {
        Self::new(
            transform.translation,
            fps.yaw,
            fps.pitch,
            inventory.hotbar,
            inventory.main,
            inventory.selected_slot,
        )
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

/// Persisted metadata of a saved world including seed, world name, creation timestamp, and engine version.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorldMetadata {
    pub seed: crate::world::WorldSeed,
    pub world_name: String,
    pub created_at_unix: u64,
    pub last_played_unix: u64,
    pub engine_version: String,
}

impl WorldMetadata {
    pub fn new(seed: crate::world::WorldSeed, world_name: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Self {
            seed,
            world_name: world_name.into(),
            created_at_unix: now,
            last_played_unix: now,
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_disk(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Returns the root data directory for MineRust.
///
/// Priority:
/// 1. `MINERUST_DATA_DIR` environment variable (if set).
/// 2. If running inside an AppImage (`APPIMAGE` or `APPDIR` set):
///    - If `$OWD/saves` exists, uses `$OWD` (portable AppImage).
///    - Otherwise, uses OS standard data directory (`~/.local/share/minerust` on Linux).
/// 3. Portable mode: if `./saves` or `./portable` exists in current working directory, uses `.`.
/// 4. OS standard data directory:
///    - Linux: `$XDG_DATA_HOME/minerust` or `~/.local/share/minerust`
///    - macOS: `~/Library/Application Support/minerust`
///    - Windows: `%APPDATA%\minerust`
/// 5. Fallback: `.`
#[must_use]
pub fn get_data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("MINERUST_DATA_DIR") {
        if !dir.is_empty() {
            return std::path::PathBuf::from(dir);
        }
    }

    let in_appimage = std::env::var("APPIMAGE").is_ok() || std::env::var("APPDIR").is_ok();
    if in_appimage {
        if let Ok(owd) = std::env::var("OWD") {
            let owd_path = std::path::PathBuf::from(&owd);
            if owd_path.join("saves").exists() || owd_path.join("portable").exists() {
                return owd_path;
            }
        }
    } else {
        let local_saves = Path::new("saves");
        let local_portable = Path::new("portable");
        if local_saves.exists() || local_portable.exists() {
            return std::path::PathBuf::from(".");
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        return std::path::PathBuf::from(appdata).join("minerust");
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("minerust");
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    if let Ok(home) = std::env::var("HOME") {
        let xdg = std::env::var("XDG_DATA_HOME").map_or_else(
            |_| std::path::PathBuf::from(home).join(".local").join("share"),
            std::path::PathBuf::from,
        );
        return xdg.join("minerust");
    }

    std::path::PathBuf::from(".")
}

/// Returns the directory where saved worlds are stored.
#[must_use]
pub fn get_saves_dir() -> std::path::PathBuf {
    get_data_dir().join("saves")
}

/// Returns the path to the tracked last played world file.
#[must_use]
pub fn get_last_world_file() -> std::path::PathBuf {
    get_saves_dir().join("last_world.txt")
}

/// Returns the directory for a specific saved world seed.
#[must_use]
pub fn get_world_dir(seed: u64) -> std::path::PathBuf {
    get_saves_dir().join(format!("world_{seed}"))
}

/// Returns the path to the graphics and user settings file.
#[must_use]
pub fn get_settings_path() -> std::path::PathBuf {
    get_data_dir().join("settings.json")
}

/// Legacy relative path constant for backward compatibility.
pub const LAST_WORLD_FILE: &str = "saves/last_world.txt";

/// Returns the seed of the most recently played world, if any exists.
#[must_use]
pub fn get_latest_saved_world() -> Option<u64> {
    // 1. Try reading the tracked last_world.txt (both new path and legacy path)
    let last_world_path = get_last_world_file();
    let content = std::fs::read_to_string(&last_world_path)
        .or_else(|_| std::fs::read_to_string(LAST_WORLD_FILE));

    if let Ok(text) = content {
        if let Ok(seed) = text.trim().parse::<u64>() {
            let world_path = get_world_dir(seed);
            if world_path.exists() {
                return Some(seed);
            }
        }
    }

    // 2. Scan saves/ directory for world_<seed> directories
    let saves_dir = get_saves_dir();
    let Ok(entries) = std::fs::read_dir(&saves_dir) else {
        return None;
    };

    let mut best_world: Option<(u64, std::time::SystemTime)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if let Some(seed_str) = file_name.strip_prefix("world_") {
                    if let Ok(seed) = seed_str.parse::<u64>() {
                        let meta_path = path.join("meta.json");
                        let meta_time = WorldMetadata::load_from_disk(&meta_path).ok().map(|m| {
                            std::time::UNIX_EPOCH
                                + std::time::Duration::from_secs(m.last_played_unix)
                        });
                        let mtime = meta_time.unwrap_or_else(|| {
                            entry
                                .metadata()
                                .and_then(|m| m.modified())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                        });
                        if best_world.as_ref().is_none_or(|b| mtime > b.1) {
                            best_world = Some((seed, mtime));
                        }
                    }
                }
            }
        }
    }

    best_world.map(|(seed, _)| seed)
}

/// Lists all saved worlds on disk with their metadata sorted by most recently played.
#[must_use]
pub fn get_saved_worlds_metadata() -> Vec<WorldMetadata> {
    let saves_dir = get_saves_dir();
    let Ok(entries) = std::fs::read_dir(&saves_dir) else {
        return Vec::new();
    };

    let mut worlds = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if let Some(seed_str) = file_name.strip_prefix("world_") {
                    if let Ok(seed) = seed_str.parse::<u64>() {
                        let meta_path = path.join("meta.json");
                        let meta = WorldMetadata::load_from_disk(&meta_path).unwrap_or_else(|_| {
                            let mtime = entry
                                .metadata()
                                .and_then(|m| m.modified())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            let secs = mtime
                                .duration_since(std::time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_secs());
                            WorldMetadata {
                                seed: crate::world::WorldSeed(seed),
                                world_name: format!("World {seed}"),
                                created_at_unix: secs,
                                last_played_unix: secs,
                                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                            }
                        });
                        worlds.push(meta);
                    }
                }
            }
        }
    }

    worlds.sort_by_key(|a| std::cmp::Reverse(a.last_played_unix));
    worlds
}

/// Records the last played world seed to disk and updates world metadata.
pub fn set_last_played_world(seed: u64) {
    let saves_dir = get_saves_dir();
    let _ = std::fs::create_dir_all(&saves_dir);
    let _ = std::fs::write(get_last_world_file(), seed.to_string());

    let meta_path = get_world_dir(seed).join("meta.json");
    let mut meta = WorldMetadata::load_from_disk(&meta_path).unwrap_or_else(|_| {
        WorldMetadata::new(crate::world::WorldSeed(seed), format!("World {seed}"))
    });
    meta.last_played_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let _ = meta.save_to_disk(&meta_path);
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
    if world.chunks.is_empty() {
        return;
    }

    if timer.0.tick(time.delta()).just_finished() {
        set_last_played_world(world.seed.0);

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
            let data = PlayerSaveData::from_player(transform, fps, &inventory);
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
        if world.chunks.is_empty() {
            return;
        }

        tracing::info!("App exit detected! Flushing all world chunks and player data to disk...");
        set_last_played_world(world.seed.0);

        if let Ok((transform, fps)) = player_query.single() {
            let data = PlayerSaveData::from_player(transform, fps, &inventory);
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
        hotbar[2] = Some(ItemStack::tool(
            crate::item::ToolType::Pickaxe,
            crate::item::ToolTier::Iron,
        ));

        let mut main = [None; MAIN_INVENTORY_SLOTS];
        main[5] = Some(ItemStack::new(BlockType::DiamondOre, 3));
        main[6] = Some(ItemStack::material(crate::item::MaterialType::Coal, 16));

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

    #[test]
    fn test_world_metadata_serialization_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "minerust_meta_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        let meta_path = temp_dir.join("meta.json");

        let original = WorldMetadata::new(crate::world::WorldSeed(987654321), "Survival Island");
        original
            .save_to_disk(&meta_path)
            .expect("saving metadata should succeed");

        let loaded =
            WorldMetadata::load_from_disk(&meta_path).expect("loading metadata should succeed");

        assert_eq!(original, loaded);
        assert_eq!(loaded.world_name, "Survival Island");
        assert_eq!(loaded.seed.0, 987654321);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_data_and_saves_dir_resolution() {
        let saves_dir = get_saves_dir();
        assert!(saves_dir.ends_with("saves"));

        let settings_path = get_settings_path();
        assert!(settings_path.ends_with("settings.json"));

        let world_dir = get_world_dir(12345);
        assert!(world_dir.ends_with("world_12345"));
        assert!(world_dir.starts_with(&saves_dir));

        let last_world = get_last_world_file();
        assert!(last_world.ends_with("last_world.txt"));
    }
}
