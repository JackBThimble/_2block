// crates/scene_3d/src/resources.rs

use bevy::prelude::*;
use crane_core::ground_bearing::*;
use crane_core::rigging::*;

#[derive(Resource)]
pub struct SceneConfig {
    pub show_grid: bool,
    pub show_crane_wireframe: bool,
    pub shadow_quality: ShadowQuality,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_crane_wireframe: false,
            shadow_quality: ShadowQuality::High,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Resource)]
pub struct SceneState {
    /// Current crane configuration
    pub crane_config: Option<crane_core::CraneConfiguration>,

    /// Current rigging configuration and analysis
    pub rigging_config: Option<RiggingConfiguration>,
    pub rigging_analysis: Option<RiggingAnalysis>,

    /// Ground bearing analysis
    pub ground_bearing_analysis: Option<GroundBearingAnalysis>,

    /// Display toggles
    pub show_tensions: bool,
    pub show_angles: bool,
    pub show_ground_pressure: bool,
    pub show_measurements: bool,
    pub show_outriggers: bool,
    pub show_counterweight: bool,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            crane_config: None,
            rigging_config: None,
            rigging_analysis: None,
            ground_bearing_analysis: None,
            show_tensions: true,
            show_angles: true,
            show_ground_pressure: true,
            show_measurements: false,
            show_outriggers: true,
            show_counterweight: true,
        }
    }
}

#[derive(Resource)]
pub struct InputSettings {
    pub touch_sensitivity: f32,
    pub mouse_sensitivity: f32,
    pub inertia_enabled: bool,
    pub inertia_strength: f32,
    pub haptics_enabled: bool,
    pub edge_swipe_enabled: bool,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            touch_sensitivity: 1.0,
            mouse_sensitivity: 1.0,
            inertia_enabled: true,
            inertia_strength: 0.90,
            haptics_enabled: true,
            edge_swipe_enabled: true,
        }
    }
}
