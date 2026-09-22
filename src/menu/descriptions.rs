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
        MenuButtonAction::CycleFpsCap => Some(OptionDescription {
            header: "PERFORMANCE & THERMALS",
            title: "Frame Rate Limiter",
            description: "Limits maximum frames rendered per second (Uncapped, 60, 120, 144 FPS) using microsecond sleep pacing.",
            impact: "- Capping FPS significantly lowers GPU temperature, power consumption, and fan noise.",
        }),
        MenuButtonAction::CycleViewDistance => Some(OptionDescription {
            header: "WORLD GENERATION & RENDER RADIUS",
            title: "Render Distance",
            description: "Sets the horizontal radius of chunks loaded and rendered around the player (8 to 64 chunks = 128m to 1024m).",
            impact: "- 16 Chunks (256m): Recommended balance of horizon view and performance.\n- 32-64 Chunks: Sweeping vistas; higher RAM/VRAM load.",
        }),
        MenuButtonAction::ToggleBackfaceCulling => Some(OptionDescription {
            header: "GPU PIPELINE BENCHMARK",
            title: "Backface Culling",
            description: "Discards triangles facing away from the camera in the GPU rasterizer. Solid voxel blocks never expose interior faces.",
            impact: "- ON: Cuts rasterizer fragment load and fill-rate by ~50%.\n- OFF: Forces GPU to rasterize front and back faces of every quad.",
        }),
        MenuButtonAction::ToggleShadows => Some(OptionDescription {
            header: "LIGHTING & SHADOWS",
            title: "Dynamic Cascaded Shadows",
            description: "Toggles real-time directional sunlight shadow cascades spanning up to 120 meters from the camera.",
            impact: "- ON: Realistic depth, tree canopy shadows, and terrain self-shadowing.\n- OFF: Skips shadow passes, yielding a large FPS boost on iGPUs.",
        }),
        MenuButtonAction::ToggleMaxYSkip => Some(OptionDescription {
            header: "MESHING BENCHMARK",
            title: "Mesher max_y Air Skipping",
            description: "Tracks the highest solid block per chunk during generation, allowing the mesher to skip empty sky layers up to Y=384.",
            impact: "- ON: ~2x faster chunk meshing, preventing CPU stutters.\n- OFF: Scans all 384 vertical Y layers even if 250 are empty sky.",
        }),
        MenuButtonAction::ToggleDistanceFog => Some(OptionDescription {
            header: "ATMOSPHERE & BLENDING",
            title: "Distance Fog",
            description: "Applies linear atmospheric distance fog that gracefully blends distant terrain into the sky before chunk boundaries.",
            impact: "- ON: Smooth, immersive horizon that hides chunk loading boundaries.\n- OFF: Sharp cutoff edge at the boundary of loaded chunks.",
        }),
        MenuButtonAction::ToggleMeshBudget => Some(OptionDescription {
            header: "FRAME PACING BENCHMARK",
            title: "Frame Mesh Upload Budget",
            description: "Limits GPU buffer uploads of newly meshed chunks to a maximum of 6 chunks per frame.",
            impact: "- ON: Smooth, consistent frame times when flying rapidly.\n- OFF: Uploads all meshes simultaneously, causing micro-stutters.",
        }),
        MenuButtonAction::ToggleAsyncMeshing => Some(OptionDescription {
            header: "MULTITHREADING BENCHMARK",
            title: "Async Multi-Threaded Meshing",
            description: "Dispatches chunk greedy meshing computations to background worker threads across all available CPU cores.",
            impact: "- ON: Zero main-thread lag (0ms) during terrain meshing.\n- OFF: Synchronous meshing on the render thread, causing frame drops.",
        }),
        MenuButtonAction::ToggleGreedyMeshing => Some(OptionDescription {
            header: "GEOMETRY OPTIMIZATION BENCHMARK",
            title: "Greedy Meshing Algorithm",
            description: "Iteratively merges adjacent coplanar block faces sharing the same voxel type into large single rectangular quads.",
            impact: "- ON: Reduces chunk vertex and triangle counts by ~75%.\n- OFF: Emits separate 1x1 quads for every exposed block face.",
        }),
        MenuButtonAction::CycleDistanceLod | MenuButtonAction::ToggleDistanceLod => {
            Some(OptionDescription {
                header: "DISTANCE LEVEL OF DETAIL (LOD)",
                title: "Sloped Heightfield LOD",
                description: "Replaces distant stepped voxel stairs on mountain slopes with smooth continuous angled surfaces and groups exposed ore veins into stone.",
                impact: "- ON: Cuts distant geometry by up to 90%, stabilizing 60+ FPS in mountainous biomes.\n- OFF: Renders every distant block as a 1x1 voxel cube.",
            })
        }
        MenuButtonAction::CycleLodThreshold => Some(OptionDescription {
            header: "LOD DISTANCE TUNING",
            title: "LOD Distance Threshold",
            description: "Distance in chunks (2, 4, 6, 8 chunks = 32m to 128m) at which chunk geometry transitions to simplified Level 2 LOD.",
            impact: "- Shorter distance = higher frame rates at the cost of closer visual simplification.\n- Longer distance = full detail preserved further out.",
        }),
        MenuButtonAction::CyclePregenMargin => Some(OptionDescription {
            header: "MEMORY & STREAMING BENCHMARK",
            title: "Lookahead Pregen Buffer",
            description: "Pre-calculates chunk voxel data in RAM just outside the camera's visual view distance (0 to 4 chunks = 0m to 64m margin).",
            impact: "- Eliminates pop-in stutter when walking forward.\n- 0 Chunks: Disabled (benchmark raw generation latency).\n- 2-4 Chunks: Seamless walking buffer.",
        }),
        MenuButtonAction::ToggleDebugHud => Some(OptionDescription {
            header: "DIAGNOSTICS",
            title: "Debug Diagnostics Overlay (F3)",
            description: "Displays in-game real-time FPS, frame timing, player coordinates, active chunk count, triangle counts, and biome data.",
            impact: "- Essential for profiling performance impacts while playing.\n- Can also be toggled anytime in-game with the F3 key.",
        }),
        MenuButtonAction::BackFromSettings | MenuButtonAction::BackFromDevSettings => {
            Some(OptionDescription {
                header: "NAVIGATION",
                title: "Back / Done",
                description: "Save configuration changes and return to the previous menu screen.",
                impact: "- All graphical and benchmark adjustments apply immediately in real-time.",
            })
        }
        MenuButtonAction::Play => Some(OptionDescription {
            header: "GAMEPLAY",
            title: "Play Game",
            description: "Generate or load the voxel world and enter gameplay.",
            impact: "- Uses the current world seed and graphics settings.",
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
            description: "Configure display mode, VSync, frame rate limit, and view distance.",
            impact: "- Adjust visuals and performance for your hardware.",
        }),
        MenuButtonAction::OpenDevSettings => Some(OptionDescription {
            header: "BENCHMARK TOOLS",
            title: "Dev & Benchmark Settings",
            description: "Toggle internal engine optimizations to measure performance impacts.",
            impact: "- Available only in Developer mode.",
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
