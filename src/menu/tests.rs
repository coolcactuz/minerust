use super::descriptions::get_option_description;
use super::seed::keycode_to_char;
use super::types::{GraphicsSettings, MenuButtonAction, ProfilerState, SeedInputState};
use bevy::input::keyboard::KeyCode;

#[test]
fn test_graphics_settings_defaults() {
    let gs = GraphicsSettings::default();

    assert!(gs.vsync, "vsync must default to true");
    assert!(!gs.fullscreen, "fullscreen must default to false");
    assert!(gs.distance_fog, "distance fog must default to true");
    assert!(gs.shadows, "shadows must default to true");
    assert_eq!(gs.fps_cap, None, "fps cap must default to uncapped");
    assert_eq!(gs.view_distance, 16, "view distance must default to 16 chunks");
}

#[test]
fn test_profiler_state_defaults() {
    let profiler = ProfilerState::default();
    assert!(
        !profiler.visible,
        "profiler HUD must default to hidden for general players"
    );
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
        MenuButtonAction::ToggleShadows,
        MenuButtonAction::ToggleDistanceFog,
        MenuButtonAction::ToggleDebugHud,
        MenuButtonAction::CycleFpsCap,
        MenuButtonAction::StepFpsCapLeft,
        MenuButtonAction::StepFpsCapRight,
        MenuButtonAction::SlideFpsCap,
        MenuButtonAction::CycleViewDistance,
        MenuButtonAction::StepViewDistanceLeft,
        MenuButtonAction::StepViewDistanceRight,
        MenuButtonAction::SlideViewDistance,
        MenuButtonAction::BackFromSettings,
        MenuButtonAction::Play,
        MenuButtonAction::ContinueGame,
        MenuButtonAction::NewGame,
        MenuButtonAction::ResumeGame,
        MenuButtonAction::OpenSettings,
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
fn test_fps_cap_slider_steps_and_ratios() {
    let mut gs = GraphicsSettings::default();

    // Default is None (Uncapped) -> index 8 (1.0 ratio)
    assert_eq!(gs.fps_cap_step_index(), 8);
    assert!((gs.fps_cap_ratio() - 1.0).abs() < 1e-4);
    assert_eq!(gs.fps_cap_label(), "FPS Limit: Uncapped (Max FPS)");

    // Step down to 240 FPS, then 165 FPS, ...
    gs.step_fps_cap(-1);
    assert_eq!(gs.fps_cap_step_index(), 7);
    assert_eq!(gs.fps_cap, Some(240));
    assert_eq!(gs.fps_cap_label(), "FPS Limit: 240 FPS");

    // Set from ratio: 0.0 -> 30 FPS
    gs.set_fps_cap_from_ratio(0.0);
    assert_eq!(gs.fps_cap_step_index(), 0);
    assert_eq!(gs.fps_cap, Some(30));
    assert_eq!(gs.fps_cap_label(), "FPS Limit: 30 FPS");

    // Set from ratio: 0.125 -> 60 FPS
    gs.set_fps_cap_from_ratio(0.125);
    assert_eq!(gs.fps_cap_step_index(), 1);
    assert_eq!(gs.fps_cap, Some(60));
    assert_eq!(gs.fps_cap_label(), "FPS Limit: 60 FPS");
}

#[test]
fn test_view_distance_slider_steps_and_ratios() {
    let mut gs = GraphicsSettings::default();

    // Default is 16 chunks -> index 6 (ratio = 6/15)
    assert_eq!(gs.view_distance_step_index(), 6);
    assert!((gs.view_distance_ratio() - 6.0 / 15.0).abs() < 1e-4);
    assert_eq!(gs.view_distance_label(), "Render Distance: 16 Chunks (256m)");

    // Step down to 14, 12, ...
    gs.step_view_distance(-1);
    assert_eq!(gs.view_distance_step_index(), 5);
    assert_eq!(gs.view_distance, 14);
    assert_eq!(gs.view_distance_label(), "Render Distance: 14 Chunks (224m)");

    // Set from ratio: 0.0 -> 4 chunks (64m)
    gs.set_view_distance_from_ratio(0.0);
    assert_eq!(gs.view_distance_step_index(), 0);
    assert_eq!(gs.view_distance, 4);
    assert_eq!(gs.view_distance_label(), "Render Distance: 4 Chunks (64m)");

    // Set from ratio: 1.0 -> 64 chunks (1024m)
    gs.set_view_distance_from_ratio(1.0);
    assert_eq!(gs.view_distance_step_index(), 15);
    assert_eq!(gs.view_distance, 64);
    assert_eq!(gs.view_distance_label(), "Render Distance: 64 Chunks (1024m)");
}

#[test]
fn test_graphics_settings_serialization_roundtrip() {
    let original = GraphicsSettings {
        vsync: false,
        fullscreen: true,
        distance_fog: false,
        shadows: false,
        fps_cap: Some(144),
        view_distance: 32,
    };

    let json = serde_json::to_string(&original).expect("Serialization failed");
    let deserialized: GraphicsSettings =
        serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(original, deserialized);
}

#[test]
fn test_menu_systems_schedule_no_conflicts() {
    use bevy::prelude::*;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<crate::voxel_material::VoxelBlockMaterial>()
        .init_asset::<Mesh>()
        .insert_resource(crate::world::WorldGrid::new(crate::world::WorldSeed::default()))
        .add_plugins(crate::menu::MenuPlugin);
    app.finish();
    app.cleanup();
    app.update();
}
