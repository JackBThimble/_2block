// crates/scene_3d/src/components.rs

use bevy::prelude::*;
use crane_core::{
    CraneConfiguration,
    rigging::{Load, Sling},
};

/// Marker for the main camera
#[derive(Component)]
pub struct MainCamera;

/// Marker for ground plane
#[derive(Component)]
pub struct GroundPlane {
    pub soil_type: crane_core::ground_bearing::SoilType,
}

/// Crane entity - now uses CraneConfiguration from crane_core
#[derive(Component)]
pub struct Crane {
    pub config: CraneConfiguration, // Full config with outriggers, counterweight, etc.
}

/// Crane visual parts (for querying specific pieces)
#[derive(Component)]
pub enum CraneVisualPart {
    Body,
    Boom,
    Cable,
    Hook,
    Outrigger(crane_core::OutriggerPosition),
    CounterweightSlab { index: usize },
}

/// Load entity
#[derive(Component)]
pub struct LiftLoad {
    pub load_data: Load,
    pub is_selected: bool,
}

/// Pick point marker
#[derive(Component)]
pub struct PickPoint {
    pub id: String,
    pub is_selected: bool,
    pub is_hovered: bool,
}

/// Sling visual component
#[derive(Component)]
pub struct SlingComponent {
    pub sling_data: Sling,
    pub tension_kg: f32,
    pub is_safe: bool,
}

/// Outrigger visual component
#[derive(Component)]
pub struct OutriggerVisual {
    pub position: crane_core::OutriggerPosition,
    pub pressure_kpa: f32,
}

#[derive(Component)]
pub struct CranePart {
    pub part_type: CranePartType,
}

#[derive(Component)]
pub enum CranePartType {
    Base,
    Tower,
    Jib,
    Hook,
    Counterweight,
}

#[derive(Component)]
pub struct GridLine;
