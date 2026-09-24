use super::types::{MenuButtonAction, OptionDescription};

/// Provides technical descriptions and performance impacts for every configurable setting and button.
#[must_use]
pub const fn get_option_description(action: &MenuButtonAction) -> Option<OptionDescription> {
    match action {
        MenuButtonAction::ToggleVsync => Some(OptionDescription {
            header: "DISPLAY & SYNC",
            title: "Vertical Synchronization (VSync)",
            description: "Synchronizes the game's rendered frame rate with your monitor's physical refresh rate to prevent screen tearing.",
            impact: "- ON: Smooth frame pacing, zero screen tearing.\n- OFF: Lowest input latency, uncapped frame rate.",
        }),
        MenuButtonAction::ToggleFullscreen => Some(OptionDescription {
            header: "DISPLAY MODE",
            title: "Display Mode (Fullscreen / Windowed)",
            description: "Switches between Borderless Fullscreen (native monitor resolution) and Windowed mode (1280x720).",
            impact: "- Fullscreen: Immersive edge-to-edge display.\n- Windowed: Convenient multitasking and window positioning.",
        }),
        MenuButtonAction::ToggleShadows => Some(OptionDescription {
            header: "LIGHTING & SHADOWS",
            title: "Dynamic Directional Shadows",
            description: "Toggles real-time directional sunlight shadow maps across the voxel terrain.",
            impact: "- ON: Realistic depth, tree canopy shadows, and terrain self-shadowing.\n- OFF: Skips shadow passes, yielding higher FPS on integrated graphics.",
        }),
        MenuButtonAction::ToggleDistanceFog => Some(OptionDescription {
            header: "ATMOSPHERE & BLENDING",
            title: "Atmospheric Distance Fog",
            description: "Applies linear atmospheric distance fog that gracefully blends distant terrain into the sky before chunk boundaries.",
            impact: "- ON: Smooth, immersive horizon that hides chunk loading boundaries.\n- OFF: Sharp cutoff edge at the boundary of loaded chunks.",
        }),
        MenuButtonAction::ToggleDebugHud => Some(OptionDescription {
            header: "DIAGNOSTICS & PROFILING",
            title: "Engine Profiler Overlay (F3)",
            description: "Displays real-time FPS, frame pacing, 1% low, memory footprint, GPU VRAM, active chunks, triangle counts, and coordinates.",
            impact: "- Essential for inspecting performance metrics.\n- Can also be toggled anytime in-game using the F3 key.",
        }),
        MenuButtonAction::CycleFpsCap
        | MenuButtonAction::StepFpsCapLeft
        | MenuButtonAction::StepFpsCapRight
        | MenuButtonAction::SlideFpsCap => Some(OptionDescription {
            header: "PERFORMANCE & THERMALS",
            title: "Frame Rate Limiter",
            description: "Limits maximum frames rendered per second (30 to 240 FPS, or Uncapped) using microsecond sleep pacing.",
            impact: "- Capping FPS significantly lowers GPU temperature, power consumption, and fan noise.",
        }),
        MenuButtonAction::CycleViewDistance
        | MenuButtonAction::StepViewDistanceLeft
        | MenuButtonAction::StepViewDistanceRight
        | MenuButtonAction::SlideViewDistance => Some(OptionDescription {
            header: "WORLD GENERATION & RENDER RADIUS",
            title: "Render Distance",
            description: "Sets the horizontal radius of chunks loaded and rendered around the player (4 to 64 chunks = 64m to 1024m).",
            impact: "- 16 Chunks (256m): Recommended balance of horizon view and performance.\n- 32-64 Chunks: Sweeping vistas; higher RAM/VRAM load.",
        }),
        MenuButtonAction::CycleGreedyMeshing
        | MenuButtonAction::StepGreedyMeshingLeft
        | MenuButtonAction::StepGreedyMeshingRight
        | MenuButtonAction::SlideGreedyMeshing
        | MenuButtonAction::ToggleGreedyMeshing => Some(OptionDescription {
            header: "GEOMETRY OPTIMIZATION",
            title: "Greedy Quad Meshing",
            description: "Merges adjacent coplanar block faces into larger single quads beyond the specified chunk threshold (or across all chunks).",
            impact: "- Reduces chunk vertex and triangle counts by up to 75%.\n- OFF: Emits separate 1x1 quads for every exposed block face.",
        }),
        MenuButtonAction::CycleDistanceLod
        | MenuButtonAction::StepDistanceLodLeft
        | MenuButtonAction::StepDistanceLodRight
        | MenuButtonAction::SlideDistanceLod
        | MenuButtonAction::ToggleDistanceLod => Some(OptionDescription {
            header: "DISTANCE LEVEL OF DETAIL (LOD)",
            title: "Distant Sloped Heightfield LOD",
            description: "Replaces distant stepped voxel stairs on mountain slopes with smooth continuous angled surfaces and groups exposed ore veins into stone.",
            impact: "- ON (2-32 Chunks): Cuts distant geometry by up to 90%, stabilizing 60+ FPS.\n- OFF: Preserves 1x1 voxel blocks all the way to the horizon.",
        }),
        MenuButtonAction::StartBenchmark => Some(OptionDescription {
            header: "AUTOMATED BENCHMARK",
            title: "Run Hardware Benchmark",
            description: "Generates a standardized world on seed 'BENCHMARK' and executes a 1,000m high-speed camera flight across diverse terrain to evaluate performance under your active graphics settings.",
            impact: "- Measures average FPS, 1% low, frametimes, RAM, VRAM, and provides optimization advice.",
        }),
        MenuButtonAction::BackFromBenchmark => Some(OptionDescription {
            header: "NAVIGATION",
            title: "Change Graphics Settings",
            description: "Return to the Graphics Settings menu to fine-tune render distance, LOD, or shadows and run another test.",
            impact: "- Quickly iterate and verify frame rate improvements.",
        }),
        MenuButtonAction::BackFromSettings => Some(OptionDescription {
            header: "NAVIGATION",
            title: "Back / Done",
            description: "Save configuration changes and return to the previous menu screen.",
            impact: "- All graphical adjustments apply immediately in real-time.",
        }),
        MenuButtonAction::Play | MenuButtonAction::NewGame => Some(OptionDescription {
            header: "WORLD GENERATION",
            title: "Start New World",
            description: "Generate and enter a brand new voxel world using the specified or randomized seed.",
            impact: "- Spawns on safe dry land with clean procedural terrain.",
        }),
        MenuButtonAction::ContinueGame => Some(OptionDescription {
            header: "SAVE GAME",
            title: "Continue Saved World",
            description: "Resume your journey in the most recently played procedural world with your saved position and inventory.",
            impact: "- Instantly restores terrain modifications and player state from disk.",
        }),
        MenuButtonAction::ResumeGame => Some(OptionDescription {
            header: "GAMEPLAY",
            title: "Resume Game",
            description: "Unpause and return to the active game world.",
            impact: "- Restores mouse capture and camera controls.",
        }),
        MenuButtonAction::OpenSettings => Some(OptionDescription {
            header: "CONFIGURATION",
            title: "Graphics Settings",
            description: "Configure display mode, VSync, frame rate limit, shadows, fog, and view distance.",
            impact: "- Adjust visuals and performance for your hardware.",
        }),
        MenuButtonAction::BackToMain => Some(OptionDescription {
            header: "NAVIGATION",
            title: "Return to Main Menu",
            description: "Save player data and chunk modifications to disk, then return to the main title screen.",
            impact: "- World progress is safely saved.",
        }),
        MenuButtonAction::QuitGame => Some(OptionDescription {
            header: "NAVIGATION",
            title: "Quit Game",
            description: "Close MineRust and return to desktop.",
            impact: "- All world modifications and player inventory are saved.",
        }),
        MenuButtonAction::ToggleEditSeed => Some(OptionDescription {
            header: "WORLD GENERATION",
            title: "Custom World Seed Input",
            description: "Type any alphanumeric string or number to generate a unique procedural world.",
            impact: "- Supports full alphanumeric seed strings.",
        }),
        MenuButtonAction::RandomizeSeed => Some(OptionDescription {
            header: "WORLD GENERATION",
            title: "Randomize World Seed",
            description: "Generates a fresh random 64-bit seed using high-resolution entropy.",
            impact: "- Each click generates a brand new terrain layout.",
        }),
    }
}
