#![allow(deprecated)]

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, Interaction};

/// Custom marker struct for UI buttons, automatically requiring interactive Bevy UI components.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
#[allow(deprecated)]
#[require(Node, FocusPolicy::Block, Interaction)]
pub struct Button;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuScreen {
    Main,
    Settings,
    AudioSettings,
    Pause,
    BenchmarkResults,
    None, // In-game gameplay
}

#[derive(Resource)]
pub struct MenuState {
    pub screen: MenuScreen,
    pub previous_screen: MenuScreen,
    pub return_stack: Vec<MenuScreen>,
    pub world_active: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            screen: MenuScreen::Main, // Start in Main Menu!
            previous_screen: MenuScreen::Main,
            return_stack: Vec::new(),
            world_active: false,
        }
    }
}

impl MenuState {
    #[inline]
    pub const fn is_open(&self) -> bool {
        !matches!(self.screen, MenuScreen::None)
    }

    /// Navigates to a new menu screen, recording the previous screen on the return stack.
    pub fn push_screen(&mut self, next: MenuScreen) {
        if self.screen != next {
            self.return_stack.push(self.screen);
            self.previous_screen = self.screen;
            self.screen = next;
        }
    }

    /// Pops the previous screen from the stack, guaranteed to unwind without loops.
    pub fn pop_screen(&mut self) -> MenuScreen {
        while let Some(prev) = self.return_stack.pop() {
            if prev != self.screen {
                self.previous_screen = self.screen;
                self.screen = prev;
                return prev;
            }
        }
        let fallback = if self.world_active {
            MenuScreen::Pause
        } else {
            MenuScreen::Main
        };
        self.previous_screen = fallback;
        self.screen = fallback;
        fallback
    }

    /// Resets the navigation stack and transitions to a root screen (Main, Pause, or None).
    pub fn set_root(&mut self, root: MenuScreen) {
        self.return_stack.clear();
        self.previous_screen = root;
        self.screen = root;
    }
}

/// Global toggle for the real-time engine profiler HUD (accessible via F3).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProfilerState {
    pub visible: bool,
}

const fn default_volume_percent() -> u32 {
    100
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub distance_fog: bool,    // Linear atmospheric distance fog
    pub shadows: bool,         // Directional light shadow maps
    pub fps_cap: Option<u32>,  // None = Uncapped, Some(30)..Some(240)
    pub view_distance: i32,    // 4 to 64 chunks (64m to 1024m)
    pub greedy_meshing: bool,  // Greedy coplanar quad merging beyond greedy threshold
    pub greedy_threshold: i32, // Distance threshold in chunks: 2 (32m), 3 (48m), 4 (64m), 0 (all)
    pub distance_lod: bool,    // Distant Sloped Heightfield LOD
    pub lod_threshold: i32,    // 2 to 32 chunks
    #[serde(default = "default_volume_percent")]
    pub master_volume: u32, // 0 to 100 (%)
    #[serde(default = "default_volume_percent")]
    pub sfx_volume: u32, // 0 to 100 (%)
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            distance_fog: true,
            shadows: true,
            fps_cap: None,
            view_distance: 16,
            greedy_meshing: true,
            greedy_threshold: 2,
            distance_lod: true,
            lod_threshold: 8,
            master_volume: 100,
            sfx_volume: 100,
        }
    }
}

impl GraphicsSettings {
    /// Legacy relative path constant for backward compatibility.
    pub const SAVE_PATH: &'static str = "saves/settings.json";

    #[must_use]
    pub fn load_or_default() -> Self {
        let settings_path = crate::save::get_settings_path();
        let bytes = std::fs::read(&settings_path).or_else(|_| std::fs::read(Self::SAVE_PATH));
        if let Ok(data) = bytes {
            if let Ok(settings) = serde_json::from_slice::<Self>(&data) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let path = crate::save::get_settings_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, json);
        }
    }
}

pub const FPS_CAP_STEPS: &[Option<u32>] = &[
    Some(30),
    Some(60),
    Some(75),
    Some(90),
    Some(120),
    Some(144),
    Some(165),
    Some(240),
    None, // Uncapped
];

pub const VIEW_DISTANCE_STEPS: &[i32] =
    &[4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 56, 64];

impl GraphicsSettings {
    #[must_use]
    pub fn fps_cap_step_index(&self) -> usize {
        FPS_CAP_STEPS
            .iter()
            .position(|&cap| cap == self.fps_cap)
            .unwrap_or(8)
    }

    #[must_use]
    pub fn fps_cap_ratio(&self) -> f32 {
        let idx = self.fps_cap_step_index();
        idx as f32 / (FPS_CAP_STEPS.len() - 1) as f32
    }

    pub fn set_fps_cap_from_ratio(&mut self, ratio: f32) {
        let max_idx = FPS_CAP_STEPS.len() - 1;
        let idx = (ratio * max_idx as f32).round().clamp(0.0, max_idx as f32) as usize;
        self.fps_cap = FPS_CAP_STEPS[idx];
    }

    pub fn step_fps_cap(&mut self, delta: i32) {
        let cur_idx = self.fps_cap_step_index() as i32;
        let max_idx = (FPS_CAP_STEPS.len() - 1) as i32;
        let new_idx = (cur_idx + delta).clamp(0, max_idx) as usize;
        self.fps_cap = FPS_CAP_STEPS[new_idx];
    }

    #[must_use]
    pub fn fps_cap_label(&self) -> String {
        match self.fps_cap {
            None => "FPS Limit: Uncapped (Max FPS)".to_string(),
            Some(cap) => format!("FPS Limit: {} FPS", cap),
        }
    }

    #[must_use]
    pub fn view_distance_step_index(&self) -> usize {
        VIEW_DISTANCE_STEPS
            .iter()
            .position(|&d| d == self.view_distance)
            .unwrap_or_else(|| {
                let mut best_idx = 6;
                let mut best_diff = i32::MAX;
                for (idx, &d) in VIEW_DISTANCE_STEPS.iter().enumerate() {
                    let diff = (d - self.view_distance).abs();
                    if diff < best_diff {
                        best_diff = diff;
                        best_idx = idx;
                    }
                }
                best_idx
            })
    }

    #[must_use]
    pub fn view_distance_ratio(&self) -> f32 {
        let idx = self.view_distance_step_index();
        idx as f32 / (VIEW_DISTANCE_STEPS.len() - 1) as f32
    }

    pub fn set_view_distance_from_ratio(&mut self, ratio: f32) {
        let max_idx = VIEW_DISTANCE_STEPS.len() - 1;
        let idx = (ratio * max_idx as f32).round().clamp(0.0, max_idx as f32) as usize;
        self.view_distance = VIEW_DISTANCE_STEPS[idx];
    }

    pub fn step_view_distance(&mut self, delta: i32) {
        let cur_idx = self.view_distance_step_index() as i32;
        let max_idx = (VIEW_DISTANCE_STEPS.len() - 1) as i32;
        let new_idx = (cur_idx + delta).clamp(0, max_idx) as usize;
        self.view_distance = VIEW_DISTANCE_STEPS[new_idx];
    }

    #[must_use]
    pub fn view_distance_label(&self) -> String {
        format!(
            "Render Distance: {} Chunks ({}m)",
            self.view_distance,
            self.view_distance * 16
        )
    }

    #[must_use]
    pub fn greedy_step_index(&self) -> usize {
        if !self.greedy_meshing {
            return 0;
        }
        GREEDY_MESHING_STEPS
            .iter()
            .position(|&(enabled, thresh)| enabled && thresh == self.greedy_threshold)
            .unwrap_or_else(|| {
                let mut best_idx = 3;
                let mut best_diff = i32::MAX;
                for (idx, &(enabled, thresh)) in GREEDY_MESHING_STEPS.iter().enumerate() {
                    if enabled {
                        let diff = (thresh - self.greedy_threshold).abs();
                        if diff < best_diff {
                            best_diff = diff;
                            best_idx = idx;
                        }
                    }
                }
                best_idx
            })
    }

    #[must_use]
    pub fn greedy_ratio(&self) -> f32 {
        let idx = self.greedy_step_index();
        idx as f32 / (GREEDY_MESHING_STEPS.len() - 1) as f32
    }

    pub fn set_greedy_from_ratio(&mut self, ratio: f32) {
        let max_idx = GREEDY_MESHING_STEPS.len() - 1;
        let idx = (ratio * max_idx as f32).round().clamp(0.0, max_idx as f32) as usize;
        let (enabled, thresh) = GREEDY_MESHING_STEPS[idx];
        self.greedy_meshing = enabled;
        self.greedy_threshold = thresh;
    }

    pub fn step_greedy(&mut self, delta: i32) {
        let cur_idx = self.greedy_step_index() as i32;
        let max_idx = (GREEDY_MESHING_STEPS.len() - 1) as i32;
        let new_idx = (cur_idx + delta).clamp(0, max_idx) as usize;
        let (enabled, thresh) = GREEDY_MESHING_STEPS[new_idx];
        self.greedy_meshing = enabled;
        self.greedy_threshold = thresh;
    }

    pub fn cycle_greedy(&mut self) {
        let cur_idx = self.greedy_step_index();
        let next_idx = (cur_idx + 1) % GREEDY_MESHING_STEPS.len();
        let (enabled, thresh) = GREEDY_MESHING_STEPS[next_idx];
        self.greedy_meshing = enabled;
        self.greedy_threshold = thresh;
    }

    #[must_use]
    pub fn greedy_label(&self) -> String {
        if !self.greedy_meshing {
            "Greedy Meshing: OFF (1x1 Quads)".to_string()
        } else if self.greedy_threshold <= 0 {
            "Greedy Distance: All Chunks (0m)".to_string()
        } else {
            format!(
                "Greedy Distance: > {} Chunks ({}m)",
                self.greedy_threshold,
                self.greedy_threshold * 16
            )
        }
    }

    #[must_use]
    pub fn lod_step_index(&self) -> usize {
        if !self.distance_lod {
            return 0;
        }
        DISTANCE_LOD_STEPS
            .iter()
            .position(|&(enabled, thresh)| enabled && thresh == self.lod_threshold)
            .unwrap_or_else(|| {
                let mut best_idx = 7;
                let mut best_diff = i32::MAX;
                for (idx, &(enabled, thresh)) in DISTANCE_LOD_STEPS.iter().enumerate() {
                    if enabled {
                        let diff = (thresh - self.lod_threshold).abs();
                        if diff < best_diff {
                            best_diff = diff;
                            best_idx = idx;
                        }
                    }
                }
                best_idx
            })
    }

    #[must_use]
    pub fn lod_ratio(&self) -> f32 {
        let idx = self.lod_step_index();
        idx as f32 / (DISTANCE_LOD_STEPS.len() - 1) as f32
    }

    pub fn set_lod_from_ratio(&mut self, ratio: f32) {
        let max_idx = DISTANCE_LOD_STEPS.len() - 1;
        let idx = (ratio * max_idx as f32).round().clamp(0.0, max_idx as f32) as usize;
        let (enabled, thresh) = DISTANCE_LOD_STEPS[idx];
        self.distance_lod = enabled;
        self.lod_threshold = thresh;
    }

    pub fn step_lod(&mut self, delta: i32) {
        let cur_idx = self.lod_step_index() as i32;
        let max_idx = (DISTANCE_LOD_STEPS.len() - 1) as i32;
        let new_idx = (cur_idx + delta).clamp(0, max_idx) as usize;
        let (enabled, thresh) = DISTANCE_LOD_STEPS[new_idx];
        self.distance_lod = enabled;
        self.lod_threshold = thresh;
    }

    pub fn cycle_lod(&mut self) {
        let cur_idx = self.lod_step_index();
        let next_idx = (cur_idx + 1) % DISTANCE_LOD_STEPS.len();
        let (enabled, thresh) = DISTANCE_LOD_STEPS[next_idx];
        self.distance_lod = enabled;
        self.lod_threshold = thresh;
    }

    #[must_use]
    pub fn lod_label(&self) -> String {
        if !self.distance_lod {
            "Distant Sloped LOD: OFF".to_string()
        } else {
            format!(
                "Distant Sloped LOD: > {} Chunks ({}m)",
                self.lod_threshold,
                self.lod_threshold * 16
            )
        }
    }

    #[must_use]
    pub fn master_volume_label(&self) -> String {
        if self.master_volume == 0 {
            "Master Volume: OFF".to_string()
        } else {
            format!("Master Volume: {}%", self.master_volume)
        }
    }

    #[must_use]
    pub fn sfx_volume_label(&self) -> String {
        if self.sfx_volume == 0 {
            "Sound Effects: OFF".to_string()
        } else {
            format!("Sound Effects: {}%", self.sfx_volume)
        }
    }

    #[must_use]
    pub fn master_volume_ratio(&self) -> f32 {
        (self.master_volume as f32 / 100.0).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn sfx_volume_ratio(&self) -> f32 {
        (self.sfx_volume as f32 / 100.0).clamp(0.0, 1.0)
    }

    pub fn set_master_volume_from_ratio(&mut self, ratio: f32) {
        let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u32;
        self.master_volume = ((pct + 2) / 5 * 5).min(100);
    }

    pub fn set_sfx_volume_from_ratio(&mut self, ratio: f32) {
        let pct = (ratio.clamp(0.0, 1.0) * 100.0).round() as u32;
        self.sfx_volume = ((pct + 2) / 5 * 5).min(100);
    }

    pub fn step_master_volume_left(&mut self) {
        self.master_volume = self.master_volume.saturating_sub(5);
    }

    pub fn step_master_volume_right(&mut self) {
        self.master_volume = (self.master_volume + 5).min(100);
    }

    pub fn step_sfx_volume_left(&mut self) {
        self.sfx_volume = self.sfx_volume.saturating_sub(5);
    }

    pub fn step_sfx_volume_right(&mut self) {
        self.sfx_volume = (self.sfx_volume + 5).min(100);
    }
}

pub const GREEDY_MESHING_STEPS: &[(bool, i32)] = &[
    (false, 2), // Index 0: OFF
    (true, 0),  // Index 1: All Chunks (0m)
    (true, 1),  // Index 2: 1 Chunk (16m)
    (true, 2),  // Index 3: 2 Chunks (32m) - Default
    (true, 3),  // Index 4: 3 Chunks (48m)
    (true, 4),  // Index 5: 4 Chunks (64m)
];

pub const DISTANCE_LOD_STEPS: &[(bool, i32)] = &[
    (false, 8), // Index 0: OFF
    (true, 2),  // Index 1: 2 Chunks (32m)
    (true, 3),  // Index 2: 3 Chunks (48m)
    (true, 4),  // Index 3: 4 Chunks (64m)
    (true, 5),  // Index 4: 5 Chunks (80m)
    (true, 6),  // Index 5: 6 Chunks (96m)
    (true, 7),  // Index 6: 7 Chunks (112m)
    (true, 8),  // Index 7: 8 Chunks (128m) - Default
    (true, 9),  // Index 8: 9 Chunks (144m)
    (true, 10), // Index 9: 10 Chunks (160m)
    (true, 12), // Index 10: 12 Chunks (192m)
    (true, 14), // Index 11: 14 Chunks (224m)
    (true, 16), // Index 12: 16 Chunks (256m)
    (true, 20), // Index 13: 20 Chunks (320m)
    (true, 24), // Index 14: 24 Chunks (384m)
    (true, 32), // Index 15: 32 Chunks (512m)
];

#[derive(Resource, Default)]
pub struct FpsLimiter {
    pub last_frame_instant: Option<std::time::Instant>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SeedInputState {
    pub seed_text: String,
    pub is_editing: bool,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuButtonAction {
    Play,
    ContinueGame,
    NewGame,
    ResumeGame,
    OpenSettings,
    BackFromSettings,
    BackToMain,
    QuitGame,
    ToggleEditSeed,
    RandomizeSeed,
    ToggleVsync,
    ToggleFullscreen,
    ToggleShadows,
    ToggleDistanceFog,
    ToggleDebugHud,
    CycleFpsCap,
    StepFpsCapLeft,
    StepFpsCapRight,
    SlideFpsCap,
    CycleViewDistance,
    StepViewDistanceLeft,
    StepViewDistanceRight,
    SlideViewDistance,
    CycleGreedyMeshing,
    StepGreedyMeshingLeft,
    StepGreedyMeshingRight,
    SlideGreedyMeshing,
    ToggleGreedyMeshing,
    CycleDistanceLod,
    StepDistanceLodLeft,
    StepDistanceLodRight,
    SlideDistanceLod,
    ToggleDistanceLod,
    StartBenchmark,
    BackFromBenchmark,
    OpenAudioSettings,
    BackFromAudioSettings,
    StepMasterVolumeLeft,
    StepMasterVolumeRight,
    SlideMasterVolume,
    StepSfxVolumeLeft,
    StepSfxVolumeRight,
    SlideSfxVolume,
    TestAudioSample,
}

#[derive(Clone, Copy, Debug)]
pub struct OptionDescription {
    pub header: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub impact: &'static str,
}

// Marker components for UI elements and hierarchies
#[derive(Component)]
pub struct SeedInputBox;

#[derive(Component)]
pub struct SeedInputText;

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SettingsMenuRoot;

#[derive(Component)]
pub struct VsyncBtnText;

#[derive(Component)]
pub struct FullscreenBtnText;

#[derive(Component)]
pub struct ShadowsBtnText;

#[derive(Component)]
pub struct DistanceFogBtnText;

#[derive(Component)]
pub struct DebugHudBtnText;

#[derive(Component)]
pub struct FpsCapBtnText;

#[derive(Component)]
pub struct ViewDistanceBtnText;

#[derive(Component)]
pub struct GraphicsGreedyBtnText;

#[derive(Component)]
pub struct GraphicsLodBtnText;

#[derive(Component)]
pub struct OptionTooltipCard;

#[derive(Component)]
pub struct OptionTooltipHeader;

#[derive(Component)]
pub struct OptionTooltipTitle;

#[derive(Component)]
pub struct OptionTooltipDesc;

#[derive(Component)]
pub struct OptionTooltipImpact;

#[derive(Component)]
pub struct SliderTrack;

#[derive(Component)]
pub struct FpsCapTrack;

#[derive(Component)]
pub struct FpsCapFill;

#[derive(Component)]
pub struct FpsCapThumb;

#[derive(Component)]
pub struct ViewDistanceTrack;

#[derive(Component)]
pub struct ViewDistanceFill;

#[derive(Component)]
pub struct ViewDistanceThumb;

#[derive(Component)]
pub struct GraphicsGreedyTrack;

#[derive(Component)]
pub struct GraphicsGreedyFill;

#[derive(Component)]
pub struct GraphicsGreedyThumb;

#[derive(Component)]
pub struct GraphicsLodTrack;

#[derive(Component)]
pub struct GraphicsLodFill;

#[derive(Component)]
pub struct GraphicsLodThumb;

#[derive(Component)]
pub struct BenchmarkResultsRoot;

#[derive(Component)]
pub struct BenchmarkRunningBanner;

#[derive(Component)]
pub struct BenchmarkBannerText;

#[derive(Component)]
pub struct BenchmarkAvgFpsText;

#[derive(Component)]
pub struct BenchmarkOnePercentLowText;

#[derive(Component)]
pub struct BenchmarkFrametimeText;

#[derive(Component)]
pub struct BenchmarkMinMaxFrametimeText;

#[derive(Component)]
pub struct BenchmarkP99Text;

#[derive(Component)]
pub struct BenchmarkRamText;

#[derive(Component)]
pub struct BenchmarkVramText;

#[derive(Component)]
pub struct BenchmarkChunksText;

#[derive(Component)]
pub struct BenchmarkVertsText;

#[derive(Component)]
pub struct BenchmarkSettingsText;

#[derive(Component)]
pub struct BenchmarkVerdictTitleText;

#[derive(Component)]
pub struct BenchmarkVerdictDescText;

// Audio Settings Marker Components
#[derive(Component)]
pub struct AudioSettingsRoot;

#[derive(Component)]
pub struct MasterVolumeBtnText;

#[derive(Component)]
pub struct MasterVolumeTrack;

#[derive(Component)]
pub struct MasterVolumeFill;

#[derive(Component)]
pub struct MasterVolumeThumb;

#[derive(Component)]
pub struct SfxVolumeBtnText;

#[derive(Component)]
pub struct SfxVolumeTrack;

#[derive(Component)]
pub struct SfxVolumeFill;

#[derive(Component)]
pub struct SfxVolumeThumb;
