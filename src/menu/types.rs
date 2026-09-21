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

#[derive(Resource)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub fps_cap: Option<u32>, // None = Uncapped, Some(60), Some(120), Some(144)
    pub view_distance: i32,   // 8, 16, 24, 32, 64
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            fps_cap: None,
            view_distance: 16,
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
