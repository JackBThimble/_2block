use super::orbit_camera::OrbitCamera;
use crate::input::navigation_command::NavigationState;
use bevy::prelude::*;
use log::{info, warn};

/// System to update camera transform from orbit parameters
pub fn update_camera_transform(mut query: Query<(&OrbitCamera, &mut Transform), With<Camera>>) {
    for (orbit, mut transform) in query.iter_mut() {
        transform.translation = orbit.get_position();
        transform.look_at(orbit.target, orbit.get_up_vector());
    }
}

/// System to apply momentum when user is NOT actively navigating
/// This is what makes it feel SLICK AS FUCK
pub fn apply_camera_momentum(
    mut camera_query: Query<&mut OrbitCamera>,
    nav_state: Res<NavigationState>,
    time: Res<Time>,
) {
    // Only apply momentum when user isn't actively controlling camera
    if !nav_state.is_navigating {
        for mut camera in camera_query.iter_mut() {
            camera.apply_momentum(time.delta_secs());
        }
    } else {
        // User is actively navigating - stop momentum to prevent fighting
        for mut camera in camera_query.iter_mut() {
            // Don't completely zero it out, just reduce it
            // This allows momentum to "resume" smoothly when they let go
            camera.orbit_velocity *= 0.5;
            camera.pan_velocity *= 0.5;
            camera.zoom_velocity *= 0.5;
        }
    }
}

/// Preset view system - integrates with your existing keyboard shortcuts
pub fn camera_preset_views(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    if let Ok(mut camera) = camera_query.single_mut() {
        // Numpad views (like Blender/3DS Max)
        if keyboard.just_pressed(KeyCode::Numpad7) {
            camera.set_preset_view(super::orbit_camera::PresetView::Top);
        }
        if keyboard.just_pressed(KeyCode::Numpad1) {
            camera.set_preset_view(super::orbit_camera::PresetView::Front);
        }
        if keyboard.just_pressed(KeyCode::Numpad3) {
            camera.set_preset_view(super::orbit_camera::PresetView::Right);
        }
        if keyboard.just_pressed(KeyCode::Numpad9) {
            camera.set_preset_view(super::orbit_camera::PresetView::Back);
        }

        // Also support regular number keys
        if keyboard.just_pressed(KeyCode::Digit7) {
            camera.set_preset_view(super::orbit_camera::PresetView::Top);
        }
        if keyboard.just_pressed(KeyCode::Digit1) {
            camera.set_preset_view(super::orbit_camera::PresetView::Front);
        }
        if keyboard.just_pressed(KeyCode::Digit3) {
            camera.set_preset_view(super::orbit_camera::PresetView::Right);
        }

        // Home key resets view
        if keyboard.just_pressed(KeyCode::Home) || keyboard.just_pressed(KeyCode::KeyH) {
            camera.reset();
        }

        // C key to toggle clamp mode (for testing/advanced users)
        if keyboard.just_pressed(KeyCode::KeyC) {
            use super::orbit_camera::ClampMode;
            camera.clamp_mode = match camera.clamp_mode {
                ClampMode::Restricted => {
                    info!("Camera: Relaxed mode (smooth over-top rotation)");
                    ClampMode::Relaxed
                }
                ClampMode::Relaxed => {
                    info!("Camera: Free mode (full 360° tumble)");
                    ClampMode::Free
                }
                ClampMode::Free => {
                    info!("Camera: Restricted mode (no upside-down)");
                    ClampMode::Restricted
                }
            };
        }
    }
}

/// Setup system to spawn the orbit camera
pub fn setup_orbit_camera(mut commands: Commands, existing_cameras: Query<Entity, With<Camera>>) {
    // If there's already a camera, add OrbitCamera to it
    if let Ok(camera_entity) = existing_cameras.single() {
        commands
            .entity(camera_entity)
            .insert(OrbitCamera::default());
        info!("Added OrbitCamera to existing camera");
    } else if existing_cameras.is_empty() {
        // Otherwise spawn a new camera with orbit controls
        commands.spawn((
            Camera3d::default(),
            OrbitCamera::default(),
            Camera {
                order: 0,
                ..default()
            },
            Transform::from_xyz(50.0, 30.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
            Name::new("OrbitCamera"),
        ));
        info!("Spawned new camera with OrbitCamera");
    } else {
        warn!("⚠ Multiple Camera3d entities detected! Using first one.");
        if let Some(camera_entity) = existing_cameras.iter().next() {
            commands
                .entity(camera_entity)
                .insert(OrbitCamera::default());
        }
    }
}
