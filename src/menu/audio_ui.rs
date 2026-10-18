#![allow(deprecated)]

use bevy::prelude::*;
use bevy::text::FontSize;

use super::types::{
    AudioSettingsRoot, GraphicsSettings, MasterVolumeBtnText, MasterVolumeFill, MasterVolumeThumb,
    MasterVolumeTrack, MenuButtonAction, SfxVolumeBtnText, SfxVolumeFill, SfxVolumeThumb,
    SfxVolumeTrack,
};
use super::widgets::{spawn_menu_button_sized, spawn_option_tooltip_card, spawn_slider_setting};

/// Spawns the dedicated Audio & Sound Settings menu screen.
pub fn spawn_audio_settings_menu(commands: &mut Commands, settings: Option<&GraphicsSettings>) {
    let (master_label, master_ratio) = settings.map_or_else(
        || ("Master Volume: 100%".to_string(), 1.0),
        |g| (g.master_volume_label(), g.master_volume_ratio()),
    );
    let (sfx_label, sfx_ratio) = settings.map_or_else(
        || ("Sound Effects: 100%".to_string(), 1.0),
        |g| (g.sfx_volume_label(), g.sfx_volume_ratio()),
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.88)),
            Visibility::Hidden,
            AudioSettingsRoot,
        ))
        .with_children(|parent| {
            // Header
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("AUDIO & SOUND SETTINGS"),
                        TextFont {
                            font_size: FontSize::Px(32.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    ));
                    header.spawn((
                        Text::new(
                            "Adjust master volume, sound effects, block breaking, and environmental audio",
                        ),
                        TextFont {
                            font_size: FontSize::Px(13.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.75, 0.85)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: Val::Px(24.0),
                    ..default()
                })
                .with_children(|row| {
                    // Controls Column
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|btn_col| {
                        // Master Volume Slider
                        spawn_slider_setting(
                            btn_col,
                            MasterVolumeBtnText,
                            MasterVolumeTrack,
                            MasterVolumeFill,
                            MasterVolumeThumb,
                            MenuButtonAction::StepMasterVolumeLeft,
                            MenuButtonAction::StepMasterVolumeRight,
                            MenuButtonAction::SlideMasterVolume,
                            MenuButtonAction::StepMasterVolumeRight,
                            &master_label,
                            master_ratio,
                            320.0,
                            46.0,
                        );

                        // Sound Effects Volume Slider
                        spawn_slider_setting(
                            btn_col,
                            SfxVolumeBtnText,
                            SfxVolumeTrack,
                            SfxVolumeFill,
                            SfxVolumeThumb,
                            MenuButtonAction::StepSfxVolumeLeft,
                            MenuButtonAction::StepSfxVolumeRight,
                            MenuButtonAction::SlideSfxVolume,
                            MenuButtonAction::StepSfxVolumeRight,
                            &sfx_label,
                            sfx_ratio,
                            320.0,
                            46.0,
                        );

                        // Test Audio Sample Button
                        spawn_menu_button_sized(
                            btn_col,
                            "🎵 Test Audio Sample",
                            MenuButtonAction::TestAudioSample,
                            true,
                            320.0,
                            38.0,
                            14.0,
                        );


                        // Back / Done Button
                        spawn_menu_button_sized(
                            btn_col,
                            "Back / Done",
                            MenuButtonAction::BackFromAudioSettings,
                            false,
                            320.0,
                            36.0,
                            14.0,
                        );
                    });

                    // Right Info Tooltip Card
                    spawn_option_tooltip_card(row, 380.0, 320.0);
                });
        });
}
