//! Camera system with professional CAD-style orbit controls
//! Features: momentum/inertia, smart clamping, smooth rotation

use bevy::prelude::*;

/// Orbit camera component with industry-standard controls
#[derive(Component)]
pub struct OrbitCamera {
    /// Point we're orbiting around
    pub target: Vec3,

    /// Distance from target
    pub distance: f32,

    /// Rotation around vertical axis (radians)
    pub yaw: f32,

    /// Vertical angle (radians, with smart clamping)
    pub pitch: f32,

    /// Sensitivity settings
    pub orbit_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,

    /// Constraints
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,

    /// Momentum/Inertia for smooth movement
    pub orbit_velocity: Vec2,
    pub pan_velocity: Vec2,
    pub zoom_velocity: f32,
    pub damping: f32,      // 0.0 = instant stop, 0.95 = lots of slide
    pub min_velocity: f32, // Stop threshold

    /// Clamp mode for different use cases
    pub clamp_mode: ClampMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClampMode {
    /// Never go upside down (good for crane operations)
    Restricted,
    /// Allow some over-the-top rotation with auto-flip (RECOMMENDED)
    Relaxed,
    /// Full 360° freedom
    Free,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 80.0,
            yaw: std::f32::consts::FRAC_PI_6,
            pitch: -std::f32::consts::FRAC_PI_4,
            orbit_sensitivity: 0.005,
            pan_sensitivity: 0.002,
            zoom_sensitivity: 0.1,
            min_distance: 1.0,
            max_distance: 1000.0,
            min_pitch: -0.2, // Allow slight over-rotation
            max_pitch: std::f32::consts::PI + 0.2,

            // Momentum settings - tune these for feel
            orbit_velocity: Vec2::ZERO,
            pan_velocity: Vec2::ZERO,
            zoom_velocity: 0.0,
            damping: 0.90, // 90% retention = smooth, 0.8 = snappier
            min_velocity: 0.01,

            clamp_mode: ClampMode::Free, // Best for crane sim
        }
    }
}

impl OrbitCamera {
    // ========================================================================
    // CORE MOVEMENT WITH VELOCITY TRACKING
    // ========================================================================

    /// Orbit camera with velocity tracking for momentum
    pub fn orbit_with_velocity(&mut self, delta: Vec2) {
        self.orbit(delta);
        // Store velocity for momentum (averaged with previous)
        self.orbit_velocity = (self.orbit_velocity + delta) * 0.5;
    }

    /// Orbit camera based on screen-space delta (NO GLITCHY CLAMP!)
    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw -= delta.x * self.orbit_sensitivity;
        let new_pitch = self.pitch + delta.y * self.orbit_sensitivity;

        match self.clamp_mode {
            ClampMode::Restricted => {
                // Hard clamp for professional work (old behavior)
                self.pitch = new_pitch.clamp(0.01, std::f32::consts::PI - 0.01);
            }

            ClampMode::Relaxed => {
                // SMOOTH OVER-THE-TOP ROTATION (no glitch!)
                self.pitch = new_pitch.clamp(self.min_pitch, self.max_pitch);

                // If we go over the poles, flip yaw and mirror pitch
                if self.pitch < 0.0 {
                    self.pitch = -self.pitch;
                    self.yaw += std::f32::consts::PI;
                } else if self.pitch > std::f32::consts::PI {
                    self.pitch = std::f32::consts::TAU - self.pitch;
                    self.yaw += std::f32::consts::PI;
                }
            }

            ClampMode::Free => {
                // No restrictions - full 360° tumble
                self.pitch = new_pitch.rem_euclid(std::f32::consts::TAU);
            }
        }

        // Normalize yaw to 0-2π
        self.yaw = self.yaw.rem_euclid(std::f32::consts::TAU);
    }

    /// Pan camera with velocity tracking for momentum
    pub fn pan_with_velocity(&mut self, delta: Vec2) {
        self.pan(delta);
        self.pan_velocity = (self.pan_velocity + delta) * 0.5;
    }

    /// Pan camera in view plane
    pub fn pan(&mut self, delta: Vec2) {
        // Scale pan speed by distance for consistent feel
        let pan_scale = self.distance * self.pan_sensitivity;

        // Get camera orientation
        let rotation = self.get_rotation();
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;

        // Pan in view space
        self.target += right * (-delta.x * pan_scale);
        self.target += up * (delta.y * pan_scale);
    }

    /// Zoom camera with velocity tracking for momentum
    pub fn zoom_with_velocity(&mut self, factor: f32) {
        self.zoom(factor);
        // Store zoom velocity as change factor
        self.zoom_velocity = (self.zoom_velocity + (factor - 1.0)) * 0.5;
    }

    /// Zoom camera (multiplicative)
    pub fn zoom(&mut self, factor: f32) {
        self.distance *= factor;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }

    // ========================================================================
    // MOMENTUM SYSTEM (THE SLICK SAUCE)
    // ========================================================================

    /// Apply velocity with damping (call every frame when NOT navigating)
    pub fn apply_momentum(&mut self, delta_time: f32) {
        let frame_rate_compensation = delta_time * 60.0;

        // Apply orbital momentum
        if self.orbit_velocity.length_squared() > self.min_velocity * self.min_velocity {
            self.orbit(self.orbit_velocity * frame_rate_compensation);
            self.orbit_velocity *= self.damping;
        } else {
            self.orbit_velocity = Vec2::ZERO;
        }

        // Apply pan momentum
        if self.pan_velocity.length_squared() > self.min_velocity * self.min_velocity {
            self.pan(self.pan_velocity * frame_rate_compensation);
            self.pan_velocity *= self.damping;
        } else {
            self.pan_velocity = Vec2::ZERO;
        }

        // Apply zoom momentum
        if self.zoom_velocity.abs() > self.min_velocity * 0.1 {
            let zoom_factor = 1.0 + self.zoom_velocity * frame_rate_compensation;
            self.zoom(zoom_factor);
            self.zoom_velocity *= self.damping;
        } else {
            self.zoom_velocity = 0.0;
        }
    }

    /// Stop all momentum (call when user grabs camera again)
    pub fn stop_momentum(&mut self) {
        self.orbit_velocity = Vec2::ZERO;
        self.pan_velocity = Vec2::ZERO;
        self.zoom_velocity = 0.0;
    }

    // ========================================================================
    // UTILITY FUNCTIONS
    // ========================================================================

    /// Zoom toward a specific screen point (advanced feature for later)
    pub fn zoom_to_point(&mut self, factor: f32, _screen_point: Vec2) {
        // TODO: Implement raycast-based zoom toward cursor/finger
        // For now, just do regular zoom
        self.zoom_with_velocity(factor);
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        let default = Self::default();
        self.target = default.target;
        self.distance = default.distance;
        self.yaw = default.yaw;
        self.pitch = default.pitch;
        self.stop_momentum();
    }

    /// Focus on a specific world point
    pub fn focus_on(&mut self, point: Vec3) {
        self.target = point;
        self.stop_momentum();
    }

    /// Set to a preset view
    pub fn set_preset_view(&mut self, preset: PresetView) {
        match preset {
            PresetView::Front => {
                self.yaw = 0.0;
                self.pitch = std::f32::consts::FRAC_PI_2;
            }
            PresetView::Back => {
                self.yaw = std::f32::consts::PI;
                self.pitch = std::f32::consts::FRAC_PI_2;
            }
            PresetView::Left => {
                self.yaw = -std::f32::consts::FRAC_PI_2;
                self.pitch = std::f32::consts::FRAC_PI_2;
            }
            PresetView::Right => {
                self.yaw = std::f32::consts::FRAC_PI_2;
                self.pitch = std::f32::consts::FRAC_PI_2;
            }
            PresetView::Top => {
                self.pitch = 0.01; // Almost straight down
            }
            PresetView::Bottom => {
                self.pitch = std::f32::consts::PI - 0.01;
            }
            PresetView::Isometric => {
                self.yaw = std::f32::consts::FRAC_PI_4;
                self.pitch = std::f32::consts::FRAC_PI_4;
            }
        }
        self.stop_momentum();
    }

    /// Get camera rotation quaternion
    pub fn get_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }

    /// Get camera position in world space
    pub fn get_position(&self) -> Vec3 {
        let rotation = self.get_rotation();
        let offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        self.target + offset
    }

    /// Get the correct up vector (auto-flips when upside down in Free mode)
    pub fn get_up_vector(&self) -> Vec3 {
        if self.clamp_mode == ClampMode::Free {
            // When pitch is between PI/2 and 3*PI/2, we're upside down
            let normalized_pitch = self.pitch.rem_euclid(std::f32::consts::TAU);

            if normalized_pitch > std::f32::consts::FRAC_PI_2
                && normalized_pitch < std::f32::consts::PI + std::f32::consts::FRAC_PI_2
            {
                return Vec3::NEG_Y; // Upside down - flip up vector
            }
        }
        Vec3::Y // Normal orientation
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PresetView {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
    Isometric,
}
