use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuScreen {
    Main,
    Settings,
    DevSettings,
    Pause,
    None, // In-game gameplay
}

#[derive(Resource)]
pub struct MenuState {
    pub screen: MenuScreen,
    pub previous_screen: MenuScreen,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            screen: MenuScreen::Main, // Start in Main Menu!
            previous_screen: MenuScreen::Main,
        }
    }
}

impl MenuState {
    #[inline]
    pub fn is_open(&self) -> bool {
        self.screen != MenuScreen::None
    }
}

#[derive(Resource, Clone, Debug)]
pub struct DevSettings {
    pub dev_mode: bool,
    pub profile_mode: bool,
    pub backface_culling: bool,
    pub shadows_enabled: bool,
    pub max_y_skip: bool,
    pub distance_fog: bool,
    pub mesh_budget: bool,
    pub async_meshing: bool,
    pub greedy_meshing: bool,
    pub distance_lod: bool,
    pub lod_threshold: i32,
    pub show_debug_hud: bool,
    pub pregen_margin: i32,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            dev_mode: false,
            profile_mode: false,
            backface_culling: true,
            shadows_enabled: true,
            max_y_skip: true,
            distance_fog: true,
            mesh_budget: true,
            async_meshing: true,
            greedy_meshing: true,
            distance_lod: true,
            lod_threshold: 4,
            show_debug_hud: false,
            pregen_margin: 2,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub fps_cap: Option<u32>, // None = Uncapped, Some(60), Some(120), Some(144)
    pub view_distance: i32,   // 8, 16, 24, 32, 64
    pub greedy_meshing: bool, // Greedy coplanar quad merging beyond greedy threshold
    pub greedy_threshold: i32, // Distance threshold in chunks: 2 (32m), 3 (48m), 4 (64m), 0 (all)
    pub distance_lod: bool,   // Distant Sloped Heightfield LOD
    pub lod_threshold: i32,   // 6, 8, 10, 12 chunks
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            fps_cap: None,
            view_distance: 16,
            greedy_meshing: true,
            greedy_threshold: 2,
            distance_lod: true,
            lod_threshold: 8,
        }
    }
}

pub const GREEDY_MESHING_STEPS: &[(bool, i32)] = &[
    (false, 2), // Index 0: OFF
    (true, 0),  // Index 1: All Chunks (0m)
    (true, 1),  // Index 2: 1 Chunk (16m)
    (true, 2),  // Index 3: 2 Chunks (32m) - Default
    (true, 3),  // Index 4: 3 Chunks (48m)
    (true, 4),  // Index 5: 4 Chunks (64m)
    (true, 5),  // Index 6: 5 Chunks (80m)
    (true, 6),  // Index 7: 6 Chunks (96m)
    (true, 7),  // Index 8: 7 Chunks (112m)
    (true, 8),  // Index 9: 8 Chunks (128m)
    (true, 10), // Index 10: 10 Chunks (160m)
    (true, 12), // Index 11: 12 Chunks (192m)
    (true, 14), // Index 12: 14 Chunks (224m)
    (true, 16), // Index 13: 16 Chunks (256m)
    (true, 20), // Index 14: 20 Chunks (320m)
    (true, 24), // Index 15: 24 Chunks (384m)
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

impl GraphicsSettings {
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

    #[must_use]
    pub fn greedy_label(&self) -> String {
        if !self.greedy_meshing {
            "Greedy Meshing: OFF (1x1 Voxels)".to_string()
        } else if self.greedy_threshold == 0 {
            "Greedy Distance: All Chunks (0m)".to_string()
        } else {
            let unit = if self.greedy_threshold == 1 { "Chunk" } else { "Chunks" };
            format!(
                "Greedy Distance: > {} {} ({}m)",
                self.greedy_threshold,
                unit,
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

    #[must_use]
    pub fn lod_label(&self) -> String {
        if !self.distance_lod {
            "Distant Sloped LOD: OFF (Blocky Only)".to_string()
        } else {
            let unit = if self.lod_threshold == 1 { "Chunk" } else { "Chunks" };
            format!(
                "Distant Sloped LOD: > {} {} ({}m)",
                self.lod_threshold,
                unit,
                self.lod_threshold * 16
            )
        }
    }
}

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
    ResumeGame,
    OpenSettings,
    OpenDevSettings,
    BackFromSettings,
    BackFromDevSettings,
    BackToMain,
    QuitGame,
    ToggleEditSeed,
    RandomizeSeed,
    ToggleVsync,
    ToggleFullscreen,
    CycleFpsCap,
    CycleViewDistance,
    CycleGreedyMeshing,
    StepGreedyMeshingLeft,
    StepGreedyMeshingRight,
    SlideGreedyMeshing,
    CycleDistanceLod,
    StepDistanceLodLeft,
    StepDistanceLodRight,
    SlideDistanceLod,
    ToggleBackfaceCulling,
    ToggleShadows,
    ToggleMaxYSkip,
    ToggleDistanceFog,
    ToggleMeshBudget,
    ToggleAsyncMeshing,
    ToggleGreedyMeshing,
    ToggleDistanceLod,
    CycleLodThreshold,
    ToggleDebugHud,
    CyclePregenMargin,
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
pub struct DevSettingsMenuRoot;

#[derive(Component)]
pub struct VsyncBtnText;

#[derive(Component)]
pub struct FullscreenBtnText;

#[derive(Component)]
pub struct FpsCapBtnText;

#[derive(Component)]
pub struct ViewDistanceBtnText;

#[derive(Component)]
pub struct BackfaceCullingBtnText;

#[derive(Component)]
pub struct ShadowsBtnText;

#[derive(Component)]
pub struct MaxYSkipBtnText;

#[derive(Component)]
pub struct DistanceFogBtnText;

#[derive(Component)]
pub struct MeshBudgetBtnText;

#[derive(Component)]
pub struct AsyncMeshingBtnText;

#[derive(Component)]
pub struct GraphicsGreedyBtnText;

#[derive(Component)]
pub struct GraphicsLodBtnText;

#[derive(Component)]
pub struct GreedyMeshingBtnText;

#[derive(Component)]
pub struct DistanceLodBtnText;

#[derive(Component)]
pub struct LodThresholdBtnText;

#[derive(Component)]
pub struct DebugHudBtnText;

#[derive(Component)]
pub struct PregenMarginBtnText;

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
pub struct GraphicsGreedyTrack;

#[derive(Component)]
pub struct GraphicsLodTrack;

#[derive(Component)]
pub struct GraphicsGreedyFill;

#[derive(Component)]
pub struct GraphicsGreedyThumb;

#[derive(Component)]
pub struct GraphicsLodFill;

#[derive(Component)]
pub struct GraphicsLodThumb;

