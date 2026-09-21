use bevy::prelude::*;

use super::types::{MenuScreen, MenuState, SeedInputBox, SeedInputState, SeedInputText};

/// Converts keyboard KeyCode inputs to characters, supporting Shift modifier.
pub fn keycode_to_char(key: KeyCode, shift: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Numpad0 => Some('0'),
        KeyCode::Numpad1 => Some('1'),
        KeyCode::Numpad2 => Some('2'),
        KeyCode::Numpad3 => Some('3'),
        KeyCode::Numpad4 => Some('4'),
        KeyCode::Numpad5 => Some('5'),
        KeyCode::Numpad6 => Some('6'),
        KeyCode::Numpad7 => Some('7'),
        KeyCode::Numpad8 => Some('8'),
        KeyCode::Numpad9 => Some('9'),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

pub fn update_seed_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<MenuState>,
    mut seed_state: Option<ResMut<SeedInputState>>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut box_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let Some(ref mut state) = seed_state else {
        return;
    };

    if menu.screen != MenuScreen::Main {
        if state.is_editing {
            state.is_editing = false;
        }
        return;
    }

    let mut state_changed = false;

    if state.is_editing {
        if keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || keys.just_pressed(KeyCode::Escape)
        {
            state.is_editing = false;
            state_changed = true;
        } else if keys.just_pressed(KeyCode::Backspace) {
            state.seed_text.pop();
            state_changed = true;
        } else {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            for &key in keys.get_just_pressed() {
                if let Some(ch) = keycode_to_char(key, shift) {
                    if state.seed_text.len() < 32 {
                        state.seed_text.push(ch);
                        state_changed = true;
                    }
                }
            }
        }
    }

    if state_changed || state.is_changed() {
        if let Ok(mut text) = text_query.single_mut() {
            *text = Text::new(if state.is_editing {
                if state.seed_text.is_empty() {
                    "Seed: |".to_string()
                } else {
                    format!("Seed: {}|", state.seed_text)
                }
            } else if state.seed_text.is_empty() {
                "Seed: [Random Seed]".to_string()
            } else {
                format!("Seed: {}", state.seed_text)
            });
        }

        if let Ok(mut border) = box_query.single_mut() {
            *border = if state.is_editing {
                BorderColor::all(Color::srgb(0.2, 0.8, 1.0))
            } else {
                BorderColor::all(Color::srgba(0.35, 0.4, 0.55, 0.8))
            };
        }
    }
}
