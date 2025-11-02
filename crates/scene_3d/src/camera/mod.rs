pub mod camera_controller;
pub mod orbit_camera;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_orbit_camera).add_systems(
            Update,
            (
                camera_controller::update_camera_transform,
                camera_controller::apply_camera_momentum, // NEW: Momentum system
                camera_controller::camera_preset_views,
            )
                .chain(), // Run in order
        );
    }
}

pub use camera_controller::setup_orbit_camera;
