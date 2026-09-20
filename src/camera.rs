use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(Component)]
pub struct FpsCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.3,
            speed: 12.0,
            sensitivity: 0.0025,
        }
    }
}

pub fn camera_look_system(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    mut query: Query<(&mut FpsCamera, &mut Transform)>,
) {
    let Ok(cursor) = cursor_options.single() else {
        return;
    };

    // Solo se il cursore è bloccato (locked) muoviamo la visuale
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

pub fn camera_move_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FpsCamera, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (fps, mut transform) in &mut query {
        let mut forward = Quat::from_rotation_y(fps.yaw) * -Vec3::Z;
        forward.y = 0.0;
        forward = forward.normalize_or_zero();

        let mut right = Quat::from_rotation_y(fps.yaw) * Vec3::X;
        right.y = 0.0;
        right = right.normalize_or_zero();

        let mut move_dir = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            move_dir += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            move_dir -= forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            move_dir -= right;
        }
        if keys.pressed(KeyCode::KeyD) {
            move_dir += right;
        }
        if keys.pressed(KeyCode::Space) {
            move_dir += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            move_dir -= Vec3::Y;
        }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        let speed_multiplier = if keys.pressed(KeyCode::ControlLeft) {
            2.5
        } else {
            1.0
        };

        transform.translation += move_dir * fps.speed * speed_multiplier * dt;
    }
}

pub fn cursor_grab_system(
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
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
