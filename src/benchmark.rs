use std::fs::File;
use std::io::Write;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::camera::FpsCamera;
use crate::menu::{DevSettings, GraphicsSettings};
use crate::physics::PlayerPhysics;
use crate::profile::read_process_memory;
use crate::world::WorldGrid;

/// Standard predefined optimization profiles for scientific benchmarking comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchmarkPreset {
    /// Raw unoptimized baseline: Naive meshing (1 quad per block face), no LOD, no culling, no max_y skip.
    Baseline,
    /// Culling optimizations only: Backface culling + Atmosphere Max-Y skip active, Naive meshing.
    Culling,
    /// Greedy Meshing active (merging coplanar faces), no distance LOD.
    Greedy,
    /// Greedy Meshing + 3D Sloped Heightfield LOD active.
    SlopedLod,
    /// Full production pipeline: Greedy + Sloped LOD + Culling + Max-Y + Distance Fog.
    Production,
}

impl BenchmarkPreset {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "baseline" | "none" | "raw" => Some(Self::Baseline),
            "culling" | "cull" => Some(Self::Culling),
            "greedy" => Some(Self::Greedy),
            "sloped" | "lod" | "sloped_lod" => Some(Self::SlopedLod),
            "production" | "full" | "all" => Some(Self::Production),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Baseline => "Baseline (No Optimizations)",
            Self::Culling => "Backface Culling & Max-Y Skip",
            Self::Greedy => "Greedy Meshing",
            Self::SlopedLod => "Greedy + Sloped Heightfield LOD",
            Self::Production => "Full Production (All Optimizations)",
        }
    }

    pub fn apply(self, dev: &mut DevSettings, graphics: &mut GraphicsSettings) {
        match self {
            Self::Baseline => {
                graphics.greedy_meshing = false;
                graphics.distance_lod = false;
                graphics.distance_fog = false;
                dev.greedy_meshing = false;
                dev.distance_lod = false;
                dev.backface_culling = false;
                dev.max_y_skip = false;
                dev.distance_fog = false;
            }
            Self::Culling => {
                graphics.greedy_meshing = false;
                graphics.distance_lod = false;
                graphics.distance_fog = false;
                dev.greedy_meshing = false;
                dev.distance_lod = false;
                dev.backface_culling = true;
                dev.max_y_skip = true;
                dev.distance_fog = false;
            }
            Self::Greedy => {
                graphics.greedy_meshing = true;
                graphics.greedy_threshold = 0; // Greedy everywhere
                graphics.distance_lod = false;
                graphics.distance_fog = false;
                dev.greedy_meshing = true;
                dev.distance_lod = false;
                dev.backface_culling = true;
                dev.max_y_skip = true;
                dev.distance_fog = false;
            }
            Self::SlopedLod => {
                graphics.greedy_meshing = true;
                graphics.greedy_threshold = 2;
                graphics.distance_lod = true;
                graphics.lod_threshold = 4;
                graphics.distance_fog = false;
                dev.greedy_meshing = true;
                dev.distance_lod = true;
                dev.backface_culling = true;
                dev.max_y_skip = true;
                dev.distance_fog = false;
            }
            Self::Production => {
                graphics.greedy_meshing = true;
                graphics.greedy_threshold = 2;
                graphics.distance_lod = true;
                graphics.lod_threshold = 4;
                graphics.distance_fog = true;
                dev.greedy_meshing = true;
                dev.distance_lod = true;
                dev.backface_culling = true;
                dev.max_y_skip = true;
                dev.distance_fog = true;
            }
        }
    }
}

/// Configuration options for the automated reproducible benchmark mode.
#[derive(Resource, Clone, Debug)]
pub struct BenchmarkConfig {
    pub enabled: bool,
    pub preset_name: String,
    pub seed: u64,
    pub view_distance: i32,
    pub warmup_duration_secs: f32,
    pub target_distance_meters: f32,
    pub flight_speed: f32,
    pub flight_altitude: f32,
    pub output_path: Option<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset_name: "Default".to_string(),
            seed: 133742,
            view_distance: 16,
            warmup_duration_secs: 2.5,
            target_distance_meters: 1000.0, // 1 km standard run
            flight_speed: 50.0,             // 50 m/s (180 km/h) = 20s recording for 1km
            flight_altitude: 92.0,
            output_path: Some("benchmark_results.json".to_string()),
        }
    }
}

/// Dynamic runtime state tracking benchmark progress and statistical telemetry.
#[derive(Resource, Default)]
pub struct BenchmarkState {
    pub elapsed: f32,
    pub start_z: Option<f32>,
    pub distance_traveled: f32,
    pub is_recording: bool,
    pub frame_times_ms: Vec<f32>,
    pub vertex_samples: Vec<usize>,
    pub chunk_samples: Vec<usize>,
    pub peak_rss_mb: f32,
    pub completed: bool,
}

/// System that executes the automated deterministic benchmark trajectory and records frame latencies.
pub fn benchmark_runner_system(
    time: Res<Time>,
    config: Res<BenchmarkConfig>,
    mut state: ResMut<BenchmarkState>,
    mut player_query: Query<(&mut Transform, &mut FpsCamera, &mut PlayerPhysics)>,
    world: Option<Res<WorldGrid>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    if !config.enabled || state.completed {
        return;
    }

    let dt = time.delta_secs();
    state.elapsed += dt;

    // 1. Move player camera deterministically along +Z axis at constant velocity
    if let Ok((mut transform, mut fps, mut physics)) = player_query.single_mut() {
        physics.is_flying = true;
        physics.velocity = Vec3::ZERO;
        fps.yaw = 0.0;
        fps.pitch = -0.06; // Look slightly downwards across terrain

        // Maintain constant altitude and advance steadily forward along +Z
        transform.translation.x = 0.0;
        transform.translation.y = config.flight_altitude;
        transform.translation.z += config.flight_speed * dt;
        transform.rotation = Quat::from_rotation_y(fps.yaw) * Quat::from_rotation_x(fps.pitch);

        if state.is_recording {
            let start = state.start_z.get_or_insert(transform.translation.z);
            state.distance_traveled = (transform.translation.z - *start).abs();
        }
    }

    // 2. Track peak physical RAM (VmRSS) from OS
    let mem = read_process_memory();
    if mem.rss_mb > state.peak_rss_mb {
        state.peak_rss_mb = mem.rss_mb;
    }

    // 3. Warmup phase: let initial chunks load before collecting metrics
    if state.elapsed < config.warmup_duration_secs {
        return;
    }

    if !state.is_recording {
        state.is_recording = true;
        if let Ok((tf, _, _)) = player_query.single() {
            state.start_z = Some(tf.translation.z);
        }
        println!(
            "\n[BENCHMARK] Warmup finished. Target Distance: {:.0}m ({:.2} km) at {:.1} m/s...",
            config.target_distance_meters,
            config.target_distance_meters / 1000.0,
            config.flight_speed
        );
    }

    let frame_ms = dt * 1000.0;
    state.frame_times_ms.push(frame_ms);

    if let Some(ref w) = world {
        state.vertex_samples.push(w.total_vertices);
        state.chunk_samples.push(w.chunks.len());
    }

    // 4. Check if exact target distance (e.g. 1000 meters) has been completed
    if state.distance_traveled >= config.target_distance_meters {
        state.completed = true;
        print_and_save_benchmark_report(&state, &config);
        exit_writer.write(AppExit::Success);
    }
}

/// Analyzes collected frame latency distributions and outputs a comprehensive report.
fn print_and_save_benchmark_report(state: &BenchmarkState, config: &BenchmarkConfig) {
    let total_frames = state.frame_times_ms.len();
    if total_frames == 0 {
        println!("[BENCHMARK] Error: No frames were recorded during benchmark.");
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

    // Sort ascending to calculate percentile latencies (1% low and 0.1% low)
    let mut sorted_times = state.frame_times_ms.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p99_idx = ((total_frames as f32 * 0.99) as usize).min(total_frames.saturating_sub(1));
    let p99_ms = sorted_times[p99_idx];
    let one_percent_low_fps = if p99_ms > 0.0 { 1000.0 / p99_ms } else { 0.0 };

    let p999_idx = ((total_frames as f32 * 0.999) as usize).min(total_frames.saturating_sub(1));
    let p999_ms = sorted_times[p999_idx];
    let point_one_percent_low_fps = if p999_ms > 0.0 { 1000.0 / p999_ms } else { 0.0 };

    // Standard deviation of frametimes (jitter metric)
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
    println!("             MINERUST AUTOMATED BENCHMARK REPORT             ");
    println!("============================================================");
    println!("  Preset Profile       : \x1b[1;36m{}\x1b[0m", config.preset_name);
    println!("  World Seed           : {} (Render Distance: {} chunks)", config.seed, config.view_distance);
    println!("  Distance Traveled    : \x1b[1;32m{:.1} m ({:.2} km)\x1b[0m in {:.2} s", state.distance_traveled, state.distance_traveled / 1000.0, total_duration_secs);
    println!("  Recorded Frames      : {} frames (Speed: {:.1} m/s)", total_frames, config.flight_speed);
    println!("------------------------------------------------------------");
    println!("  AVERAGE FRAMERATE    : \x1b[1;32m{:.1} FPS\x1b[0m", avg_fps);
    println!("  1% LOW FRAMERATE     : \x1b[1;33m{:.1} FPS\x1b[0m (p99 latency: {:.2} ms)", one_percent_low_fps, p99_ms);
    println!("  0.1% LOW FRAMERATE   : \x1b[1;31m{:.1} FPS\x1b[0m (p99.9 latency: {:.2} ms)", point_one_percent_low_fps, p999_ms);
    println!("------------------------------------------------------------");
    println!("  Average Frametime    : {:.2} ms (StDev / Jitter: {:.2} ms)", avg_frametime_ms, stdev_ms);
    println!("  Frametime Min / Max  : {:.2} ms ({:.1} FPS) / {:.2} ms ({:.1} FPS)", min_frametime_ms, max_fps, max_frametime_ms, min_fps);
    println!("------------------------------------------------------------");
    println!("  Peak Loaded Chunks   : {} chunks", peak_chunks);
    println!("  Average Geometry     : {} vertices (~{} triangles)", avg_verts, avg_verts / 2);
    println!("  Peak Geometry        : {} vertices (~{} triangles)", peak_verts, peak_verts / 2);
    println!("  Peak RAM (VmRSS)     : {:.1} MB", state.peak_rss_mb);
    println!("============================================================\n");

    if let Some(ref path) = config.output_path {
        let json = format!(
            "{{\n  \"preset\": \"{}\",\n  \"seed\": {},\n  \"view_distance\": {},\n  \"distance_meters\": {:.1},\n  \"duration_secs\": {:.3},\n  \"total_frames\": {},\n  \"avg_fps\": {:.2},\n  \"one_percent_low_fps\": {:.2},\n  \"point_one_percent_low_fps\": {:.2},\n  \"avg_frametime_ms\": {:.3},\n  \"min_frametime_ms\": {:.3},\n  \"max_frametime_ms\": {:.3},\n  \"frametime_stdev_ms\": {:.3},\n  \"avg_vertices\": {},\n  \"peak_vertices\": {},\n  \"peak_chunks\": {},\n  \"peak_rss_mb\": {:.2}\n}}\n",
            config.preset_name,
            config.seed,
            config.view_distance,
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
        );
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
            .add_systems(Update, benchmark_runner_system.in_set(crate::stage::VoxelStage::PlayerPhysics));
    }
}
