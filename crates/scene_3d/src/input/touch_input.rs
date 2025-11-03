use super::gesture_recognizer::{GestureState, GestureType};
use super::navigation_command::{NavigationCommand, NavigationState};
use bevy::input::touch::Touches;
use bevy::prelude::*;

pub fn touch_navigation_system(
    touches: Res<Touches>,
    mut gesture_state: ResMut<GestureState>,
    mut nav_state: ResMut<NavigationState>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Update gesture recognizer
    if let Some(gesture) = gesture_state.update(&touches, current_time) {
        match gesture.gesture_type {
            GestureType::SingleFingerDrag => {
                // One finger = orbit
                nav_state.add_command(NavigationCommand::Orbit {
                    delta: gesture.delta,
                });
            }

            GestureType::TwoFingerPan => {
                // Two finger drag = pan
                nav_state.add_command(NavigationCommand::Pan {
                    delta: gesture.delta,
                });
            }

            GestureType::Pinch => {
                if let Some(pinch_delta) = gesture.pinch_delta {
                    // Pinch = zoom toward gesture center
                    let zoom_factor = 1.0 - (pinch_delta * 0.01);

                    nav_state.add_command(NavigationCommand::ZoomToPoint {
                        factor: zoom_factor,
                        screen_point: gesture.position,
                    });
                }
            }

            GestureType::DoubleTap => {
                // Double tap = reset camera
                nav_state.add_command(NavigationCommand::Reset);
            }

            GestureType::Tap => {
                // Single tap = could select object
                // TODO: Implement object selection
            }

            GestureType::None => {}
        }
    }
}
