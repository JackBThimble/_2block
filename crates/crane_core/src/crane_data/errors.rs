use std::fmt;

#[derive(Debug, Clone)]
pub enum CraneConfigError {
    BoomLengthOutOfRange {
        current: f32,
        min: f32,
        max: f32,
    },
    BoomAngleInvalid {
        angle: f32,
    },
    RadiusOutOfRange {
        current: f32,
        min: f32,
        max: f32,
    },
    HeightExceeded {
        current: f32,
        max: f32,
    },
    LoadExceedsCapacity {
        load_kg: f32,
        capacity_kg: f32,
        radius_m: f32,
    },
    OutriggerPositionInvalid {
        position: String,
    },
    OutriggerExtensionInvalid {
        extension: f32,
        min: f32,
        max: f32,
    },
    CounterweightInvalid {
        weight_kg: f32,
        min: f32,
        max: f32,
    },
    CapacityChartNotFound {
        boom_length: f32,
    },
    UnsafeConfiguration {
        reason: String,
    },
}

impl fmt::Display for CraneConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CraneConfigError::BoomLengthOutOfRange { current, min, max } => {
                write!(
                    f,
                    "Boom length {:.1}m is out of range ({:.1}m - {:.1}m)",
                    current, min, max
                )
            }
            CraneConfigError::BoomAngleInvalid { angle } => {
                write!(f, "Boom angle {:.1}° is invalid (must be 0-85°)", angle)
            }
            CraneConfigError::RadiusOutOfRange { current, min, max } => {
                write!(
                    f,
                    "Radius {:.1}m is out of range ({:.1}m - {:.1}m)",
                    current, min, max
                )
            }
            CraneConfigError::HeightExceeded { current, max } => {
                write!(f, "Hook height {:.1}m exceeds maximum {:.1}m", current, max)
            }
            CraneConfigError::LoadExceedsCapacity {
                load_kg,
                capacity_kg,
                radius_m,
            } => {
                write!(
                    f,
                    "Load {:.0}kg exceeds capacity {:.0}kg at {:.1}m radius",
                    load_kg, capacity_kg, radius_m
                )
            }
            CraneConfigError::OutriggerPositionInvalid { position } => {
                write!(f, "Invalid outrigger position: {}", position)
            }
            CraneConfigError::OutriggerExtensionInvalid {
                extension,
                min,
                max,
            } => {
                write!(
                    f,
                    "Outrigger extension {:.1}m out of range ({:.1}m - {:.1}m)",
                    extension, min, max
                )
            }
            CraneConfigError::CounterweightInvalid {
                weight_kg,
                min,
                max,
            } => {
                write!(
                    f,
                    "Counterweight {:.0}kg invalid (must be {:.0}kg - {:.0}kg)",
                    weight_kg, min, max
                )
            }
            CraneConfigError::CapacityChartNotFound { boom_length } => {
                write!(
                    f,
                    "No capacity chart found for boom length {:.1}m",
                    boom_length
                )
            }
            CraneConfigError::UnsafeConfiguration { reason } => {
                write!(f, "Unsafe configuration: {}", reason)
            }
        }
    }
}

impl std::error::Error for CraneConfigError {}
pub type Result<T> = std::result::Result<T, CraneConfigError>;
