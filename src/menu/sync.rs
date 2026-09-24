use bevy::prelude::*;

use super::descriptions::get_option_description;
use super::types::{
    DebugHudBtnText, DistanceFogBtnText, FpsCapBtnText, FpsCapFill, FpsCapThumb, FpsLimiter,
    FullscreenBtnText, GraphicsSettings, MenuButtonAction, OptionTooltipCard, OptionTooltipDesc,
    OptionTooltipHeader, OptionTooltipImpact, OptionTooltipTitle, ProfilerState, ShadowsBtnText,
    ViewDistanceBtnText, ViewDistanceFill, ViewDistanceThumb, VsyncBtnText,
};

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
    )>,
) {
    let hud_visible = profiler_state.as_ref().is_some_and(|p| p.visible);

    for (mut text, vsync, fs, shadows, fog, hud, fps, dist) in &mut query {
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
    )>,
    mut last_ratios: Local<Option<(f32, f32)>>,
) {
    let fps_ratio = settings.fps_cap_ratio();
    let dist_ratio = settings.view_distance_ratio();
    let current_ratios = (fps_ratio, dist_ratio);

    if last_ratios.is_some_and(|r| r == current_ratios) {
        return;
    }
    *last_ratios = Some(current_ratios);

    for (mut node, fps_f, fps_t, dist_f, dist_t) in &mut query {
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
        }
    }
}

pub fn auto_save_graphics_settings_system(settings: Res<GraphicsSettings>) {
    if settings.is_changed() {
        settings.save_to_disk();
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
