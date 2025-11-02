pub mod crane_data;
pub mod ground_bearing;
pub mod kinematics;
pub mod rigging;

// Re-export commonly used types
pub use crane_data::{
    CapacityChart, CapacityPoint, CounterweightConfig, CounterweightSlab, CraneConfigError,
    CraneConfiguration, CraneSpec, CraneState, CraneType, LoadChart, OutriggerConfig,
    OutriggerPosition, OutriggerSystem,
};

pub use rigging::{
    Load, PickPoint, RiggingAnalysis, RiggingCalculator, RiggingConfiguration, Sling,
    SlingMaterial, SlingSpec,
};

pub use ground_bearing::{
    BearingPressure, GroundBearingAnalysis, GroundBearingCalculator, GroundConfiguration,
    MatMaterial, PadMaterial, SoilType, SupportPoint, SupportType,
};

pub use kinematics::{
    calculate_boom_angle_for_height, calculate_boom_tip_position,
    calculate_hoist_length_for_height, calculate_hook_position, calculate_swing_path,
    check_clearance,
};
