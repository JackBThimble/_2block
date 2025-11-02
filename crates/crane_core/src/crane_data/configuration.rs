use super::counterweight::CounterweightConfig;
use super::errors::{CraneConfigError, Result};
use super::outriggers::OutriggerSystem;
use super::spec::CraneSpec;
use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Complete crane configuration at a specific moment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraneConfiguration {
    // Crane spec (immutable)
    pub spec: CraneSpec,

    // Position & orientation
    pub position: Point3<f32>,
    pub heading_deg: f32, // Crane facing direction (0° = North)

    // Boom configuration
    pub boom_length_m: f32,
    pub boom_angle_deg: f32,
    pub swing_angle_deg: f32, // Relative to crane heading

    // Hoist configuration
    pub hoist_length_m: f32,

    // Outriggers
    pub outriggers: OutriggerSystem,

    // Counterweight
    pub counterweight: CounterweightConfig,
}

impl CraneConfiguration {
    pub fn new(spec: CraneSpec) -> Self {
        let outriggers = spec.create_outrigger_system();
        let counterweight = spec.create_counterweight_config();

        let _default_hoist = spec.hoist_length_range.0 + spec.hoist_length_range.1;

        Self {
            spec,
            position: Point3::origin(),
            heading_deg: 0.0,
            boom_length_m: 30.0,
            boom_angle_deg: 60.0,
            swing_angle_deg: 0.0,
            hoist_length_m: 10.0, // Default to 10m of cable
            outriggers,
            counterweight,
        }
    }

    /// Get boom tip position (where sheave/pulley is)
    pub fn get_boom_tip_position(&self) -> Point3<f32> {
        crate::kinematics::calculate_boom_tip_position(
            self.position,
            self.boom_length_m,
            self.boom_angle_deg,
            self.swing_angle_deg + self.heading_deg,
            self.spec.boom_pivot_height_m,
        )
    }

    /// Get hook position in world coordinates
    pub fn get_hook_position(&self) -> Point3<f32> {
        crate::kinematics::calculate_hook_position(
            self.position,
            self.boom_length_m,
            self.boom_angle_deg,
            self.swing_angle_deg + self.heading_deg,
            self.spec.boom_pivot_height_m,
            self.hoist_length_m,
        )
    }

    /// Get current operating radius
    pub fn get_radius(&self) -> f32 {
        let boom_angle_rad = self.boom_angle_deg.to_radians();
        self.boom_length_m * boom_angle_rad.cos()
    }

    /// Get current hook height
    pub fn get_hook_height(&self) -> f32 {
        self.get_hook_position().z
    }

    /// Get capacity at current configuration
    pub fn get_current_capacity(&self) -> Option<f32> {
        let radius = self.get_radius();
        let on_tires = !self.outriggers.all_deployed();

        // Calculate average outrigger extension percentage
        let avg_extension = if on_tires {
            0.0
        } else {
            let total: f32 = self
                .outriggers
                .outriggers
                .iter()
                .map(|o| o.extension_m / o.max_extension_m)
                .sum();
            total / self.outriggers.outriggers.len() as f32
        };

        self.spec.capacity_chart.get_capacity(
            self.boom_length_m,
            radius,
            self.swing_angle_deg,
            avg_extension,
            on_tires,
        )
    }

    /// Check if a load is within capacity
    pub fn can_lift(&self, load_kg: f32) -> Result<bool> {
        let capacity =
            self.get_current_capacity()
                .ok_or_else(|| CraneConfigError::UnsafeConfiguration {
                    reason: "Cannot determine capacity for current configuration".to_string(),
                })?;

        // Apply 75% of chart capacity as safe working load
        let safe_capacity = capacity * 0.75;

        if load_kg > capacity {
            return Err(CraneConfigError::LoadExceedsCapacity {
                load_kg,
                capacity_kg: capacity,
                radius_m: self.get_radius(),
            });
        }

        Ok(load_kg <= safe_capacity)
    }

    /// Validate entire crane configuration
    pub fn validate(&self) -> Result<()> {
        // Check boom length
        if self.boom_length_m < self.spec.boom_length_range.0
            || self.boom_length_m > self.spec.boom_length_range.1
        {
            return Err(CraneConfigError::BoomLengthOutOfRange {
                current: self.boom_length_m,
                min: self.spec.boom_length_range.0,
                max: self.spec.boom_length_range.1,
            });
        }

        // Check boom angle
        if self.boom_angle_deg < self.spec.min_boom_angle_deg
            || self.boom_angle_deg > self.spec.max_boom_angle_deg
        {
            return Err(CraneConfigError::BoomAngleInvalid {
                angle: self.boom_angle_deg,
            });
        }

        // Check radius
        let radius = self.get_radius();
        if radius < self.spec.min_radius_m || radius > self.spec.max_radius_m {
            return Err(CraneConfigError::RadiusOutOfRange {
                current: radius,
                min: self.spec.min_radius_m,
                max: self.spec.max_radius_m,
            });
        }

        // Check height
        let height = self.get_hook_height();
        if height > self.spec.max_tip_height_m {
            return Err(CraneConfigError::HeightExceeded {
                current: height,
                max: self.spec.max_tip_height_m,
            });
        }

        // Validate outriggers
        self.outriggers.validate()?;

        // Validate counterweight
        self.counterweight.validate()?;

        Ok(())
    }

    /// Get total crane weight (base + counterweight)
    pub fn get_total_weight_kg(&self) -> f32 {
        self.spec.base_weight_kg + self.counterweight.get_total_weight_kg()
    }
}

/// Lightweight state for hot paths
#[derive(Debug, Clone, Copy)]
pub struct CraneState {
    pub boom_length_m: f32,
    pub boom_angle_deg: f32,
    pub swing_angle_deg: f32,
    pub position: Point3<f32>,
}

impl From<&CraneConfiguration> for CraneState {
    fn from(config: &CraneConfiguration) -> Self {
        Self {
            boom_length_m: config.boom_length_m,
            boom_angle_deg: config.boom_angle_deg,
            swing_angle_deg: config.swing_angle_deg,
            position: config.position,
        }
    }
}
