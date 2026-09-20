use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::Inventory;
use crate::menu::MenuState;

#[derive(Component)]
pub struct FpsCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.3,
            sensitivity: 0.0025,
        }
    }
}

pub fn camera_look_system(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    inventory: Option<Res<Inventory>>,
    menu: Option<Res<MenuState>>,
    mut query: Query<(&mut FpsCamera, &mut Transform)>,
) {
    if let Some(menu) = menu {
        if menu.is_open() {
            return;
        }
    }
    if let Some(inv) = inventory {
        if inv.is_open {
            return;
        }
    }

    let Ok(cursor) = cursor_options.single() else {
        return;
    };

    // Only rotate the camera when the cursor is locked
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let delta = accumulated_mouse_motion.delta;
    if delta.length_squared() == 0.0 {
        return;
    }

    for (mut fps, mut transform) in &mut query {
        fps.yaw -= delta.x * fps.sensitivity;
        fps.pitch -= delta.y * fps.sensitivity;

        let max_pitch = 89.0_f32.to_radians();
        fps.pitch = fps.pitch.clamp(-max_pitch, max_pitch);

        transform.rotation =
            Quat::from_rotation_y(fps.yaw) * Quat::from_rotation_x(fps.pitch);
    }
}


pub fn cursor_grab_system(
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    inventory: Option<Res<Inventory>>,
    menu: Option<Res<MenuState>>,
) {
    if let Some(menu) = menu {
        if menu.is_open() {
            return;
        }
    }
    if let Some(inv) = inventory {
        if inv.is_open {
            return;
        }
    }

    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Left)
        && cursor.grab_mode != CursorGrabMode::Locked
    {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }

    if keys.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}
