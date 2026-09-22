use bevy::prelude::*;

use super::descriptions::get_option_description;
use super::types::{
    AsyncMeshingBtnText, BackfaceCullingBtnText, DebugHudBtnText, DevSettings,
    DistanceFogBtnText, DistanceLodBtnText, FpsCapBtnText, FpsLimiter, FullscreenBtnText,
    GraphicsGreedyBtnText, GraphicsGreedyFill, GraphicsGreedyThumb, GraphicsLodBtnText,
    GraphicsLodFill, GraphicsLodThumb, GraphicsSettings, GreedyMeshingBtnText,
    LodThresholdBtnText, MaxYSkipBtnText, MenuButtonAction, MeshBudgetBtnText, OptionTooltipCard,
    OptionTooltipDesc, OptionTooltipHeader, OptionTooltipImpact, OptionTooltipTitle,
    PregenMarginBtnText, ShadowsBtnText, ViewDistanceBtnText, VsyncBtnText,
};

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    mut query: Query<(
        &mut Text,
        Option<&VsyncBtnText>,
        Option<&FullscreenBtnText>,
        Option<&FpsCapBtnText>,
        Option<&ViewDistanceBtnText>,
        Option<&GraphicsGreedyBtnText>,
        Option<&GraphicsLodBtnText>,
    )>,
) {
    if !settings.is_changed() {
        return;
    }
    for (mut text, vsync, fs, fps, dist, greedy, lod) in &mut query {
        if vsync.is_some() {
            *text = Text::new(format!(
                "VSync: {}",
                if settings.vsync {
                    "ON (Smooth)"
                } else {
                    "OFF (Uncapped)"
                }
            ));
        } else if fs.is_some() {
            *text = Text::new(format!(
                "Display: {}",
                if settings.fullscreen {
                    "Fullscreen"
                } else {
                    "Windowed (1280x720)"
                }
            ));
        } else if fps.is_some() {
            *text = Text::new(format!(
                "FPS Limit: {}",
                match settings.fps_cap {
                    None => "Uncapped".to_string(),
                    Some(cap) => format!("{} FPS", cap),
                }
            ));
        } else if dist.is_some() {
            *text = Text::new(format!(
                "Render Distance: {} Chunks",
                settings.view_distance
            ));
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
        Option<&DistanceFogBtnText>,
        Option<&MeshBudgetBtnText>,
        Option<&AsyncMeshingBtnText>,
        Option<&GreedyMeshingBtnText>,
        Option<&DistanceLodBtnText>,
        Option<&LodThresholdBtnText>,
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
    for (mut text, cull, shadow, max_y, fog, budget, async_m, greedy, lod, thresh, hud, margin) in
        &mut query
    {
        if cull.is_some() {
            *text = Text::new(format!(
                "Backface Culling: {}",
                if dev.backface_culling {
                    "ON (GPU -50%)"
                } else {
                    "OFF (Draw front & back)"
                }
            ));
        } else if shadow.is_some() {
            *text = Text::new(format!(
                "Dynamic Shadows: {}",
                if dev.shadows_enabled {
                    "ON (120m Cascades)"
                } else {
                    "OFF (Zero shadow passes)"
                }
            ));
        } else if max_y.is_some() {
            *text = Text::new(format!(
                "Mesher max_y Skip: {}",
                if dev.max_y_skip {
                    "ON (2x faster meshing)"
                } else {
                    "OFF (Loop all 384 layers)"
                }
            ));
        } else if fog.is_some() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if dev.distance_fog {
                    "ON (Blended horizon)"
                } else {
                    "OFF (Harsh edge)"
                }
            ));
        } else if budget.is_some() {
            *text = Text::new(format!(
                "Mesh Budget: {}",
                if dev.mesh_budget {
                    "ON (6/frame smooth)"
                } else {
                    "OFF (Spike benchmark)"
                }
            ));
        } else if async_m.is_some() {
            *text = Text::new(format!(
                "Async Meshing: {}",
                if dev.async_meshing {
                    "ON (0ms main thread)"
                } else {
                    "OFF (Sync frame spikes)"
                }
            ));
        } else if greedy.is_some() {
            *text = Text::new(format!(
                "Greedy Meshing: {}",
                if dev.greedy_meshing {
                    "ON (-75% verts)"
                } else {
                    "OFF (1x1 block quads)"
                }
            ));
        } else if lod.is_some() {
            *text = Text::new(format!(
                "Distance LOD: {}",
                if dev.distance_lod {
                    "ON (Dynamic detail)"
                } else {
                    "OFF (Uniform meshing)"
                }
            ));
        } else if thresh.is_some() {
            *text = Text::new(format!(
                "LOD Distance: {} Chunks ({}m)",
                dev.lod_threshold,
                dev.lod_threshold * 16
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
                if dev.show_debug_hud {
                    "ON (Visible)"
                } else {
                    "OFF (Hidden)"
                }
            ));
        }
    }
}

pub fn update_dev_settings_system(
    graphics_settings: Option<Res<GraphicsSettings>>,
    dev_settings: Option<Res<DevSettings>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<crate::world::WorldGrid>,
    mut dir_lights: Query<&mut DirectionalLight>,
    mut fog_query: Query<&mut bevy::pbr::DistanceFog>,
    mut last_config: Local<Option<(bool, i32, bool, i32)>>,
) {
    let (greedy_meshing, greedy_threshold, distance_lod, lod_threshold) =
        graphics_settings.as_ref().map_or_else(
            || {
                dev_settings.as_ref().map_or(
                    (true, 2, true, 8),
                    |d| (d.greedy_meshing, 2, d.distance_lod, d.lod_threshold),
                )
            },
            |g| (g.greedy_meshing, g.greedy_threshold, g.distance_lod, g.lod_threshold),
        );
    let current_config = (greedy_meshing, greedy_threshold, distance_lod, lod_threshold);

    // 1. If greedy meshing or LOD settings changed, clear tracked tiers and re-queue all loaded chunks for re-meshing
    if last_config.map_or(false, |last| last != current_config) {
        world.chunk_lod.clear();
        let coords: Vec<_> = world.chunks.keys().copied().collect();
        for coord in coords {
            world.queue_mesh(coord);
        }
    }
    *last_config = Some(current_config);

    let Some(dev) = dev_settings else {
        return;
    };
    if dev.is_changed() {
        // 2. Update Backface Culling in real-time across ALL chunks
        if let Some(ref mat_handle) = world.block_material {
            if let Some(mut mat) = materials.get_mut(mat_handle) {
                mat.cull_mode = if dev.backface_culling {
                    Some(bevy::render::render_resource::Face::Back)
                } else {
                    None
                };
            }
        }

        // 3. Update Directional Light Shadows in real-time
        for mut light in &mut dir_lights {
            light.shadow_maps_enabled = dev.shadows_enabled;
        }

        // 4. Update Distance Fog in real-time
        for mut fog in &mut fog_query {
            if dev.distance_fog {
                fog.falloff = bevy::pbr::FogFalloff::Linear {
                    start: 180.0,
                    end: 255.0,
                };
            } else {
                fog.falloff = bevy::pbr::FogFalloff::Linear {
                    start: 99999.0,
                    end: 100000.0,
                };
            }
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
        Option<&GraphicsGreedyFill>,
        Option<&GraphicsLodFill>,
        Option<&GraphicsGreedyThumb>,
        Option<&GraphicsLodThumb>,
    )>,
    mut last_ratios: Local<Option<(f32, f32)>>,
) {
    let greedy_ratio = settings.greedy_ratio();
    let lod_ratio = settings.lod_ratio();
    let current_ratios = (greedy_ratio, lod_ratio);

    if last_ratios.is_some_and(|r| r == current_ratios) {
        return;
    }
    *last_ratios = Some(current_ratios);

    for (mut node, g_fill, l_fill, g_thumb, l_thumb) in &mut query {
        if g_fill.is_some() {
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

