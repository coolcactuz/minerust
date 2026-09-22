use bevy::prelude::*;

use super::descriptions::get_option_description;
use super::types::{
    AsyncMeshingBtnText, BackfaceCullingBtnText, DebugHudBtnText, DevSettings, DistanceFogBtnText,
    DistanceLodBtnText, FpsCapBtnText, FpsCapFill, FpsCapThumb, FpsLimiter, FullscreenBtnText,
    GraphicsGreedyBtnText, GraphicsGreedyFill, GraphicsGreedyThumb, GraphicsLodBtnText,
    GraphicsLodFill, GraphicsLodThumb, GraphicsSettings, GreedyMeshingBtnText, MaxYSkipBtnText,
    MenuButtonAction, MeshBudgetBtnText, OptionTooltipCard, OptionTooltipDesc, OptionTooltipHeader,
    OptionTooltipImpact, OptionTooltipTitle, PregenMarginBtnText, ShadowsBtnText,
    ViewDistanceBtnText, ViewDistanceFill, ViewDistanceThumb, VsyncBtnText,
};

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    mut query: Query<(
        &mut Text,
        Option<&VsyncBtnText>,
        Option<&FullscreenBtnText>,
        Option<&DistanceFogBtnText>,
        Option<&FpsCapBtnText>,
        Option<&ViewDistanceBtnText>,
        Option<&GraphicsGreedyBtnText>,
        Option<&GraphicsLodBtnText>,
    )>,
) {
    if !settings.is_changed() {
        return;
    }
    for (mut text, vsync, fs, fog, fps, dist, greedy, lod) in &mut query {
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
        } else if fog.is_some() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if settings.distance_fog { "ON" } else { "OFF" }
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

pub fn update_dev_button_text_system(
    dev_settings: Option<Res<DevSettings>>,
    mut query: Query<(
        &mut Text,
        Option<&BackfaceCullingBtnText>,
        Option<&ShadowsBtnText>,
        Option<&MaxYSkipBtnText>,
        Option<&MeshBudgetBtnText>,
        Option<&AsyncMeshingBtnText>,
        Option<&GreedyMeshingBtnText>,
        Option<&DistanceLodBtnText>,
        Option<&DebugHudBtnText>,
        Option<&PregenMarginBtnText>,
    )>,
) {
    let Some(dev) = dev_settings else {
        return;
    };
    if !dev.is_changed() {
        return;
    }
    for (mut text, cull, shadow, max_y, budget, async_m, greedy, lod, hud, margin) in &mut query {
        if cull.is_some() {
            *text = Text::new(format!(
                "Backface Culling: {}",
                if dev.backface_culling { "ON" } else { "OFF" }
            ));
        } else if shadow.is_some() {
            *text = Text::new(format!(
                "Dynamic Shadows: {}",
                if dev.shadows_enabled { "ON" } else { "OFF" }
            ));
        } else if max_y.is_some() {
            *text = Text::new(format!(
                "Mesher max_y Skip: {}",
                if dev.max_y_skip { "ON" } else { "OFF" }
            ));
        } else if budget.is_some() {
            *text = Text::new(format!(
                "Mesh Budget: {}",
                if dev.mesh_budget { "ON" } else { "OFF" }
            ));
        } else if async_m.is_some() {
            *text = Text::new(format!(
                "Async Meshing: {}",
                if dev.async_meshing { "ON" } else { "OFF" }
            ));
        } else if greedy.is_some() {
            *text = Text::new(format!(
                "Greedy Meshing: {}",
                if dev.greedy_meshing { "ON" } else { "OFF" }
            ));
        } else if lod.is_some() {
            *text = Text::new(format!(
                "Distance LOD: {}",
                if dev.distance_lod { "ON" } else { "OFF" }
            ));
        } else if margin.is_some() {
            *text = Text::new(if dev.pregen_margin == 0 {
                "Lookahead Buffer: 0 (Disabled / Stutter prone)".to_string()
            } else {
                format!(
                    "Lookahead Buffer: {} Chunks (+{}m RAM cache)",
                    dev.pregen_margin,
                    dev.pregen_margin * 16
                )
            });
        } else if hud.is_some() {
            *text = Text::new(format!(
                "Dev HUD (F3): {}",
                if dev.show_debug_hud { "ON" } else { "OFF" }
            ));
        }
    }
}

pub fn update_dev_settings_system(
    mut commands: Commands,
    graphics_settings: Option<Res<GraphicsSettings>>,
    dev_settings: Option<Res<DevSettings>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<crate::world::WorldGrid>,
    mut dir_lights: Query<&mut DirectionalLight>,
    mut camera_query: Query<
        (Entity, Option<&mut bevy::pbr::DistanceFog>),
        With<crate::camera::FpsCamera>,
    >,
    mut last_config: Local<Option<(bool, i32, bool, i32, bool, i32)>>,
) {
    let (
        greedy_meshing,
        greedy_threshold,
        distance_lod,
        lod_threshold,
        distance_fog,
        view_distance,
    ) = graphics_settings.as_ref().map_or_else(
        || {
            dev_settings
                .as_ref()
                .map_or((true, 2, true, 8, true, 16), |d| {
                    (
                        d.greedy_meshing,
                        2,
                        d.distance_lod,
                        d.lod_threshold,
                        d.distance_fog,
                        16,
                    )
                })
        },
        |g| {
            (
                g.greedy_meshing,
                g.greedy_threshold,
                g.distance_lod,
                g.lod_threshold,
                g.distance_fog,
                g.view_distance,
            )
        },
    );
    let current_config = (
        greedy_meshing,
        greedy_threshold,
        distance_lod,
        lod_threshold,
        distance_fog,
        view_distance,
    );

    // 1. If greedy meshing or LOD settings changed, clear tracked tiers and re-queue all loaded chunks for re-meshing
    if last_config.map_or(false, |last| {
        last.0 != current_config.0
            || last.1 != current_config.1
            || last.2 != current_config.2
            || last.3 != current_config.3
    }) {
        world.chunk_lod.clear();
        let coords: Vec<_> = world.chunks.keys().copied().collect();
        for coord in coords {
            world.queue_mesh(coord);
        }
    }

    // 2. If fog or view_distance changed, update Distance Fog in real-time
    if last_config.map_or(true, |last| {
        last.4 != current_config.4 || last.5 != current_config.5
    }) {
        let max_dist = (view_distance as f32 * 16.0).max(64.0);
        let fog_start = (max_dist * 0.70).max(48.0);
        let fog_end = (max_dist - 2.0).max(64.0);

        for (cam_entity, mut maybe_fog) in &mut camera_query {
            if distance_fog {
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

    *last_config = Some(current_config);

    let Some(dev) = dev_settings else {
        return;
    };
    if dev.is_changed() {
        // 3. Update Backface Culling in real-time across ALL chunks
        if let Some(ref mat_handle) = world.block_material {
            if let Some(mut mat) = materials.get_mut(mat_handle) {
                mat.cull_mode = if dev.backface_culling {
                    Some(bevy::render::render_resource::Face::Back)
                } else {
                    None
                };
            }
        }

        // 4. Update Directional Light Shadows in real-time
        for mut light in &mut dir_lights {
            light.shadow_maps_enabled = dev.shadows_enabled;
        }
    }
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
                        "- All MineRust optimizations are tuned for maximum 60+ FPS stability.",
                    );
                }
            }
            for mut border in &mut card_query {
                *border = BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8));
            }
        }
    }
}

pub fn fps_limiter_system(settings: Res<GraphicsSettings>, mut limiter: ResMut<FpsLimiter>) {
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

    for (mut node, fps_f, fps_t, dist_f, dist_t, g_fill, l_fill, g_thumb, l_thumb) in &mut query {
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
        } else if g_fill.is_some() {
            let fill_pct = if greedy_ratio <= 0.0 {
                0.0
            } else {
                (greedy_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if l_fill.is_some() {
            let fill_pct = if lod_ratio <= 0.0 {
                0.0
            } else {
                (lod_ratio * 95.0 + 5.0).min(100.0)
            };
            node.width = Val::Percent(fill_pct);
        } else if g_thumb.is_some() {
            node.left = Val::Percent(greedy_ratio * 95.0);
        } else if l_thumb.is_some() {
            node.left = Val::Percent(lod_ratio * 95.0);
        }
    }
}

pub fn auto_save_graphics_settings_system(settings: Res<GraphicsSettings>) {
    if settings.is_changed() {
        settings.save_to_disk();
    }
}
