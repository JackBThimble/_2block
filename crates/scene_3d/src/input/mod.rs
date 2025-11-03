pub mod gesture_recognizer;
pub mod mouse_input;
pub mod navigation_command;
pub mod touch_input;

use bevy::prelude::*;

pub struct Scene3dInputPlugin;

impl Plugin for Scene3dInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<gesture_recognizer::GestureState>()
            .init_resource::<navigation_command::NavigationState>()
            .add_systems(
                Update,
                (
                    (
                        // Input detection systems run first
                        mouse_input::mouse_navigation_system,
                        touch_input::touch_navigation_system,
                    ),
                    // Then we apply navigation commands to camera
                    navigation_command::apply_navigation_commands,
                )
                    .chain(),
            );
    }
}
