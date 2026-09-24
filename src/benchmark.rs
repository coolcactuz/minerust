use std::fs::File;
use std::io::Write;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::camera::FpsCamera;
use crate::menu::GraphicsSettings;
use crate::physics::PlayerPhysics;
use crate::profile::read_process_memory;
use crate::world::WorldGrid;

/// Standard predefined flight benchmark profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchmarkPreset {
    Flight5km,
}

impl BenchmarkPreset {
    pub const ALL: [Self; 1] = [Self::Flight5km];

    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "flight" | "standard" | "production" | "5km" | "1km" | "bench" | "baseline" | "culling" | "greedy" | "sloped_lod" => {
                Some(Self::Flight5km)
            }
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        "flight_5km"
    }

    pub fn name(self) -> &'static str {
        "5km Flight Benchmark"
    }

    pub fn into_scenario(self, view_distance: i32) -> BenchmarkScenario {
        BenchmarkScenario::standard(view_distance)
    }
}

/// Fully-encapsulated configuration for an individual benchmark run.
///
/// Contains all flight trajectory and render distance parameters needed
/// to execute a deterministic and reproducible benchmark against the built-in engine.
#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkScenario {
    /// Machine-readable identifier
    pub id: &'static str,
    /// Human-readable display name for reports and logs
    pub name: &'static str,
    /// Render distance in chunks (radius)
    pub view_distance: i32,
    /// Target flight distance in meters (default 1000m / 1km)
    pub flight_distance: f32,
    /// Flight speed in m/s (default 50m/s = 180km/h)
    pub flight_speed: f32,
    /// Flight altitude in world Y coordinates (default 92.0)
    pub flight_altitude: f32,
    /// Minimum warmup duration in seconds before recording
    pub warmup_duration_secs: f32,
    /// Enable atmospheric distance fog
    pub distance_fog: bool,
    /// Output file path for JSON metrics report
    pub output_path: Option<String>,
}

impl BenchmarkScenario {
    pub const DEFAULT_FLIGHT_DISTANCE: f32 = 5000.0;
    pub const DEFAULT_FLIGHT_SPEED: f32 = 50.0;
    pub const DEFAULT_FLIGHT_ALTITUDE: f32 = 92.0;
    pub const DEFAULT_WARMUP_SECS: f32 = 2.5;

    pub fn standard(view_distance: i32) -> Self {
        Self {
            id: "standard",
            name: "Standard 5km Flight Benchmark",
            view_distance,
            flight_distance: Self::DEFAULT_FLIGHT_DISTANCE,
            flight_speed: Self::DEFAULT_FLIGHT_SPEED,
            flight_altitude: Self::DEFAULT_FLIGHT_ALTITUDE,
            warmup_duration_secs: Self::DEFAULT_WARMUP_SECS,
            distance_fog: false,
            output_path: Some("benchmark_results/flight_benchmark.json".to_string()),
        }
    }

    pub fn production(view_distance: i32) -> Self {
        Self::standard(view_distance)
    }

    pub fn from_preset(preset: BenchmarkPreset, view_distance: i32) -> Self {
        preset.into_scenario(view_distance)
    }

    pub fn from_preset_name(name: &str, view_distance: i32) -> Option<Self> {
        BenchmarkPreset::from_str_name(name).map(|p| p.into_scenario(view_distance))
    }

    /// Directly configures Bevy's GraphicsSettings according to this scenario.
    pub fn apply(&self, graphics: &mut GraphicsSettings) {
        graphics.vsync = false;
        graphics.fps_cap = None;
        graphics.distance_fog = self.distance_fog;
        graphics.view_distance = self.view_distance;
    }
}

/// A collection of benchmark scenarios executed deterministically against a fixed seed.
#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkSuite {
    pub seed: u64,
    pub scenarios: Vec<BenchmarkScenario>,
}

impl BenchmarkSuite {
    pub const DEFAULT_SEED: u64 = 133742;

    pub fn standard(view_distance: i32) -> Self {
        Self {
            seed: Self::DEFAULT_SEED,
            scenarios: vec![BenchmarkScenario::standard(view_distance)],
        }
    }
}

/// Runtime resource controlling the automated benchmark runner.
#[derive(Resource, Clone, Debug)]
pub struct BenchmarkConfig {
    pub enabled: bool,
    pub is_cli: bool,
    pub seed: u64,
    pub scenario: BenchmarkScenario,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            is_cli: false,
            seed: BenchmarkSuite::DEFAULT_SEED,
            scenario: BenchmarkScenario::production(16),
        }
    }
}

/// Comprehensive summary of benchmark metrics presented to the user on completion.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct BenchmarkSummary {
    pub has_results: bool,
    pub total_duration_secs: f32,
    pub total_frames: usize,
    pub avg_fps: f32,
    pub one_percent_low_fps: f32,
    pub p99_frametime_ms: f32,
    pub avg_frametime_ms: f32,
    pub min_frametime_ms: f32,
    pub max_frametime_ms: f32,
    pub peak_rss_mb: f32,
    pub peak_vram_mb: f32,
    pub total_chunks_meshed: usize,
    pub peak_chunks_active: usize,
    pub peak_vertices: usize,
    pub distance_traveled: f32,
    pub tested_view_distance: i32,
    pub tested_greedy: String,
    pub tested_lod: String,
    pub tested_shadows: bool,
    pub tested_fog: bool,
    pub verdict_title: String,
    pub verdict_desc: String,
}

impl BenchmarkSummary {
    #[must_use]
    pub fn generate_verdict(avg_fps: f32, one_percent_low_fps: f32, _p99_ms: f32) -> (String, String) {
        if avg_fps >= 100.0 && one_percent_low_fps >= 60.0 {
            (
                "PERFECT: Ultra-Smooth High Refresh Rate".to_string(),
                "Your hardware delivers 100+ FPS with rock-solid frame pacing. You can comfortably increase Render Distance or maximize visual quality!".to_string(),
            )
        } else if avg_fps >= 60.0 && one_percent_low_fps >= 45.0 {
            (
                "GREAT: Smooth 60+ FPS Gameplay".to_string(),
                "Your configuration delivers a consistent 60+ FPS experience with minimal frame drops under fast terrain streaming.".to_string(),
            )
        } else if avg_fps >= 45.0 {
            (
                "PLAYABLE: Minor Streaming Stutter".to_string(),
                "Framerate is generally playable but experiences dips during high-speed terrain loading. Try setting Distant Sloped LOD to > 4-8 chunks to reduce distant geometry.".to_string(),
            )
        } else {
            (
                "SUB-OPTIMAL: Heavy GPU / CPU Load".to_string(),
                "Significant frame drops detected. Recommended tweaks: enable Distant Sloped LOD (> 4 chunks), disable Dynamic Shadows, or reduce Render Distance by 2-4 chunks.".to_string(),
            )
        }
    }
}

/// Phases of the scientific automated benchmark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BenchmarkPhase {
    /// Phase 1: Camera stationary at spawn while background worker threads generate and mesh all initial chunks.
    #[default]
    InitializingWorld,
    /// Phase 2: World is 100% generated and meshed; camera remains stationary for 1.5s to record pure static GPU rendering.
    StationarySettle,
    /// Phase 3: Camera flies 1,000 meters forward along +Z at 50 m/s across terrain, recording dynamic streaming metrics.
    FlightRecording,
    /// Phase 4: Benchmark completed; report printed and JSON exported.
    Completed,
}

/// Dynamic runtime state tracking benchmark progress and statistical telemetry.
#[derive(Resource, Default)]
pub struct BenchmarkState {
    pub phase: BenchmarkPhase,
    pub elapsed: f32,
    pub stationary_timer: f32,
    pub last_status_print: f32,
    pub start_pos: Option<Vec3>,
    pub distance_traveled: f32,

    // Phase 2: Static baseline metrics (100% loaded world, zero streaming/generation CPU load)
    pub static_frame_times_ms: Vec<f32>,
    pub static_chunks: usize,
    pub static_vertices: usize,
    pub static_fps: f32,
    pub static_frametime_ms: f32,
    pub static_vram_mb: f32,

    // Phase 3: Dynamic 1km flight streaming metrics
    pub frame_times_ms: Vec<f32>,
    pub vertex_samples: Vec<usize>,
    pub chunk_samples: Vec<usize>,
    pub peak_rss_mb: f32,
    pub peak_vram_mb: f32,
    pub completed: bool,
}

/// System that executes the automated deterministic benchmark trajectory and records frame latencies.
pub fn benchmark_runner_system(
    time: Res<Time>,
    mut config: ResMut<BenchmarkConfig>,
    mut state: ResMut<BenchmarkState>,
    mut summary: ResMut<BenchmarkSummary>,
    mut menu: Option<ResMut<crate::menu::MenuState>>,
    graphics_settings: Option<Res<crate::menu::GraphicsSettings>>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions, With<bevy::window::PrimaryWindow>>,
    mut player_query: Query<(&mut Transform, &mut FpsCamera, &mut PlayerPhysics)>,
    world: Option<Res<WorldGrid>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    if !config.enabled || state.completed {
        return;
    }

    let dt = time.delta_secs();
    state.elapsed += dt;

    // Track peak physical RAM (VmRSS) and GPU VRAM from OS
    let mem = read_process_memory();
    if mem.rss_mb > state.peak_rss_mb {
        state.peak_rss_mb = mem.rss_mb;
    }
    let vram = crate::profile::read_gpu_vram();
    if vram.used_mb > state.peak_vram_mb {
        state.peak_vram_mb = vram.used_mb;
    }

    let Ok((mut transform, mut fps, mut physics)) = player_query.single_mut() else {
        return;
    };

    physics.is_flying = true;
    physics.velocity = Vec3::ZERO;
    fps.yaw = 0.0;
    fps.pitch = -0.06; // Look slightly downwards across terrain
    transform.rotation = Quat::from_rotation_y(fps.yaw) * Quat::from_rotation_x(fps.pitch);
    transform.translation.x = 0.0;
    transform.translation.y = config.scenario.flight_altitude;

    match state.phase {
        BenchmarkPhase::InitializingWorld => {
            // Keep camera stationary at spawn (Z = 0)
            transform.translation.z = 0.0;

            let Some(ref w) = world else {
                return;
            };

            // Print status updates while background threads generate and mesh initial spawn chunks
            let total_needed = (2 * config.scenario.view_distance + 1) * (2 * config.scenario.view_distance + 1);
            if state.elapsed - state.last_status_print >= 0.5 {
                state.last_status_print = state.elapsed;
                println!(
                    "[BENCHMARK] Initializing world around spawn... Meshed: {} / {} chunks | Gen queue: {}, Mesh queue: {}",
                    w.chunk_entities.len(),
                    total_needed,
                    w.generation_queue.len(),
                    w.mesh_queue.len()
                );
            }

            let queues_empty = w.generation_queue.is_empty()
                && w.in_progress_chunks.is_empty()
                && w.mesh_queue.is_empty()
                && w.in_progress_meshes.is_empty();

            // Give at least 0.4s for frame 0 queues to register, then check queues empty and meshes spawned
            if state.elapsed >= 0.4 && queues_empty && !w.chunk_entities.is_empty() {
                state.static_chunks = w.chunk_entities.len();
                state.static_vertices = w.total_vertices;
                let current_vram = crate::profile::read_gpu_vram();
                state.static_vram_mb = current_vram.used_mb;
                state.phase = BenchmarkPhase::StationarySettle;
                state.stationary_timer = 0.0;

                println!("\n============================================================");
                println!("           MINERUST WORLD INITIALIZATION COMPLETE           ");
                println!("============================================================");
                println!("  Spawn Area Meshed    : {} / {} chunks", state.static_chunks, total_needed);
                println!(
                    "  Spawn Area Geometry  : {} vertices (~{} triangles)",
                    state.static_vertices,
                    state.static_vertices / 2
                );
                if state.static_vram_mb > 0.0 {
                    println!("  Spawn Area GPU VRAM  : {:.1} MB", state.static_vram_mb);
                }
                println!("  Settling Baseline    : Measuring 1.5s of stationary render...");
                println!("============================================================\n");
            }
        }
        BenchmarkPhase::StationarySettle => {
            // Keep camera stationary at Z = 0
            transform.translation.z = 0.0;

            state.stationary_timer += dt;
            let frame_ms = dt * 1000.0;
            state.static_frame_times_ms.push(frame_ms);

            // Record stationary baseline for 1.5 seconds
            if state.stationary_timer >= 1.5 {
                let total_frames = state.static_frame_times_ms.len();
                let total_time_ms: f32 = state.static_frame_times_ms.iter().sum();
                state.static_fps = if total_time_ms > 0.0 {
                    (total_frames as f32 * 1000.0) / total_time_ms
                } else {
                    0.0
                };
                state.static_frametime_ms = if total_frames > 0 {
                    total_time_ms / total_frames as f32
                } else {
                    0.0
                };

                println!(
                    "[BENCHMARK] Static Baseline Render: \x1b[1;32m{:.1} FPS\x1b[0m ({:.2} ms frametime)",
                    state.static_fps, state.static_frametime_ms
                );
                println!(
                    "[BENCHMARK] Starting 5km Trajectory Flight at {:.1} m/s (Target: {:.0}m)...\n",
                    config.scenario.flight_speed, config.scenario.flight_distance
                );

                state.phase = BenchmarkPhase::FlightRecording;
                state.start_pos = Some(transform.translation);
                state.distance_traveled = 0.0;
            }
        }
        BenchmarkPhase::FlightRecording => {
            // Advance in camera forward direction at steady flight speed
            let mut forward = *transform.forward();
            forward.y = 0.0;
            let forward = forward.normalize_or_zero();
            transform.translation += forward * config.scenario.flight_speed * dt;

            let start = state.start_pos.get_or_insert(transform.translation);
            let diff = transform.translation - *start;
            state.distance_traveled = Vec2::new(diff.x, diff.z).length();

            let frame_ms = dt * 1000.0;
            state.frame_times_ms.push(frame_ms);

            if let Some(ref w) = world {
                state.vertex_samples.push(w.total_vertices);
                state.chunk_samples.push(w.chunk_entities.len());
            }

            if state.distance_traveled >= config.scenario.flight_distance {
                state.phase = BenchmarkPhase::Completed;
                state.completed = true;
                let computed = compute_benchmark_summary(&state, &config, graphics_settings.as_deref());
                *summary = computed;

                if config.is_cli {
                    print_and_save_benchmark_report(&state, &config);
                    exit_writer.write(AppExit::Success);
                } else {
                    println!("[BENCHMARK] In-game benchmark complete! Transitioning to results dashboard.");
                    config.enabled = false;
                    if let Some(ref mut m) = menu {
                        m.screen = crate::menu::MenuScreen::BenchmarkResults;
                        m.world_active = true;
                    }
                    if let Ok(mut cursor) = cursor_options.single_mut() {
                        cursor.grab_mode = bevy::window::CursorGrabMode::None;
                        cursor.visible = true;
                    }
                }
            }
        }
        BenchmarkPhase::Completed => {}
    }
}

/// Computes structured statistical benchmark results from collected telemetry.
#[must_use]
pub fn compute_benchmark_summary(
    state: &BenchmarkState,
    config: &BenchmarkConfig,
    graphics: Option<&GraphicsSettings>,
) -> BenchmarkSummary {
    let total_frames = state.frame_times_ms.len();
    if total_frames == 0 {
        return BenchmarkSummary::default();
    }

    let total_time_ms: f32 = state.frame_times_ms.iter().sum();
    let total_duration_secs = total_time_ms / 1000.0;
    let avg_fps = total_frames as f32 / total_duration_secs;
    let avg_frametime_ms = total_time_ms / total_frames as f32;

    let min_frametime_ms = state
        .frame_times_ms
        .iter()
        .copied()
        .reduce(f32::min)
        .unwrap_or(0.0);
    let max_frametime_ms = state
        .frame_times_ms
        .iter()
        .copied()
        .reduce(f32::max)
        .unwrap_or(0.0);

    let mut sorted_times = state.frame_times_ms.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p99_idx = ((total_frames as f32 * 0.99) as usize).min(total_frames.saturating_sub(1));
    let p99_frametime_ms = sorted_times[p99_idx];
    let one_percent_low_fps = if p99_frametime_ms > 0.0 {
        1000.0 / p99_frametime_ms
    } else {
        0.0
    };

    let peak_verts = state.vertex_samples.iter().copied().max().unwrap_or(0);
    let peak_chunks = state.chunk_samples.iter().copied().max().unwrap_or(0);

    let (tested_view_distance, tested_greedy, tested_lod, tested_shadows, tested_fog) =
        if let Some(g) = graphics {
            (
                g.view_distance,
                g.greedy_label(),
                g.lod_label(),
                g.shadows,
                g.distance_fog,
            )
        } else {
            (
                config.scenario.view_distance,
                "Greedy: Built-in (>32m)".to_string(),
                "Sloped LOD: Built-in (>128m)".to_string(),
                true,
                config.scenario.distance_fog,
            )
        };

    let (verdict_title, verdict_desc) =
        BenchmarkSummary::generate_verdict(avg_fps, one_percent_low_fps, p99_frametime_ms);

    BenchmarkSummary {
        has_results: true,
        total_duration_secs,
        total_frames,
        avg_fps,
        one_percent_low_fps,
        p99_frametime_ms,
        avg_frametime_ms,
        min_frametime_ms,
        max_frametime_ms,
        peak_rss_mb: state.peak_rss_mb,
        peak_vram_mb: state.peak_vram_mb,
        total_chunks_meshed: state.static_chunks,
        peak_chunks_active: peak_chunks,
        peak_vertices: peak_verts,
        distance_traveled: state.distance_traveled,
        tested_view_distance,
        tested_greedy,
        tested_lod,
        tested_shadows,
        tested_fog,
        verdict_title,
        verdict_desc,
    }
}

/// Analyzes collected frame latency distributions and outputs a comprehensive scientific report.
fn print_and_save_benchmark_report(state: &BenchmarkState, config: &BenchmarkConfig) {
    let total_frames = state.frame_times_ms.len();
    if total_frames == 0 {
        println!("[BENCHMARK] Error: No frames were recorded during benchmark flight.");
        return;
    }

    let total_time_ms: f32 = state.frame_times_ms.iter().sum();
    let total_duration_secs = total_time_ms / 1000.0;
    let avg_fps = total_frames as f32 / total_duration_secs;
    let avg_frametime_ms = total_time_ms / total_frames as f32;

    let min_frametime_ms = state
        .frame_times_ms
        .iter()
        .copied()
        .reduce(f32::min)
        .unwrap_or(0.0);
    let max_frametime_ms = state
        .frame_times_ms
        .iter()
        .copied()
        .reduce(f32::max)
        .unwrap_or(0.0);

    let max_fps = if min_frametime_ms > 0.0 { 1000.0 / min_frametime_ms } else { 0.0 };
    let min_fps = if max_frametime_ms > 0.0 { 1000.0 / max_frametime_ms } else { 0.0 };

    let mut sorted_times = state.frame_times_ms.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p99_idx = ((total_frames as f32 * 0.99) as usize).min(total_frames.saturating_sub(1));
    let p99_ms = sorted_times[p99_idx];
    let one_percent_low_fps = if p99_ms > 0.0 { 1000.0 / p99_ms } else { 0.0 };

    let p999_idx = ((total_frames as f32 * 0.999) as usize).min(total_frames.saturating_sub(1));
    let p999_ms = sorted_times[p999_idx];
    let point_one_percent_low_fps = if p999_ms > 0.0 { 1000.0 / p999_ms } else { 0.0 };

    let variance = state
        .frame_times_ms
        .iter()
        .map(|t| (t - avg_frametime_ms).powi(2))
        .sum::<f32>()
        / total_frames as f32;
    let stdev_ms = variance.sqrt();

    let avg_verts = if !state.vertex_samples.is_empty() {
        state.vertex_samples.iter().sum::<usize>() / state.vertex_samples.len()
    } else {
        0
    };
    let peak_verts = state.vertex_samples.iter().copied().max().unwrap_or(0);
    let peak_chunks = state.chunk_samples.iter().copied().max().unwrap_or(0);

    println!("\n============================================================");
    println!("             MINERUST SCIENTIFIC BENCHMARK REPORT            ");
    println!("============================================================");
    println!("  Preset Profile       : \x1b[1;36m{}\x1b[0m", config.scenario.name);
    println!("  World Seed           : {} (Render Distance: {} chunks)", config.seed, config.scenario.view_distance);
    println!("------------------------------------------------------------");
    println!("  [PHASE 1: STATIC SCENE - PURE GPU RENDER (ZERO STREAMING)]");
    println!("  Static Meshed Chunks : {} chunks", state.static_chunks);
    println!(
        "  Static Geometry      : {} vertices (~{} triangles)",
        state.static_vertices,
        state.static_vertices / 2
    );
    if state.static_vram_mb > 0.0 {
        println!("  Static GPU VRAM      : {:.1} MB", state.static_vram_mb);
    }
    println!(
        "  Static Framerate     : \x1b[1;32m{:.1} FPS\x1b[0m ({:.2} ms frametime)",
        state.static_fps, state.static_frametime_ms
    );
    println!("------------------------------------------------------------");
    println!("  [PHASE 2: DYNAMIC FLIGHT - 5KM TRAJECTORY STREAMING]");
    println!(
        "  Distance Traveled    : \x1b[1;32m{:.1} m ({:.2} km)\x1b[0m in {:.2} s (Speed: {:.1} m/s)",
        state.distance_traveled,
        state.distance_traveled / 1000.0,
        total_duration_secs,
        config.scenario.flight_speed
    );
    println!("  Recorded Frames      : {} frames", total_frames);
    println!("  AVERAGE FRAMERATE    : \x1b[1;32m{:.1} FPS\x1b[0m", avg_fps);
    println!("  1% LOW FRAMERATE     : \x1b[1;33m{:.1} FPS\x1b[0m (p99 latency: {:.2} ms)", one_percent_low_fps, p99_ms);
    println!("  0.1% LOW FRAMERATE   : \x1b[1;31m{:.1} FPS\x1b[0m (p99.9 latency: {:.2} ms)", point_one_percent_low_fps, p999_ms);
    println!("  Average Frametime    : {:.2} ms (Jitter StDev: {:.2} ms)", avg_frametime_ms, stdev_ms);
    println!("  Frametime Min / Max  : {:.2} ms ({:.1} FPS) / {:.2} ms ({:.1} FPS)", min_frametime_ms, max_fps, max_frametime_ms, min_fps);
    println!("  Active GPU Chunks    : Peak {} chunks", peak_chunks);
    println!(
        "  Active Geometry      : Avg {} verts / Peak {} verts (~{} tris)",
        avg_verts,
        peak_verts,
        peak_verts / 2
    );
    println!("  Peak RAM (VmRSS)     : {:.1} MB", state.peak_rss_mb);
    if state.peak_vram_mb > 0.0 {
        println!("  Peak GPU VRAM        : \x1b[1;35m{:.1} MB\x1b[0m", state.peak_vram_mb);
    }
    println!("============================================================\n");

    if let Some(ref path) = config.scenario.output_path {
        let json = format!(
            "{{\n  \"preset\": \"{}\",\n  \"preset_id\": \"{}\",\n  \"seed\": {},\n  \"view_distance\": {},\n  \"static_fps\": {:.2},\n  \"static_frametime_ms\": {:.3},\n  \"static_chunks\": {},\n  \"static_vertices\": {},\n  \"static_triangles\": {},\n  \"static_vram_mb\": {:.2},\n  \"distance_meters\": {:.1},\n  \"flight_duration_secs\": {:.3},\n  \"flight_frames\": {},\n  \"flight_avg_fps\": {:.2},\n  \"flight_one_percent_low_fps\": {:.2},\n  \"flight_point_one_percent_low_fps\": {:.2},\n  \"flight_avg_frametime_ms\": {:.3},\n  \"flight_min_frametime_ms\": {:.3},\n  \"flight_max_frametime_ms\": {:.3},\n  \"flight_frametime_stdev_ms\": {:.3},\n  \"flight_avg_vertices\": {},\n  \"flight_peak_vertices\": {},\n  \"flight_peak_chunks\": {},\n  \"flight_peak_rss_mb\": {:.2},\n  \"flight_peak_vram_mb\": {:.2}\n}}\n",
            config.scenario.name,
            config.scenario.id,
            config.seed,
            config.scenario.view_distance,
            state.static_fps,
            state.static_frametime_ms,
            state.static_chunks,
            state.static_vertices,
            state.static_vertices / 2,
            state.static_vram_mb,
            state.distance_traveled,
            total_duration_secs,
            total_frames,
            avg_fps,
            one_percent_low_fps,
            point_one_percent_low_fps,
            avg_frametime_ms,
            min_frametime_ms,
            max_frametime_ms,
            stdev_ms,
            avg_verts,
            peak_verts,
            peak_chunks,
            state.peak_rss_mb,
            state.peak_vram_mb,
        );
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = File::create(path) {
            let _ = file.write_all(json.as_bytes());
            println!("[BENCHMARK] Saved structured JSON results to: {}", path);
        }
    }
}

/// Plugin that integrates benchmark systems into Bevy.
pub struct BenchmarkPlugin;

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BenchmarkConfig>()
            .init_resource::<BenchmarkState>()
            .init_resource::<BenchmarkSummary>()
            .add_systems(
                Update,
                benchmark_runner_system.in_set(crate::stage::VoxelStage::PlayerPhysics),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_scenario_standard_properties() {
        let bench = BenchmarkScenario::standard(64);
        assert_eq!(bench.view_distance, 64);
        assert!((bench.flight_distance - 5000.0).abs() < f32::EPSILON);
        assert!((bench.flight_speed - 50.0).abs() < f32::EPSILON);
        assert!(!bench.distance_fog);
    }

    #[test]
    fn test_benchmark_scenario_apply_configures_settings() {
        let scenario = BenchmarkScenario::standard(48);
        let mut graphics = GraphicsSettings::default();

        scenario.apply(&mut graphics);

        assert!(!graphics.vsync);
        assert_eq!(graphics.view_distance, 48);
        assert_eq!(graphics.fps_cap, None);
        assert!(!graphics.distance_fog);
    }

    #[test]
    fn test_benchmark_suite_standard_contains_scenario() {
        let suite = BenchmarkSuite::standard(32);
        assert_eq!(suite.scenarios.len(), 1);
        assert_eq!(suite.seed, BenchmarkSuite::DEFAULT_SEED);
        assert_eq!(suite.scenarios[0].view_distance, 32);
        assert!((suite.scenarios[0].flight_distance - 5000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_benchmark_preset_from_str_name() {
        assert_eq!(
            BenchmarkPreset::from_str_name("flight"),
            Some(BenchmarkPreset::Flight5km)
        );
        assert_eq!(
            BenchmarkPreset::from_str_name("PRODUCTION"),
            Some(BenchmarkPreset::Flight5km)
        );
        assert_eq!(
            BenchmarkPreset::from_str_name("standard"),
            Some(BenchmarkPreset::Flight5km)
        );
        assert_eq!(
            BenchmarkPreset::from_str_name("5km"),
            Some(BenchmarkPreset::Flight5km)
        );
        assert_eq!(BenchmarkPreset::from_str_name("invalid_preset"), None);
    }

    #[test]
    fn test_compute_benchmark_summary_and_verdict() {
        let mut frame_times = vec![10.0; 100];
        // Inject 1 frame of 20ms to test 1% low
        frame_times[99] = 20.0;

        let state = BenchmarkState {
            frame_times_ms: frame_times,
            peak_rss_mb: 150.0,
            peak_vram_mb: 600.0,
            static_chunks: 289,
            chunk_samples: vec![289],
            vertex_samples: vec![100_000],
            distance_traveled: 5000.0,
            ..Default::default()
        };

        let config = BenchmarkConfig::default();
        let graphics = GraphicsSettings::default();
        let summary = compute_benchmark_summary(&state, &config, Some(&graphics));

        assert!(summary.has_results);
        assert_eq!(summary.total_frames, 100);
        assert!(summary.avg_fps > 90.0);
        assert!(summary.one_percent_low_fps > 45.0);
        assert_eq!(summary.tested_view_distance, 16);
        assert!(summary.verdict_title.contains("GREAT") || summary.verdict_title.contains("PERFECT"));
    }
}
