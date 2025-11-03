use super::navigation_command::{NavigationCommand, NavigationState};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[derive(Default)]
pub struct MouseState {
    is_orbiting: bool,
    is_panning: bool,
    is_zooming: bool,
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
    let window = windows.single().ok();

    let left_pressed = mouse_button.pressed(MouseButton::Left);
    let middle_pressed = mouse_button.pressed(MouseButton::Middle);
    let right_pressed = mouse_button.pressed(MouseButton::Right);

    // Check modifier keys
    let shift_pressed =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let ctrl_pressed =
        keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let alt_pressed = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    mouse_state.is_orbiting =
        left_pressed && !shift_pressed && !ctrl_pressed && !middle_pressed && !right_pressed;
    mouse_state.is_panning = middle_pressed || (left_pressed && shift_pressed);
    mouse_state.is_zooming = right_pressed || (left_pressed && ctrl_pressed);

    // ========================================================================
    // MOUSE MOTION (Orbit / Pan / Zoom)
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
        } else if mouse_state.is_zooming {
            let zoom_delta = -total_delta.y * 0.01; // 
            let zoom_factor = 1.0 - zoom_delta;

            nav_state.add_command(NavigationCommand::Zoom {
                factor: zoom_factor,
            });
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

            if let Some(window) = window {
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
