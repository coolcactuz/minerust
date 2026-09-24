use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuScreen {
    Main,
    Settings,
    Pause,
    None, // In-game gameplay
}

#[derive(Resource)]
pub struct MenuState {
    pub screen: MenuScreen,
    pub previous_screen: MenuScreen,
    pub world_active: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            screen: MenuScreen::Main, // Start in Main Menu!
            previous_screen: MenuScreen::Main,
            world_active: false,
        }
    }
}

impl MenuState {
    #[inline]
    pub const fn is_open(&self) -> bool {
        !matches!(self.screen, MenuScreen::None)
    }
}

/// Global toggle for the real-time engine profiler HUD (accessible via F3).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProfilerState {
    pub visible: bool,
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub distance_fog: bool,   // Linear atmospheric distance fog
    pub shadows: bool,        // Directional light shadow maps
    pub fps_cap: Option<u32>, // None = Uncapped, Some(30)..Some(240)
    pub view_distance: i32,   // 4 to 64 chunks (64m to 1024m)
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
        }
    }
}

impl GraphicsSettings {
    pub const SAVE_PATH: &'static str = "saves/settings.json";

    #[must_use]
    pub fn load_or_default() -> Self {
        if let Ok(bytes) = std::fs::read(Self::SAVE_PATH) {
            if let Ok(settings) = serde_json::from_slice::<Self>(&bytes) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save_to_disk(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::create_dir_all("saves");
            let _ = std::fs::write(Self::SAVE_PATH, json);
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

pub const VIEW_DISTANCE_STEPS: &[i32] = &[
    4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 56, 64,
];

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
