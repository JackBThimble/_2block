// crates/crane_core/src/crane_data/outriggers.rs

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

/// Outrigger position on crane
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutriggerPosition {
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
}

impl OutriggerPosition {
    pub fn all() -> [OutriggerPosition; 4] {
        [
            OutriggerPosition::FrontLeft,
            OutriggerPosition::FrontRight,
            OutriggerPosition::RearLeft,
            OutriggerPosition::RearRight,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OutriggerPosition::FrontLeft => "Front Left",
            OutriggerPosition::FrontRight => "Front Right",
            OutriggerPosition::RearLeft => "Rear Left",
            OutriggerPosition::RearRight => "Rear Right",
        }
    }
}

/// Outrigger deployment state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutriggerDeployment {
    /// Outriggers retracted (crane traveling)
    Retracted,
    /// Outriggers extended horizontally but not set
    Extended,
    /// Outriggers set and supporting crane
    Set {
        /// Vertical extension of jack (m)
        jack_extension_m: f32,
    },
}

/// Configuration for a single outrigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutriggerConfig {
    pub position: OutriggerPosition,
    pub deployment: OutriggerDeployment,

    /// Horizontal extension from crane center (m)
    /// Typically 0.5x to 1.0x of max_extension_m
    pub extension_m: f32,

    /// Maximum horizontal extension (m)
    pub max_extension_m: f32,

    /// Minimum horizontal extension (m)
    pub min_extension_m: f32,

    /// Pad dimensions if using outrigger pads
    pub pad_diameter_m: Option<f32>,
}

impl OutriggerConfig {
    pub fn new(position: OutriggerPosition, max_extension_m: f32, min_extension_m: f32) -> Self {
        Self {
            position,
            deployment: OutriggerDeployment::Retracted,
            extension_m: max_extension_m,
            max_extension_m,
            min_extension_m,
            pad_diameter_m: Some(0.6), // 600mm standard pad
        }
    }

    /// Get the outrigger contact point in crane local coordinates
    /// Assumes crane center is at origin, front is +Y
    pub fn get_contact_point(&self, crane_base_width_m: f32) -> Point3<f32> {
        let half_width = crane_base_width_m / 2.0;

        // Base position on crane body
        let (x_sign, y_sign) = match self.position {
            OutriggerPosition::FrontLeft => (-1.0, 1.0),
            OutriggerPosition::FrontRight => (1.0, 1.0),
            OutriggerPosition::RearLeft => (-1.0, -1.0),
            OutriggerPosition::RearRight => (1.0, -1.0),
        };

        // Diagonal extension
        let diagonal = (half_width.powi(2) + half_width.powi(2)).sqrt();
        let extension_ratio = self.extension_m / diagonal;

        Point3::new(
            x_sign * half_width * (1.0 + extension_ratio),
            y_sign * half_width * (1.0 + extension_ratio),
            0.0, // Ground level
        )
    }

    /// Check if outrigger is fully deployed
    pub fn is_deployed(&self) -> bool {
        matches!(self.deployment, OutriggerDeployment::Set { .. })
    }

    /// Validate outrigger configuration
    pub fn validate(&self) -> crate::crane_data::errors::Result<()> {
        if self.extension_m < self.min_extension_m || self.extension_m > self.max_extension_m {
            return Err(
                crate::crane_data::errors::CraneConfigError::OutriggerExtensionInvalid {
                    extension: self.extension_m,
                    min: self.min_extension_m,
                    max: self.max_extension_m,
                },
            );
        }
        Ok(())
    }
}

/// Complete outrigger system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutriggerSystem {
    pub outriggers: Vec<OutriggerConfig>,

    /// Base width of crane body (m)
    pub crane_base_width_m: f32,

    /// Base length of crane body (m)
    pub crane_base_length_m: f32,

    /// Are all outriggers required to be deployed for operation?
    pub all_required: bool,
}

impl OutriggerSystem {
    pub fn new(crane_base_width_m: f32, crane_base_length_m: f32, max_extension_m: f32) -> Self {
        let min_extension = max_extension_m * 0.5;

        let outriggers = OutriggerPosition::all()
            .iter()
            .map(|&pos| OutriggerConfig::new(pos, max_extension_m, min_extension))
            .collect();

        Self {
            outriggers,
            crane_base_width_m,
            crane_base_length_m,
            all_required: true,
        }
    }

    /// Get outrigger by position
    pub fn get_outrigger(&self, position: OutriggerPosition) -> Option<&OutriggerConfig> {
        self.outriggers.iter().find(|o| o.position == position)
    }

    /// Get mutable outrigger by position
    pub fn get_outrigger_mut(
        &mut self,
        position: OutriggerPosition,
    ) -> Option<&mut OutriggerConfig> {
        self.outriggers.iter_mut().find(|o| o.position == position)
    }

    /// Check if all required outriggers are deployed
    pub fn all_deployed(&self) -> bool {
        if self.all_required {
            self.outriggers.iter().all(|o| o.is_deployed())
        } else {
            self.outriggers.iter().any(|o| o.is_deployed())
        }
    }

    /// Get all contact points in crane local coordinates
    pub fn get_all_contact_points(&self) -> Vec<(OutriggerPosition, Point3<f32>)> {
        self.outriggers
            .iter()
            .filter(|o| o.is_deployed())
            .map(|o| (o.position, o.get_contact_point(self.crane_base_width_m)))
            .collect()
    }

    /// Calculate effective support base (polygon area)
    pub fn calculate_support_area(&self) -> f32 {
        let points = self.get_all_contact_points();
        if points.len() < 3 {
            return 0.0;
        }

        // Simple rectangular approximation for 4 outriggers
        if points.len() == 4 {
            let mut x_vals: Vec<f32> = points.iter().map(|(_, p)| p.x).collect();
            let mut y_vals: Vec<f32> = points.iter().map(|(_, p)| p.y).collect();

            x_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
            y_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let width = x_vals[3] - x_vals[0];
            let length = y_vals[3] - y_vals[0];

            width * length
        } else {
            // TODO: Implement proper polygon area calculation
            0.0
        }
    }

    /// Validate entire outrigger system
    pub fn validate(&self) -> crate::crane_data::errors::Result<()> {
        for outrigger in &self.outriggers {
            outrigger.validate()?;
        }

        if self.all_required && !self.all_deployed() {
            return Err(
                crate::crane_data::errors::CraneConfigError::UnsafeConfiguration {
                    reason: "Not all required outriggers are deployed".to_string(),
                },
            );
        }

        Ok(())
    }
}

/// Preset outrigger configurations
impl OutriggerSystem {
    /// Maximum extension (100% - best stability)
    pub fn preset_max_extension(&mut self) {
        for outrigger in &mut self.outriggers {
            outrigger.extension_m = outrigger.max_extension_m;
            outrigger.deployment = OutriggerDeployment::Set {
                jack_extension_m: 0.5,
            };
        }
    }

    /// Medium extension (75% - good for most lifts)
    pub fn preset_medium_extension(&mut self) {
        for outrigger in &mut self.outriggers {
            outrigger.extension_m = outrigger.max_extension_m * 0.75;
            outrigger.deployment = OutriggerDeployment::Set {
                jack_extension_m: 0.5,
            };
        }
    }

    /// Minimum extension (50% - reduced capacity)
    pub fn preset_min_extension(&mut self) {
        for outrigger in &mut self.outriggers {
            outrigger.extension_m = outrigger.min_extension_m;
            outrigger.deployment = OutriggerDeployment::Set {
                jack_extension_m: 0.5,
            };
        }
    }

    /// On tires (0% - travel mode, no lifting)
    pub fn preset_on_tires(&mut self) {
        for outrigger in &mut self.outriggers {
            outrigger.deployment = OutriggerDeployment::Retracted;
        }
    }
}
