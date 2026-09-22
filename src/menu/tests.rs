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
        MenuButtonAction::StepFpsCapLeft,
        MenuButtonAction::StepFpsCapRight,
        MenuButtonAction::SlideFpsCap,
        MenuButtonAction::CycleViewDistance,
        MenuButtonAction::StepViewDistanceLeft,
        MenuButtonAction::StepViewDistanceRight,
        MenuButtonAction::SlideViewDistance,
        MenuButtonAction::CycleGreedyMeshing,
        MenuButtonAction::StepGreedyMeshingLeft,
        MenuButtonAction::StepGreedyMeshingRight,
        MenuButtonAction::SlideGreedyMeshing,
        MenuButtonAction::CycleDistanceLod,
        MenuButtonAction::StepDistanceLodLeft,
        MenuButtonAction::StepDistanceLodRight,
        MenuButtonAction::SlideDistanceLod,
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

#[test]
fn test_greedy_meshing_slider_steps_and_ratios() {
    let mut gs = super::types::GraphicsSettings::default();

    // Default is 2 chunks -> index 3
    assert_eq!(gs.greedy_step_index(), 3);
    assert!((gs.greedy_ratio() - 3.0 / 15.0).abs() < 1e-4);
    assert_eq!(gs.greedy_label(), "Greedy Distance: > 2 Chunks (32m)");

    // Step down to 1 chunk, then 0 chunks (all), then OFF
    gs.step_greedy(-1);
    assert_eq!(gs.greedy_step_index(), 2);
    assert_eq!(gs.greedy_threshold, 1);
    assert_eq!(gs.greedy_label(), "Greedy Distance: > 1 Chunk (16m)");

    gs.step_greedy(-1);
    assert_eq!(gs.greedy_step_index(), 1);
    assert_eq!(gs.greedy_threshold, 0);
    assert_eq!(gs.greedy_label(), "Greedy Distance: All Chunks (0m)");

    gs.step_greedy(-1);
    assert_eq!(gs.greedy_step_index(), 0);
    assert!(!gs.greedy_meshing);
    assert_eq!(gs.greedy_label(), "Greedy Meshing: OFF (1x1 Voxels)");

    // Clamping at index 0
    gs.step_greedy(-1);
    assert_eq!(gs.greedy_step_index(), 0);

    // Set from ratio: 1.0 (far right) -> 24 chunks
    gs.set_greedy_from_ratio(1.0);
    assert_eq!(gs.greedy_step_index(), 15);
    assert!(gs.greedy_meshing);
    assert_eq!(gs.greedy_threshold, 24);
    assert_eq!(gs.greedy_label(), "Greedy Distance: > 24 Chunks (384m)");
}

#[test]
fn test_distance_lod_slider_steps_and_ratios() {
    let mut gs = super::types::GraphicsSettings::default();

    // Default is 8 chunks -> index 7
    assert_eq!(gs.lod_step_index(), 7);
    assert!((gs.lod_ratio() - 7.0 / 15.0).abs() < 1e-4);
    assert_eq!(gs.lod_label(), "Distant Sloped LOD: > 8 Chunks (128m)");

    // Step down to 7, 6, ..., OFF
    gs.step_lod(-1);
    assert_eq!(gs.lod_step_index(), 6);
    assert_eq!(gs.lod_threshold, 7);

    // Set from ratio: 0.0 (OFF)
    gs.set_lod_from_ratio(0.0);
    assert_eq!(gs.lod_step_index(), 0);
    assert!(!gs.distance_lod);
    assert_eq!(gs.lod_label(), "Distant Sloped LOD: OFF (Blocky Only)");

    // Set from ratio: 1.0 (far right) -> 32 chunks
    gs.set_lod_from_ratio(1.0);
    assert_eq!(gs.lod_step_index(), 15);
    assert!(gs.distance_lod);
    assert_eq!(gs.lod_threshold, 32);
    assert_eq!(gs.lod_label(), "Distant Sloped LOD: > 32 Chunks (512m)");
}

#[test]
fn test_fps_cap_slider_steps_and_ratios() {
    let mut gs = super::types::GraphicsSettings::default();

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
    let mut gs = super::types::GraphicsSettings::default();

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
fn test_distance_fog_default() {
    let gs = super::types::GraphicsSettings::default();
    assert!(gs.distance_fog, "distance fog should default to true in graphics settings");
}

#[test]
fn test_menu_systems_schedule_no_conflicts() {
    use bevy::prelude::*;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<StandardMaterial>()
        .init_asset::<Mesh>()
        .insert_resource(crate::world::WorldGrid::new(crate::world::WorldSeed::default()))
        .add_plugins(crate::menu::MenuPlugin);
    app.finish();
    app.cleanup();
    app.update();
}

#[test]
fn test_menu_systems_schedule_dev_mode() {
    use bevy::prelude::*;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .init_asset::<StandardMaterial>()
        .init_asset::<Mesh>()
        .insert_resource(crate::world::WorldGrid::new(crate::world::WorldSeed::default()))
        .insert_resource(crate::menu::DevSettings {
            dev_mode: true,
            ..default()
        })
        .add_plugins(crate::menu::MenuPlugin);
    app.finish();
    app.cleanup();
    app.update();
}
