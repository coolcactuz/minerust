use bevy::prelude::*;

use crate::coords::BlockPos;
use crate::fluid::FluidSimulation;
use crate::menu::DevSettings;
use crate::world::{WorldGrid, calculate_biome_and_height};

/// Process physical and virtual memory metrics read safely from the OS.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProcessMemory {
    pub rss_mb: f32,
    pub virt_mb: f32,
}

/// Safely queries the current process memory footprint from `/proc/self/status` on Linux.
/// Returns zeros on non-Linux platforms or when `/proc` is unavailable.
pub fn read_process_memory() -> ProcessMemory {
    parse_process_memory_from_str(&std::fs::read_to_string("/proc/self/status").unwrap_or_default())
}

/// Helper to parse VmRSS and VmSize from a status buffer.
pub fn parse_process_memory_from_str(content: &str) -> ProcessMemory {
    let mut mem = ProcessMemory::default();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb_str = rest.trim().trim_end_matches("kB").trim();
            if let Ok(kb) = kb_str.parse::<f32>() {
                mem.rss_mb = kb / 1024.0;
            }
        } else if let Some(rest) = line.strip_prefix("VmSize:") {
            let kb_str = rest.trim().trim_end_matches("kB").trim();
            if let Ok(kb) = kb_str.parse::<f32>() {
                mem.virt_mb = kb / 1024.0;
            }
        }
    }
    mem
}

/// Rolling frame time tracker for calculating current FPS, frame pacing, and 1% low metrics.
#[derive(Debug)]
pub struct ProfilerFpsTracker {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub one_percent_low_fps: f32,
    pub min_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub timer: f32,
    pub frame_times: Vec<f32>,
}

impl Default for ProfilerFpsTracker {
    fn default() -> Self {
        Self {
            fps: 60.0,
            frame_time_ms: 16.6,
            one_percent_low_fps: 60.0,
            min_frame_time_ms: 16.6,
            max_frame_time_ms: 16.6,
            timer: 0.0,
            frame_times: Vec::with_capacity(128),
        }
    }
}

impl ProfilerFpsTracker {
    /// Records a new frame delta and updates statistics when the measurement window (0.25s) completes.
    pub fn update(&mut self, dt_secs: f32) -> bool {
        self.timer += dt_secs;
        self.frame_times.push(dt_secs * 1000.0);

        if self.timer >= 0.25 && !self.frame_times.is_empty() {
            let count = self.frame_times.len();
            let total_ms: f32 = self.frame_times.iter().sum();
            self.frame_time_ms = total_ms / count as f32;
            self.fps = if self.frame_time_ms > 0.0 {
                1000.0 / self.frame_time_ms
            } else {
                0.0
            };

            // 1% Low: sort ascending to inspect the worst (99th percentile) frame times
            self.frame_times
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p99_idx = ((count as f32 * 0.99) as usize).min(count - 1);
            let p99_ms = self.frame_times[p99_idx];
            self.one_percent_low_fps = if p99_ms > 0.0 {
                1000.0 / p99_ms
            } else {
                self.fps
            };

            self.min_frame_time_ms = self.frame_times[0];
            self.max_frame_time_ms = self.frame_times[count - 1];

            self.timer = 0.0;
            self.frame_times.clear();
            true
        } else {
            false
        }
    }
}

/// Root node for the in-game profiling HUD overlay.
#[derive(Component)]
pub struct ProfilingHudRoot;

/// Text component containing formatted real-time performance telemetry.
#[derive(Component)]
pub struct ProfilingHudText;

/// Spawns the in-game telemetry HUD overlay with a high-contrast theme.
pub fn setup_profiling_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.5)),
                max_width: Val::Px(720.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.88)),
            BorderColor::all(Color::srgba(0.2, 0.65, 0.95, 0.8)),
            ProfilingHudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("MINERUST ENGINE PROFILER [F3: Toggle]\nLoading real-time telemetry..."),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::srgb(0.35, 1.0, 0.55)),
                ProfilingHudText,
            ));
        });
}

/// Updates the profiling HUD text and toggles visibility based on `DevSettings::show_debug_hud`.
pub fn update_profiling_hud_system(
    time: Res<Time>,
    mut fps: Local<ProfilerFpsTracker>,
    world: Option<Res<WorldGrid>>,
    dev_settings: Option<Res<DevSettings>>,
    fluid_sim: Option<Res<FluidSimulation>>,
    camera_query: Query<&Transform, With<crate::camera::FpsCamera>>,
    mut text_query: Query<&mut Text, With<ProfilingHudText>>,
    mut root_query: Query<&mut Visibility, With<ProfilingHudRoot>>,
) {
    let show_hud = dev_settings.as_ref().is_some_and(|d| d.show_debug_hud);

    if let Ok(mut vis) = root_query.single_mut() {
        let target = if show_hud {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }

    if !show_hud {
        return;
    }

    let dt = time.delta_secs();
    let updated = fps.update(dt);

    if updated {
        if let Ok(mut text) = text_query.single_mut() {
            let (
                chunks_loaded,
                meshes_active,
                gen_q,
                mesh_q,
                total_verts,
                cache_len,
                in_progress,
                seed_val,
            ) = if let Some(ref w) = world {
                (
                    w.chunks.len(),
                    w.chunk_entities.len(),
                    w.generation_queue.len(),
                    w.mesh_queue.len(),
                    w.total_vertices,
                    w.chunk_cache.len(),
                    w.in_progress_chunks.len() + w.in_progress_meshes.len(),
                    w.seed.0,
                )
            } else {
                (0, 0, 0, 0, 0, 0, 0, 0)
            };

            // Memory: Physical RSS and Virtual Memory
            let mem = read_process_memory();
            let ram_str = if mem.rss_mb > 0.0 {
                let rss_str = if mem.rss_mb >= 1024.0 {
                    format!("{:.2} GB", mem.rss_mb / 1024.0)
                } else {
                    format!("{:.1} MB", mem.rss_mb)
                };
                let virt_str = if mem.virt_mb >= 1024.0 {
                    format!("{:.2} GB", mem.virt_mb / 1024.0)
                } else {
                    format!("{:.1} MB", mem.virt_mb)
                };
                format!("RSS: {} | Virt: {}", rss_str, virt_str)
            } else {
                "RSS: N/A (Linux only)".to_string()
            };

            // Geometry & VRAM Estimation:
            // 48 bytes vertex attributes + 6 bytes index buffer per vertex = 54 bytes
            let geom_bytes = total_verts * 54;
            let geom_mb = geom_bytes as f32 / (1024.0 * 1024.0);
            let total_vram_mb = geom_mb + 32.0; // ~32 MB for atlas texture, swapchain, depth buffer

            let vram_str = if total_vram_mb >= 1024.0 {
                format!("{:.2} GB", total_vram_mb / 1024.0)
            } else {
                format!("{:.1} MB", total_vram_mb)
            };

            let verts_str = if total_verts >= 1_000_000 {
                format!("{:.2}M", total_verts as f32 / 1_000_000.0)
            } else if total_verts >= 1_000 {
                format!("{:.1}k", total_verts as f32 / 1_000.0)
            } else {
                format!("{}", total_verts)
            };

            let triangles = total_verts / 2;
            let visible_quads = total_verts / 4;
            let voxels_in_ram = (chunks_loaded * 32_768) as f32 / 1_000_000.0;

            // Player position & Biome
            let (pos_str, biome_str) = if let Ok(cam_tf) = camera_query.single() {
                let p = cam_tf.translation;
                let bp = BlockPos::new(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
                let (cp, lp) = bp.to_chunk_and_local();
                let biome_name = if let Some(ref w) = world {
                    let (b, h, _) = calculate_biome_and_height(p.x as f64, p.z as f64, &w.noise);
                    format!("{:?} (Surf: {})", b, h)
                } else {
                    "Unknown".to_string()
                };
                (
                    format!(
                        "World: X={:.1} Y={:.1} Z={:.1} | Chunk: ({}, {}) | Local: [{}, {}, {}]",
                        p.x, p.y, p.z, cp.0.x, cp.0.y, lp.x, lp.y, lp.z
                    ),
                    biome_name,
                )
            } else {
                ("World: X=-- Y=-- Z=--".to_string(), "Unknown".to_string())
            };

            // Fluid simulation status
            let fluid_queue_len = fluid_sim.as_ref().map_or(0, |f| f.queue.len());

            // Optimizations status
            let (cull, max_y, budget, async_m, greedy, lod) = if let Some(ref dev) = dev_settings {
                (
                    if dev.backface_culling { "ON" } else { "OFF" },
                    if dev.max_y_skip { "ON" } else { "OFF" },
                    if dev.mesh_budget { "ON (6/fr)" } else { "OFF" },
                    if dev.async_meshing { "ON" } else { "OFF" },
                    if dev.greedy_meshing {
                        "ON (~75% drop)"
                    } else {
                        "OFF"
                    },
                    if dev.distance_lod {
                        format!("ON ({}ch)", dev.lod_threshold)
                    } else {
                        "OFF".to_string()
                    },
                )
            } else {
                (
                    "ON",
                    "ON",
                    "ON (6/fr)",
                    "ON",
                    "ON (~75% drop)",
                    "ON (4ch)".to_string(),
                )
            };

            *text = Text::new(format!(
                "=== MINERUST ENGINE PROFILER [F3: Toggle HUD] ===\n\
                 PERFORMANCE:  FPS: {:.0} ({:.1} ms) | 1% Low: {:.0} FPS | Min/Max: {:.1}ms / {:.1}ms\n\
                 PROCESS RAM:  {} | Cache: {} entries\n\
                 VRAM (EST):   Total: ~{} [Geom: {:.1} MB | Textures/FB: ~32.0 MB]\n\
                 GEOMETRY:     Active Meshes: {} | Verts: {} | Tris: {} | Visible Quads: {}\n\
                 VOXEL WORLD:  Chunks Loaded: {} | Voxels in RAM: ~{:.2}M | Seed: {}\n\
                 STREAMING:    Gen Queue: {} | Mesh Queue: {} | Active Tasks: {}\n\
                 FLUID ENGINE: Water Queue: {} | Tick Rate: 1.0s batch\n\
                 PLAYER POS:   {} | Biome: {}\n\
                 OPTIMIZATION: [Greedy: {}] [LOD: {}] [Max-Y: {}] [Cull: {}] [Async: {}] [Budget: {}] [Direct-GPU: ON]",
                fps.fps,
                fps.frame_time_ms,
                fps.one_percent_low_fps,
                fps.min_frame_time_ms,
                fps.max_frame_time_ms,
                ram_str,
                cache_len,
                vram_str,
                geom_mb,
                meshes_active,
                verts_str,
                triangles,
                visible_quads,
                chunks_loaded,
                voxels_in_ram,
                seed_val,
                gen_q,
                mesh_q,
                in_progress,
                fluid_queue_len,
                pos_str,
                biome_str,
                greedy,
                lod,
                max_y,
                cull,
                async_m,
                budget,
            ));
        }
    }
}

pub struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_profiling_ui)
            .add_systems(Update, update_profiling_hud_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_memory_parsing() {
        let sample = "Name:\tminerust\nVmSize:\t  1048576 kB\nVmRSS:\t   524288 kB\n";
        let mem = parse_process_memory_from_str(sample);
        assert!((mem.virt_mb - 1024.0).abs() < 0.1);
        assert!((mem.rss_mb - 512.0).abs() < 0.1);
    }

    #[test]
    fn test_process_memory_empty_string_does_not_panic() {
        let mem = parse_process_memory_from_str("");
        assert!(mem.rss_mb.abs() < f32::EPSILON);
        assert!(mem.virt_mb.abs() < f32::EPSILON);
    }

    #[test]
    fn test_fps_tracker_pacing_and_one_percent_low() {
        let mut tracker = ProfilerFpsTracker::default();
        // 10 frames of 16.6ms = 166ms (< 250ms measurement window)
        for _ in 0..10 {
            assert!(!tracker.update(0.0166));
        }
        // Adding 100ms brings total to 266ms (>= 250ms measurement window)
        let updated = tracker.update(0.100);
        assert!(updated, "tracker should update after passing 0.25s window");

        assert!(tracker.fps > 0.0);
        assert!(tracker.frame_time_ms > 0.0);
        assert!(tracker.one_percent_low_fps <= tracker.fps);
        assert!(tracker.max_frame_time_ms >= 100.0);
    }

    #[test]
    fn test_profiling_hud_visibility_toggle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ProfilePlugin);

        app.insert_resource(DevSettings {
            show_debug_hud: false,
            ..Default::default()
        });

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&Visibility, With<ProfilingHudRoot>>();
        let vis = query.single(app.world()).unwrap();
        assert_eq!(*vis, Visibility::Hidden);

        // Toggle on
        app.world_mut().resource_mut::<DevSettings>().show_debug_hud = true;
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&Visibility, With<ProfilingHudRoot>>();
        let vis = query.single(app.world()).unwrap();
        assert_eq!(*vis, Visibility::Inherited);
    }
}
