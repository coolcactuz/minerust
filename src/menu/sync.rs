use bevy::prelude::*;

use super::descriptions::get_option_description;
use super::types::{
    AsyncMeshingBtnText, BackfaceCullingBtnText, DebugHudBtnText, DevSettings,
    DistanceFogBtnText, DistanceLodBtnText, FpsCapBtnText, FpsLimiter, FullscreenBtnText,
    GraphicsGreedyBtnText, GraphicsLodBtnText, GraphicsSettings, GreedyMeshingBtnText,
    LodThresholdBtnText, MaxYSkipBtnText, MenuButtonAction, MeshBudgetBtnText, OptionTooltipCard,
    OptionTooltipDesc, OptionTooltipHeader, OptionTooltipImpact, OptionTooltipTitle,
    PregenMarginBtnText, ShadowsBtnText, ViewDistanceBtnText, VsyncBtnText,
};

pub fn update_settings_button_text_system(
    settings: Res<GraphicsSettings>,
    mut vsync_text_query: Query<&mut Text, With<VsyncBtnText>>,
    mut fs_text_query: Query<&mut Text, With<FullscreenBtnText>>,
    mut fps_text_query: Query<&mut Text, With<FpsCapBtnText>>,
    mut dist_text_query: Query<&mut Text, With<ViewDistanceBtnText>>,
    mut greedy_text_query: Query<&mut Text, With<GraphicsGreedyBtnText>>,
    mut lod_text_query: Query<&mut Text, With<GraphicsLodBtnText>>,
) {
    if settings.is_changed() {
        for mut text in &mut vsync_text_query {
            *text = Text::new(format!(
                "VSync: {}",
                if settings.vsync {
                    "ON (Smooth)"
                } else {
                    "OFF (Uncapped)"
                }
            ));
        }
        for mut text in &mut fs_text_query {
            *text = Text::new(format!(
                "Display: {}",
                if settings.fullscreen {
                    "Fullscreen"
                } else {
                    "Windowed (1280x720)"
                }
            ));
        }
        for mut text in &mut fps_text_query {
            *text = Text::new(format!(
                "FPS Limit: {}",
                match settings.fps_cap {
                    None => "Uncapped".to_string(),
                    Some(cap) => format!("{} FPS", cap),
                }
            ));
        }
        for mut text in &mut dist_text_query {
            *text = Text::new(format!(
                "Render Distance: {} Chunks",
                settings.view_distance
            ));
        }
        for mut text in &mut greedy_text_query {
            *text = Text::new(if settings.greedy_meshing {
                if settings.greedy_threshold == 0 {
                    "Greedy Distance: All Chunks (0m)".to_string()
                } else {
                    format!(
                        "Greedy Distance: > {} Chunks ({}m)",
                        settings.greedy_threshold,
                        settings.greedy_threshold * 16
                    )
                }
            } else {
                "Greedy Meshing: OFF (1x1 Voxels)".to_string()
            });
        }
        for mut text in &mut lod_text_query {
            *text = Text::new(if settings.distance_lod {
                format!(
                    "Distant Sloped LOD: > {} Chunks ({}m)",
                    settings.lod_threshold,
                    settings.lod_threshold * 16
                )
            } else {
                "Distant Sloped LOD: OFF (Blocky Only)".to_string()
            });
        }
    }
}

pub fn update_dev_button_text_system(
    dev_settings: Option<Res<DevSettings>>,
    mut cull_text_query: Query<
        &mut Text,
        (
            With<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut shadow_text_query: Query<
        &mut Text,
        (
            With<ShadowsBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut max_y_text_query: Query<
        &mut Text,
        (
            With<MaxYSkipBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut fog_text_query: Query<
        &mut Text,
        (
            With<DistanceFogBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut budget_text_query: Query<
        &mut Text,
        (
            With<MeshBudgetBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut async_text_query: Query<
        &mut Text,
        (
            With<AsyncMeshingBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut greedy_text_query: Query<
        &mut Text,
        (
            With<GreedyMeshingBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut lod_text_query: Query<
        &mut Text,
        (
            With<DistanceLodBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut thresh_text_query: Query<
        &mut Text,
        (
            With<LodThresholdBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<DebugHudBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut hud_text_query: Query<
        &mut Text,
        (
            With<DebugHudBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<PregenMarginBtnText>,
        ),
    >,
    mut margin_text_query: Query<
        &mut Text,
        (
            With<PregenMarginBtnText>,
            Without<BackfaceCullingBtnText>,
            Without<ShadowsBtnText>,
            Without<MaxYSkipBtnText>,
            Without<DistanceFogBtnText>,
            Without<MeshBudgetBtnText>,
            Without<AsyncMeshingBtnText>,
            Without<GreedyMeshingBtnText>,
            Without<DistanceLodBtnText>,
            Without<LodThresholdBtnText>,
            Without<DebugHudBtnText>,
        ),
    >,
) {
    let Some(dev) = dev_settings else {
        return;
    };
    if dev.is_changed() {
        if let Ok(mut text) = cull_text_query.single_mut() {
            *text = Text::new(format!(
                "Backface Culling: {}",
                if dev.backface_culling {
                    "ON (GPU -50%)"
                } else {
                    "OFF (Draw front & back)"
                }
            ));
        }
        if let Ok(mut text) = shadow_text_query.single_mut() {
            *text = Text::new(format!(
                "Dynamic Shadows: {}",
                if dev.shadows_enabled {
                    "ON (120m Cascades)"
                } else {
                    "OFF (Zero shadow passes)"
                }
            ));
        }
        if let Ok(mut text) = max_y_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesher max_y Skip: {}",
                if dev.max_y_skip {
                    "ON (2x faster meshing)"
                } else {
                    "OFF (Loop all 384 layers)"
                }
            ));
        }
        if let Ok(mut text) = fog_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance Fog: {}",
                if dev.distance_fog {
                    "ON (Blended horizon)"
                } else {
                    "OFF (Harsh edge)"
                }
            ));
        }
        if let Ok(mut text) = budget_text_query.single_mut() {
            *text = Text::new(format!(
                "Mesh Budget: {}",
                if dev.mesh_budget {
                    "ON (6/frame smooth)"
                } else {
                    "OFF (Spike benchmark)"
                }
            ));
        }
        if let Ok(mut text) = async_text_query.single_mut() {
            *text = Text::new(format!(
                "Async Meshing: {}",
                if dev.async_meshing {
                    "ON (0ms main thread)"
                } else {
                    "OFF (Sync frame spikes)"
                }
            ));
        }
        if let Ok(mut text) = greedy_text_query.single_mut() {
            *text = Text::new(format!(
                "Greedy Meshing: {}",
                if dev.greedy_meshing {
                    "ON (-75% verts)"
                } else {
                    "OFF (1x1 block quads)"
                }
            ));
        }
        if let Ok(mut text) = lod_text_query.single_mut() {
            *text = Text::new(format!(
                "Distance LOD: {}",
                if dev.distance_lod {
                    "ON (Dynamic detail)"
                } else {
                    "OFF (Uniform meshing)"
                }
            ));
        }
        if let Ok(mut text) = thresh_text_query.single_mut() {
            *text = Text::new(format!(
                "LOD Distance: {} Chunks ({}m)",
                dev.lod_threshold,
                dev.lod_threshold * 16
            ));
        }
        if let Ok(mut text) = margin_text_query.single_mut() {
            *text = Text::new(if dev.pregen_margin == 0 {
                "Lookahead Buffer: 0 (Disabled / Stutter prone)".to_string()
            } else {
                format!(
                    "Lookahead Buffer: {} Chunks (+{}m RAM cache)",
                    dev.pregen_margin,
                    dev.pregen_margin * 16
                )
            });
        }
        if let Ok(mut text) = hud_text_query.single_mut() {
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
    mut header_query: Query<
        &mut Text,
        (
            With<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipDesc>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut title_query: Query<
        &mut Text,
        (
            With<OptionTooltipTitle>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipDesc>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<OptionTooltipDesc>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipImpact>,
        ),
    >,
    mut impact_query: Query<
        &mut Text,
        (
            With<OptionTooltipImpact>,
            Without<OptionTooltipHeader>,
            Without<OptionTooltipTitle>,
            Without<OptionTooltipDesc>,
        ),
    >,
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
                for mut text in &mut header_query {
                    *text = Text::new(format!("[ {} ]", desc.header));
                }
                for mut text in &mut title_query {
                    *text = Text::new(desc.title);
                }
                for mut text in &mut desc_query {
                    *text = Text::new(desc.description);
                }
                for mut text in &mut impact_query {
                    *text = Text::new(desc.impact);
                }
                for mut border in &mut card_query {
                    *border = BorderColor::all(Color::srgb(1.0, 0.85, 0.2));
                }
            }
        } else {
            for mut text in &mut header_query {
                *text = Text::new("[ SETTING INFO ]");
            }
            for mut text in &mut title_query {
                *text = Text::new("Hover over any setting");
            }
            for mut text in &mut desc_query {
                *text = Text::new(
                    "Move your mouse over any graphic or performance setting on the left to inspect its technical details, rendering behavior, and performance impact.",
                );
            }
            for mut text in &mut impact_query {
                *text = Text::new(
                    "- All MineRust optimizations are tuned for maximum 60+ FPS stability.",
                );
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
