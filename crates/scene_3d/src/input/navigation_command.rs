use bevy::prelude::*;

/// Unified navigation commands that work across all platforms
#[derive(Debug, Clone, Copy)]
pub enum NavigationCommand {
    /// Orbit camera around target
    /// Delta is in screen-space pixels
    Orbit { delta: Vec2 },

    /// Pan camera in view plane
    /// Delta is in screen-space pixels
    Pan { delta: Vec2 },

    /// Zoom camera toward/away from target
    /// Factor is multiplicative (1.0 = no change, 2.0 = twice as close)
    Zoom { factor: f32 },

    /// Zoom to a specific point in screen space
    /// Used for pinch-to-zoom on touch devices
    ZoomToPoint { factor: f32, screen_point: Vec2 },

    /// Reset camera to default view
    Reset,

    /// Focus on a specific world point
    FocusOn { world_point: Vec3 },
}

/// Accumulates navigation commands for the current frame
#[derive(Resource, Default)]
pub struct NavigationState {
    pub commands: Vec<NavigationCommand>,
    pub is_navigating: bool,
}

impl NavigationState {
    pub fn add_command(&mut self, command: NavigationCommand) {
        self.commands.push(command);
        self.is_navigating = true;
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.is_navigating = false;
    }
}

/// Apply accumulated navigation commands to the camera
pub fn apply_navigation_commands(
    mut nav_state: ResMut<NavigationState>,
    mut camera_query: Query<&mut crate::camera::orbit_camera::OrbitCamera>,
) {
    if nav_state.commands.is_empty() {
        nav_state.is_navigating = false;
        return;
    }

    let mut camera = match camera_query.single_mut() {
        Ok(cam) => cam,
        Err(_) => {
            nav_state.clear();
            return;
        }
    };

    // Process all commands for this frame
    for command in nav_state.commands.drain(..) {
        match command {
            NavigationCommand::Orbit { delta } => {
                camera.orbit_with_velocity(delta);
            }
            NavigationCommand::Pan { delta } => {
                camera.pan_with_velocity(delta);
            }
            NavigationCommand::Zoom { factor } => {
                camera.zoom_with_velocity(factor);
            }
            NavigationCommand::ZoomToPoint {
                factor,
                screen_point,
            } => {
                camera.zoom_to_point(factor, screen_point);
            }
            NavigationCommand::Reset => {
                camera.reset();
            }
            NavigationCommand::FocusOn { world_point } => {
                camera.focus_on(world_point);
            }
        }
    }

    nav_state.is_navigating = false;
}
