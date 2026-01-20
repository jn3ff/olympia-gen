//! Movement domain: input sampling for locomotion.

use bevy::prelude::*;

use crate::movement::MovementInput;

pub(crate) fn read_input(keyboard: Res<ButtonInput<KeyCode>>, mut input: ResMut<MovementInput>) {
    // Horizontal axis
    let mut x = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        x += 1.0;
    }

    // Vertical axis (for wall cling direction, etc.)
    let mut y = 0.0;
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        y += 1.0;
    }

    input.axis = Vec2::new(x, y);
    input.jump_just_pressed =
        keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyK);
    input.jump_held = keyboard.pressed(KeyCode::Space) || keyboard.pressed(KeyCode::KeyK);
    input.dash_just_pressed =
        keyboard.just_pressed(KeyCode::ShiftLeft) || keyboard.just_pressed(KeyCode::KeyJ);
}
