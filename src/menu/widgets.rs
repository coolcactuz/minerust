use bevy::prelude::*;
use bevy::text::FontSize;

use super::types::{
    MenuButtonAction, OptionTooltipCard, OptionTooltipDesc, OptionTooltipHeader,
    OptionTooltipImpact, OptionTooltipTitle, SliderTrack,
};

pub fn spawn_option_tooltip_card(parent: &mut ChildSpawnerCommands, width_px: f32, height_px: f32) {
    parent
        .spawn((
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.09, 0.15, 0.95)),
            BorderColor::all(Color::srgba(0.35, 0.55, 0.85, 0.8)),
            OptionTooltipCard,
        ))
        .with_children(|card| {
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|top| {
                top.spawn((
                    Text::new("[ SETTING INFO ]"),
                    TextFont {
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.35, 0.8, 1.0)),
                    OptionTooltipHeader,
                ));

                top.spawn((
                    Text::new("Hover over any setting"),
                    TextFont {
                        font_size: FontSize::Px(16.5),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                    OptionTooltipTitle,
                ));

                top.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(1.5),
                        margin: UiRect::axes(Val::Px(0.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.4, 0.5, 0.7, 0.35)),
                ));

                top.spawn((
                    Text::new(
                        "Move your mouse over any graphic or performance setting on the left to inspect its technical details, rendering behavior, and performance impact.",
                    ),
                    TextFont {
                        font_size: FontSize::Px(12.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.88, 0.93)),
                    OptionTooltipDesc,
                ));
            });

            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.16, 0.25, 0.85)),
                BorderColor::all(Color::srgba(0.3, 0.5, 0.7, 0.5)),
            ))
            .with_children(|impact_box| {
                impact_box.spawn((
                    Text::new("- All MineRust optimizations are tuned for maximum 60+ FPS stability."),
                    TextFont {
                        font_size: FontSize::Px(11.5),
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.95, 0.65)),
                    OptionTooltipImpact,
                ));
            });
        });
}

pub fn spawn_menu_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    highlight: bool,
) {
    spawn_menu_button_sized(parent, label, action, highlight, 320.0, 50.0, 16.0);
}

pub fn spawn_menu_button_sized(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    highlight: bool,
    width_px: f32,
    height_px: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(if highlight { 2.5 } else { 1.5 })),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
            BorderColor::all(if highlight {
                Color::srgb(1.0, 0.85, 0.2)
            } else {
                Color::srgba(0.45, 0.45, 0.55, 0.8)
            }),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn spawn_settings_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuButtonAction,
    text_marker: T,
    width_px: f32,
    height_px: f32,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.5)),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.16, 0.16, 0.22, 0.9)),
            BorderColor::all(Color::srgba(0.45, 0.45, 0.55, 0.8)),
            action,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
                text_marker,
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_slider_setting<
    TText: Component,
    TTrack: Component,
    TFill: Component,
    TThumb: Component,
>(
    parent: &mut ChildSpawnerCommands,
    text_marker: TText,
    track_marker: TTrack,
    fill_marker: TFill,
    thumb_marker: TThumb,
    left_action: MenuButtonAction,
    right_action: MenuButtonAction,
    track_action: MenuButtonAction,
    label_action: MenuButtonAction,
    label: &str,
    ratio: f32,
    width_px: f32,
    height_px: f32,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.13, 0.14, 0.20, 0.95)),
            BorderColor::all(Color::srgba(0.40, 0.45, 0.55, 0.8)),
        ))
        .with_children(|container| {
            // Label row (acting as a clickable button to cycle and show tooltip on hover)
            container
                .spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(16.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(Color::NONE),
                    label_action,
                ))
                .with_children(|title_btn| {
                    title_btn.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: FontSize::Px(12.5),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.95, 1.0)),
                        text_marker,
                    ));
                });

            // Slider controls row: [<] [===========|===========] [>]
            container
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(20.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    // Left stepper button [<]
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(18.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.20, 0.22, 0.30, 0.9)),
                        BorderColor::all(Color::srgba(0.45, 0.50, 0.65, 0.8)),
                        left_action,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("<"),
                            TextFont {
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });

                    // Interactive track with RelativeCursorPosition and SliderTrack
                    row.spawn((
                        Button,
                        bevy::ui::RelativeCursorPosition::default(),
                        SliderTrack,
                        track_marker,
                        track_action,
                        Node {
                            flex_grow: 1.0,
                            height: Val::Px(12.0),
                            position_type: PositionType::Relative,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.06, 0.07, 0.11, 0.98)),
                        BorderColor::all(Color::srgba(0.35, 0.40, 0.55, 0.7)),
                    ))
                    .with_children(|track| {
                        let fill_pct = if ratio <= 0.0 {
                            0.0
                        } else {
                            (ratio * 95.0 + 5.0).min(100.0)
                        };

                        // Fill bar
                        track.spawn((
                            fill_marker,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Percent(fill_pct),
                                height: Val::Percent(100.0),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.25, 0.60, 0.90, 0.70)),
                        ));

                        // Cursor knob (thumb)
                        track.spawn((
                            thumb_marker,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(ratio * 95.0),
                                width: Val::Px(10.0),
                                height: Val::Px(16.0),
                                border: UiRect::all(Val::Px(1.5)),
                                border_radius: BorderRadius::all(Val::Px(2.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.95, 0.96, 1.0)),
                            BorderColor::all(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });

                    // Right stepper button [>]
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(18.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.20, 0.22, 0.30, 0.9)),
                        BorderColor::all(Color::srgba(0.45, 0.50, 0.65, 0.8)),
                        right_action,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(">"),
                            TextFont {
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
        });
}
