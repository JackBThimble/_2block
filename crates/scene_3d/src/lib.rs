use bevy::prelude::*;

mod camera;
mod components;
mod coordinate_conversion;
mod crane_renderer;
mod input;
mod load_renderer;
mod resources;
mod setup;
mod sling_renderer;
mod test_scene;

pub use components::*;
pub use coordinate_conversion::*;
pub use crane_renderer::*;
pub use load_renderer::*;
pub use resources::*;
pub use setup::*;
pub use sling_renderer::*;
pub use test_scene::*;

pub struct Scene3DPlugin;

impl Plugin for Scene3DPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneState>()
            .init_resource::<CameraController>()
            .init_resource::<InteractionState>();
        app.add_plugins(input::Scene3dInputPlugin)
            .add_systems(
                Startup,
                (
                    setup::setup_scene,
                    test_scene::spawn_test_scene,
                    camera::camera_controller::setup_orbit_camera,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    camera::camera_controller::update_camera_transform,
                    camera::camera_controller::apply_camera_momentum,
                    camera::camera_controller::camera_preset_views,
                    // Crane updates
                    crane_renderer::update_crane_visuals_system,
                    // Load updates
                    load_renderer::update_load_visual_system,
                    load_renderer::highlight_selected_loads_system,
                    load_renderer::highlight_pick_points_system,
                    // Sling updates
                    sling_renderer::update_sling_colors,
                    sling_renderer::update_sling_geometry_system,
                ),
            );
    }
}
