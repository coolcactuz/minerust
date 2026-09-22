use super::descriptions::get_option_description;
use super::seed::keycode_to_char;
use super::types::{DevSettings, MenuButtonAction, SeedInputState};
use bevy::input::keyboard::KeyCode;

#[test]
fn test_dev_settings_production_defaults() {
    let dev = DevSettings::default();

    // Production mode must be default
    assert!(
        !dev.dev_mode,
        "dev_mode must default to false for public release"
    );
    assert!(
        !dev.show_debug_hud,
        "debug HUD must default to false for public release"
    );

    // All performance optimizations must be locked ON by default
    assert!(dev.backface_culling, "backface culling must be enabled");
    assert!(dev.shadows_enabled, "shadows must be enabled");
    assert!(dev.max_y_skip, "max_y skip must be enabled");
    assert!(dev.distance_fog, "distance fog must be enabled");
    assert!(dev.mesh_budget, "mesh budget must be enabled");
    assert!(dev.async_meshing, "async meshing must be enabled");
    assert!(dev.greedy_meshing, "greedy meshing must be enabled");
    assert!(dev.distance_lod, "distance LOD must be enabled");
    assert_eq!(
        dev.lod_threshold, 4,
        "default LOD threshold should be 4 chunks"
    );
    assert_eq!(
        dev.pregen_margin, 2,
        "default lookahead pregen margin should be 2 chunks"
    );
}

#[test]
fn test_dev_settings_custom_dev_mode() {
    let dev = DevSettings {
        dev_mode: true,
        show_debug_hud: true,
        ..Default::default()
    };

    assert!(dev.dev_mode);
    assert!(dev.show_debug_hud);
    assert!(dev.greedy_meshing);
}

#[test]
fn test_seed_input_state_and_keycode_to_char() {
    let default_state = SeedInputState::default();
    assert!(default_state.seed_text.is_empty());
    assert!(!default_state.is_editing);

    // Test character conversion
    assert_eq!(keycode_to_char(KeyCode::KeyA, false), Some('a'));
    assert_eq!(keycode_to_char(KeyCode::KeyA, true), Some('A'));
    assert_eq!(keycode_to_char(KeyCode::Digit7, false), Some('7'));
    assert_eq!(keycode_to_char(KeyCode::Minus, false), Some('-'));
    assert_eq!(keycode_to_char(KeyCode::Minus, true), Some('_'));
}

#[test]
fn test_get_option_description_all_actions() {
    let actions = [
        MenuButtonAction::ToggleVsync,
        MenuButtonAction::ToggleFullscreen,
        MenuButtonAction::CycleFpsCap,
        MenuButtonAction::CycleViewDistance,
        MenuButtonAction::CycleGreedyMeshing,
        MenuButtonAction::CycleDistanceLod,
        MenuButtonAction::ToggleBackfaceCulling,
        MenuButtonAction::ToggleShadows,
        MenuButtonAction::ToggleMaxYSkip,
        MenuButtonAction::ToggleDistanceFog,
        MenuButtonAction::ToggleMeshBudget,
        MenuButtonAction::ToggleAsyncMeshing,
        MenuButtonAction::ToggleGreedyMeshing,
        MenuButtonAction::ToggleDistanceLod,
        MenuButtonAction::CycleLodThreshold,
        MenuButtonAction::CyclePregenMargin,
        MenuButtonAction::ToggleDebugHud,
        MenuButtonAction::BackFromSettings,
        MenuButtonAction::BackFromDevSettings,
        MenuButtonAction::Play,
        MenuButtonAction::ResumeGame,
        MenuButtonAction::OpenSettings,
        MenuButtonAction::OpenDevSettings,
        MenuButtonAction::BackToMain,
        MenuButtonAction::QuitGame,
        MenuButtonAction::ToggleEditSeed,
        MenuButtonAction::RandomizeSeed,
    ];

    for action in actions {
        let desc = get_option_description(&action);
        assert!(desc.is_some(), "Every action must have a valid description");
        let d = desc.unwrap();
        assert!(!d.header.is_empty(), "Header must not be empty");
        assert!(!d.title.is_empty(), "Title must not be empty");
        assert!(!d.description.is_empty(), "Description must not be empty");
        assert!(!d.impact.is_empty(), "Impact must not be empty");
    }
}

#[test]
fn test_graphics_settings_lod_defaults() {
    let gs = super::types::GraphicsSettings::default();
    assert!(
        gs.greedy_meshing,
        "greedy meshing should default to true in graphics settings"
    );
    assert_eq!(
        gs.greedy_threshold, 2,
        "default greedy threshold should be 2 chunks (32m)"
    );
    assert!(
        gs.distance_lod,
        "distance LOD should default to true in graphics settings"
    );
    assert_eq!(
        gs.lod_threshold, 8,
        "default distant lod threshold should be 8 chunks (128m)"
    );
}
