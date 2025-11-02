//! Mouse and trackpad input handling
//! Supports: drag to orbit, middle-drag to pan, scroll to zoom, modifier keys

use super::navigation_command::{NavigationCommand, NavigationState};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[derive(Default)]
pub struct MouseState {
    is_orbiting: bool,
    is_panning: bool,
    _last_position: Vec2,
}

pub fn mouse_navigation_system(
    mut mouse_state: Local<MouseState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut nav_state: ResMut<NavigationState>,
    windows: Query<&Window>,
) {
    // Get window for cursor position
    let window = match windows.single() {
        Ok(window) => window,
        Err(_) => return,
    };

    // ========================================================================
    // MOUSE BUTTON STATE
    // ========================================================================

    let left_pressed = mouse_button.pressed(MouseButton::Left);
    let middle_pressed = mouse_button.pressed(MouseButton::Middle);
    let right_pressed = mouse_button.pressed(MouseButton::Right);

    // Check modifier keys
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let _ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let _alt_pressed = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    // Determine navigation mode based on buttons + modifiers
    // CAD-style: Left = orbit, Middle = pan, Shift+Left = pan
    let should_orbit = left_pressed && !shift_pressed && !middle_pressed;
    let should_pan = middle_pressed || (left_pressed && shift_pressed) || right_pressed;

    mouse_state.is_orbiting = should_orbit;
    mouse_state.is_panning = should_pan;

    // ========================================================================
    // MOUSE MOTION (Orbit / Pan)
    // ========================================================================

    let mut total_delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        total_delta += motion.delta;
    }

    if total_delta.length_squared() > 0.0 {
        if mouse_state.is_orbiting {
            nav_state.add_command(NavigationCommand::Orbit { delta: total_delta });
        } else if mouse_state.is_panning {
            nav_state.add_command(NavigationCommand::Pan { delta: total_delta });
        }
    }

    // ========================================================================
    // MOUSE WHEEL (Zoom)
    // ========================================================================

    for wheel in mouse_wheel.read() {
        let zoom_delta = match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 0.1,
            MouseScrollUnit::Pixel => wheel.y * 0.01,
        };

        if zoom_delta.abs() > 0.001 {
            // Zoom factor: positive = zoom in, negative = zoom out
            let zoom_factor = 1.0 - zoom_delta;

            // If we have cursor position, zoom toward cursor
            if let Some(cursor_pos) = window.cursor_position() {
                nav_state.add_command(NavigationCommand::ZoomToPoint {
                    factor: zoom_factor,
                    screen_point: cursor_pos,
                });
            } else {
                nav_state.add_command(NavigationCommand::Zoom {
                    factor: zoom_factor,
                });
            }
        }
    }

    // ========================================================================
    // KEYBOARD SHORTCUTS
    // ========================================================================

    if keyboard.just_pressed(KeyCode::Home) || keyboard.just_pressed(KeyCode::KeyH) {
        nav_state.add_command(NavigationCommand::Reset);
    }

    // F key = frame/focus on selection (implement selection later)
    if keyboard.just_pressed(KeyCode::KeyF) {
        // TODO: Focus on selected object
        // nav_state.add_command(NavigationCommand::FocusOn { world_point: selected_pos });
    }
}
