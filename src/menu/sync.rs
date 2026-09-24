use bevy::prelude::*;

use super::descriptions::get_option_description;
use super::types::{
    BenchmarkAvgFpsText, BenchmarkBannerText, BenchmarkChunksText, BenchmarkFrametimeText,
    BenchmarkMinMaxFrametimeText, BenchmarkOnePercentLowText, BenchmarkP99Text, BenchmarkRamText,
    BenchmarkRunningBanner, BenchmarkSettingsText, BenchmarkVerdictDescText,
    BenchmarkVerdictTitleText, BenchmarkVertsText, BenchmarkVramText, DebugHudBtnText,
    DistanceFogBtnText, FpsCapBtnText, FpsCapFill, FpsCapThumb, FpsLimiter, FullscreenBtnText,
    GraphicsGreedyBtnText, GraphicsGreedyFill, GraphicsGreedyThumb, GraphicsLodBtnText,
    GraphicsLodFill, GraphicsLodThumb, GraphicsSettings, MenuButtonAction, OptionTooltipCard,
    OptionTooltipDesc, OptionTooltipHeader, OptionTooltipImpact, OptionTooltipTitle, ProfilerState,
    ShadowsBtnText, ViewDistanceBtnText, ViewDistanceFill, ViewDistanceThumb, VsyncBtnText,
};
use crate::benchmark::{BenchmarkConfig, BenchmarkPhase, BenchmarkState, BenchmarkSummary};

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    profiler_state: Option<Res<ProfilerState>>,
    mut query: Query<(
        &mut Text,
        Option<&VsyncBtnText>,
        Option<&FullscreenBtnText>,
        Option<&ShadowsBtnText>,
        Option<&DistanceFogBtnText>,
        Option<&DebugHudBtnText>,
        Option<&FpsCapBtnText>,
        Option<&ViewDistanceBtnText>,
        Option<&GraphicsGreedyBtnText>,
        Option<&GraphicsLodBtnText>,
    )>,
) {
    let hud_visible = profiler_state.as_ref().is_some_and(|p| p.visible);

    for (mut text, vsync, fs, shadows, fog, hud, fps, dist, greedy, lod) in &mut query {
        if vsync.is_some() {
            *text = Text::new(format!(
                "VSync: {}",
                if settings.vsync { "ON" } else { "OFF" }
            ));
        } else if fs.is_some() {
            *text = Text::new(format!(
                "Display: {}",
                if settings.fullscreen {
                    "Fullscreen"
                } else {
                    "Windowed"
                }
            ));
        } else if shadows.is_some() {
            *text = Text::new(format!(
                "Shadows: {}",
                if settings.shadows { "ON" } else { "OFF" }
            ));
        } else if fog.is_some() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if settings.distance_fog { "ON" } else { "OFF" }
            ));
        } else if hud.is_some() {
            *text = Text::new(format!(
                "Profiler HUD (F3): {}",
                if hud_visible { "ON" } else { "OFF" }
            ));
        } else if fps.is_some() {
            *text = Text::new(settings.fps_cap_label());
        } else if dist.is_some() {
            *text = Text::new(settings.view_distance_label());
        } else if greedy.is_some() {
            *text = Text::new(settings.greedy_label());
        } else if lod.is_some() {
            *text = Text::new(settings.lod_label());
        }
    }
}

pub fn sync_graphics_settings_to_bevy(
    mut commands: Commands,
    settings: Res<GraphicsSettings>,
    mut dir_lights: Query<&mut DirectionalLight>,
    mut camera_query: Query<
        (Entity, Option<&mut bevy::pbr::DistanceFog>),
        With<crate::camera::FpsCamera>,
    >,
    mut last_config: Local<Option<(bool, i32, bool)>>,
) {
    let current_config = (settings.distance_fog, settings.view_distance, settings.shadows);

    // 1. If fog or view_distance changed, update Distance Fog in real-time
    if last_config.map_or(true, |last| {
        last.0 != current_config.0 || last.1 != current_config.1
    }) {
        let max_dist = (settings.view_distance as f32 * 16.0).max(64.0);
        let fog_start = (max_dist * 0.70).max(48.0);
        let fog_end = (max_dist - 2.0).max(64.0);

        for (cam_entity, mut maybe_fog) in &mut camera_query {
            if settings.distance_fog {
                if let Some(ref mut fog) = maybe_fog {
                    fog.color = Color::srgb(0.53, 0.81, 0.98);
                    fog.falloff = bevy::pbr::FogFalloff::Linear {
                        start: fog_start,
                        end: fog_end,
                    };
                } else {
                    commands.entity(cam_entity).insert(bevy::pbr::DistanceFog {
                        color: Color::srgb(0.53, 0.81, 0.98),
                        falloff: bevy::pbr::FogFalloff::Linear {
                            start: fog_start,
                            end: fog_end,
                        },
                        ..default()
                    });
                }
            } else if maybe_fog.is_some() {
                commands
                    .entity(cam_entity)
                    .remove::<bevy::pbr::DistanceFog>();
            }
        }
    }

    // 2. If shadows setting changed, update DirectionalLight in real-time
    if last_config.map_or(true, |last| last.2 != current_config.2) {
        for mut light in &mut dir_lights {
            light.shadow_maps_enabled = settings.shadows;
        }
    }

    *last_config = Some(current_config);
}

pub fn update_option_tooltip_system(
    interaction_query: Query<(&Interaction, &MenuButtonAction), With<Button>>,
    mut last_hovered: Local<Option<MenuButtonAction>>,
    mut text_query: Query<(
        &mut Text,
        Option<&OptionTooltipHeader>,
        Option<&OptionTooltipTitle>,
        Option<&OptionTooltipDesc>,
        Option<&OptionTooltipImpact>,
    )>,
    mut card_query: Query<&mut BorderColor, With<OptionTooltipCard>>,
) {
    let currently_hovered = interaction_query
        .iter()
        .find(|(interaction, _)| {
            **interaction == Interaction::Hovered || **interaction == Interaction::Pressed
        })
        .map(|(_, action)| *action);

    if *last_hovered != currently_hovered {
        *last_hovered = currently_hovered;

        if let Some(action) = currently_hovered {
            if let Some(desc) = get_option_description(&action) {
                for (mut text, header, title, desc_opt, impact) in &mut text_query {
                    if header.is_some() {
                        *text = Text::new(format!("[ {} ]", desc.header));
                    } else if title.is_some() {
                        *text = Text::new(desc.title);
                    } else if desc_opt.is_some() {
                        *text = Text::new(desc.description);
                    } else if impact.is_some() {
                        *text = Text::new(desc.impact);
                    }
                }
                for mut border in &mut card_query {
                    *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.2));
                }
            }
        } else {
            for (mut text, header, title, desc_opt, impact) in &mut text_query {
                if header.is_some() {
                    *text = Text::new("[ SETTING INFO ]");
                } else if title.is_some() {
                    *text = Text::new("Hover over any setting");
                } else if desc_opt.is_some() {
                    *text = Text::new(
                        "Move your mouse over any graphic or performance setting on the left to inspect its technical details, rendering behavior, and performance impact.",
                    );
                } else if impact.is_some() {
                    *text = Text::new(
                        "- All MineRust optimizations are built-in for maximum 60+ FPS stability.",
                    );
                }
            }
            for mut border in &mut card_query {
                *border = BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8));
            }
        }
    }
}

pub fn update_slider_visuals_system(
    settings: Res<GraphicsSettings>,
    mut query: Query<(
        &mut Node,
        Option<&FpsCapFill>,
        Option<&FpsCapThumb>,
        Option<&ViewDistanceFill>,
        Option<&ViewDistanceThumb>,
        Option<&GraphicsGreedyFill>,
        Option<&GraphicsLodFill>,
        Option<&GraphicsGreedyThumb>,
        Option<&GraphicsLodThumb>,
    )>,
    mut last_ratios: Local<Option<(f32, f32, f32, f32)>>,
) {
    let fps_ratio = settings.fps_cap_ratio();
    let dist_ratio = settings.view_distance_ratio();
    let greedy_ratio = settings.greedy_ratio();
    let lod_ratio = settings.lod_ratio();
    let current_ratios = (fps_ratio, dist_ratio, greedy_ratio, lod_ratio);

    if last_ratios.is_some_and(|r| r == current_ratios) {
        return;
    }
    *last_ratios = Some(current_ratios);

    for (mut node, fps_f, fps_t, dist_f, dist_t, greedy_f, lod_f, greedy_t, lod_t) in &mut query {
        if fps_f.is_some() {
            let fill_pct = if fps_ratio <= 0.0 {
                0.0
            } else {
                (fps_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if fps_t.is_some() {
            node.left = Val::Percent(fps_ratio * 95.0);
        } else if dist_f.is_some() {
            let fill_pct = if dist_ratio <= 0.0 {
                0.0
            } else {
                (dist_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if dist_t.is_some() {
            node.left = Val::Percent(dist_ratio * 95.0);
        } else if greedy_f.is_some() {
            let fill_pct = if greedy_ratio <= 0.0 {
                0.0
            } else {
                (greedy_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if greedy_t.is_some() {
            node.left = Val::Percent(greedy_ratio * 95.0);
        } else if lod_f.is_some() {
            let fill_pct = if lod_ratio <= 0.0 {
                0.0
            } else {
                (lod_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if lod_t.is_some() {
            node.left = Val::Percent(lod_ratio * 95.0);
        }
    }
}

pub fn auto_save_graphics_settings_system(settings: Res<GraphicsSettings>) {
    if settings.is_changed() {
        settings.save_to_disk();
    }
}

pub fn fps_limiter_system(
    settings: Res<GraphicsSettings>,
    bench_config: Option<Res<BenchmarkConfig>>,
    mut limiter: ResMut<FpsLimiter>,
) {
    if bench_config.as_ref().is_some_and(|c| c.enabled) {
        limiter.last_frame_instant = None;
        return;
    }

    if let Some(cap) = settings.fps_cap {
        let target_frame_duration = std::time::Duration::from_secs_f64(1.0 / cap as f64);
        if let Some(last) = limiter.last_frame_instant {
            let elapsed = last.elapsed();
            if let Some(remaining) = target_frame_duration.checked_sub(elapsed) {
                std::thread::sleep(remaining);
            }
        }
        limiter.last_frame_instant = Some(std::time::Instant::now());
    } else {
        limiter.last_frame_instant = None;
    }
}

pub fn update_benchmark_banner_system(
    config: Option<Res<BenchmarkConfig>>,
    state: Option<Res<BenchmarkState>>,
    mut banner_query: Query<&mut Visibility, With<BenchmarkRunningBanner>>,
    mut text_query: Query<&mut Text, With<BenchmarkBannerText>>,
) {
    let (Some(config), Some(state)) = (config, state) else {
        return;
    };

    let is_running = config.enabled && !config.is_cli && !state.completed;
    for mut vis in &mut banner_query {
        let target = if is_running {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != target {
            *vis = target;
        }
    }

    if is_running {
        let total_dist = config.scenario.flight_distance;
        let dist = state.distance_traveled.min(total_dist);
        let pct = if total_dist > 0.0 {
            (dist / total_dist * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let status_str = match state.phase {
            BenchmarkPhase::InitializingWorld => {
                format!(
                    "[ BENCHMARK: PREGENERATING CHUNKS ] Elapsed: {:.1}s | Press [ESC] to Cancel",
                    state.elapsed
                )
            }
            BenchmarkPhase::StationarySettle => {
                format!(
                    "[ BENCHMARK: MEASURING BASELINE ] Settling 1.5s ({:.1}s) | Press [ESC] to Cancel",
                    state.stationary_timer
                )
            }
            BenchmarkPhase::FlightRecording => {
                format!(
                    "[ BENCHMARK IN PROGRESS ] Distance: {:.0}m / {:.0}m ({:.0}%) | Press [ESC] to Cancel",
                    dist, total_dist, pct
                )
            }
            BenchmarkPhase::Completed => "[ BENCHMARK COMPLETED ]".to_string(),
        };

        for mut text in &mut text_query {
            *text = Text::new(&status_str);
        }
    }
}

pub fn update_benchmark_results_ui_system(
    summary: Option<Res<BenchmarkSummary>>,
    mut query: Query<(
        &mut Text,
        Option<&mut TextColor>,
        Option<&BenchmarkAvgFpsText>,
        Option<&BenchmarkOnePercentLowText>,
        Option<&BenchmarkP99Text>,
        Option<&BenchmarkFrametimeText>,
        Option<&BenchmarkMinMaxFrametimeText>,
        Option<&BenchmarkRamText>,
        Option<&BenchmarkVramText>,
        Option<&BenchmarkChunksText>,
        Option<&BenchmarkVertsText>,
        Option<&BenchmarkSettingsText>,
        Option<&BenchmarkVerdictTitleText>,
        Option<&BenchmarkVerdictDescText>,
    )>,
) {
    let Some(summary) = summary else {
        return;
    };
    if !summary.is_changed() || !summary.has_results {
        return;
    }

    for (
        mut text,
        mut text_color,
        avg_fps,
        low_fps,
        p99,
        frametime,
        min_max,
        ram,
        vram,
        chunks,
        verts,
        settings,
        verdict_title,
        verdict_desc,
    ) in &mut query
    {
        if avg_fps.is_some() {
            *text = Text::new(format!("Avg: {:.1} FPS", summary.avg_fps));
        } else if low_fps.is_some() {
            *text = Text::new(format!("1% Low: {:.1} FPS", summary.one_percent_low_fps));
        } else if p99.is_some() {
            *text = Text::new(format!("99th %: {:.2} ms", summary.p99_frametime_ms));
        } else if frametime.is_some() {
            *text = Text::new(format!("Frametime: {:.2} ms", summary.avg_frametime_ms));
        } else if min_max.is_some() {
            *text = Text::new(format!(
                "Min/Max: {:.1} / {:.1} ms",
                summary.min_frametime_ms, summary.max_frametime_ms
            ));
        } else if ram.is_some() {
            *text = Text::new(format!("RAM (RSS): {:.1} MB", summary.peak_rss_mb));
        } else if vram.is_some() {
            *text = Text::new(if summary.peak_vram_mb > 0.0 {
                format!("VRAM: {:.1} MB", summary.peak_vram_mb)
            } else {
                "VRAM: N/A".to_string()
            });
        } else if chunks.is_some() {
            *text = Text::new(format!("Active Chunks: {}", summary.peak_chunks_active));
        } else if verts.is_some() {
            let tris_k = (summary.peak_vertices / 2) as f32 / 1000.0;
            *text = Text::new(format!(
                "Peak Geometry: {} verts (~{:.1}k tris)",
                summary.peak_vertices, tris_k
            ));
        } else if settings.is_some() {
            *text = Text::new(format!(
                "Render Dist: {} chunks\nGreedy Meshing: {}\nSloped LOD: {}\nShadows: {}\nFog: {}",
                summary.tested_view_distance,
                summary.tested_greedy,
                summary.tested_lod,
                if summary.tested_shadows { "ON" } else { "OFF" },
                if summary.tested_fog { "ON" } else { "OFF" }
            ));
        } else if verdict_title.is_some() {
            *text = Text::new(format!("[ HARDWARE VERDICT ] {}", summary.verdict_title));
            if let Some(ref mut color) = text_color {
                if summary.avg_fps >= 100.0 && summary.one_percent_low_fps >= 60.0 {
                    color.0 = Color::srgb(0.35, 1.0, 0.55);
                } else if summary.avg_fps >= 60.0 {
                    color.0 = Color::srgb(0.4, 0.85, 1.0);
                } else if summary.avg_fps >= 45.0 {
                    color.0 = Color::srgb(1.0, 0.85, 0.2);
                } else {
                    color.0 = Color::srgb(1.0, 0.45, 0.4);
                }
            }
        } else if verdict_desc.is_some() {
            *text = Text::new(&summary.verdict_desc);
        }
    }
}
