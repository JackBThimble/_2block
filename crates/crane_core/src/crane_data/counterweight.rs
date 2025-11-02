// crates/crane_core/src/crane_data/counterweight.rs

use serde::{Deserialize, Serialize};

/// Individual counterweight slab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterweightSlab {
    pub id: String,
    pub weight_kg: f32,
    pub position_index: usize, // Position in stack (0 = closest to crane)
}

/// Complete counterweight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterweightConfig {
    /// Individual slabs installed
    pub slabs: Vec<CounterweightSlab>,

    /// Standard slab weight (kg) - typically 2000-5000 kg
    pub standard_slab_weight_kg: f32,

    /// Maximum number of slabs allowed
    pub max_slabs: usize,

    /// Minimum counterweight for any lift (kg)
    pub min_counterweight_kg: f32,

    /// Maximum counterweight capacity (kg)
    pub max_counterweight_kg: f32,

    /// Distance from crane center to counterweight center of mass (m)
    pub moment_arm_m: f32,
}

impl CounterweightConfig {
    pub fn new(standard_slab_weight_kg: f32, max_slabs: usize, moment_arm_m: f32) -> Self {
        let min_counterweight_kg = standard_slab_weight_kg;
        let max_counterweight_kg = standard_slab_weight_kg * max_slabs as f32;

        Self {
            slabs: vec![],
            standard_slab_weight_kg,
            max_slabs,
            min_counterweight_kg,
            max_counterweight_kg,
            moment_arm_m,
        }
    }

    /// Get total counterweight mass
    pub fn get_total_weight_kg(&self) -> f32 {
        self.slabs.iter().map(|s| s.weight_kg).sum()
    }

    /// Get number of slabs installed
    pub fn get_slab_count(&self) -> usize {
        self.slabs.len()
    }

    /// Add a standard slab
    pub fn add_slab(&mut self) -> crate::crane_data::errors::Result<()> {
        if self.slabs.len() >= self.max_slabs {
            return Err(
                crate::crane_data::errors::CraneConfigError::UnsafeConfiguration {
                    reason: format!(
                        "Cannot add more than {} counterweight slabs",
                        self.max_slabs
                    ),
                },
            );
        }

        let slab = CounterweightSlab {
            id: format!("slab_{}", self.slabs.len() + 1),
            weight_kg: self.standard_slab_weight_kg,
            position_index: self.slabs.len(),
        };

        self.slabs.push(slab);
        Ok(())
    }

    /// Remove the top slab
    pub fn remove_slab(&mut self) -> Option<CounterweightSlab> {
        self.slabs.pop()
    }

    /// Set to specific number of standard slabs
    pub fn set_slab_count(&mut self, count: usize) -> crate::crane_data::errors::Result<()> {
        if count > self.max_slabs {
            return Err(
                crate::crane_data::errors::CraneConfigError::CounterweightInvalid {
                    weight_kg: count as f32 * self.standard_slab_weight_kg,
                    min: self.min_counterweight_kg,
                    max: self.max_counterweight_kg,
                },
            );
        }

        self.slabs.clear();
        for i in 0..count {
            self.slabs.push(CounterweightSlab {
                id: format!("slab_{}", i + 1),
                weight_kg: self.standard_slab_weight_kg,
                position_index: i,
            });
        }

        Ok(())
    }

    /// Calculate counterweight moment (kg⋅m) about crane center
    pub fn calculate_moment(&self) -> f32 {
        self.get_total_weight_kg() * self.moment_arm_m
    }

    /// Validate counterweight configuration
    pub fn validate(&self) -> crate::crane_data::errors::Result<()> {
        let total = self.get_total_weight_kg();

        if total < self.min_counterweight_kg || total > self.max_counterweight_kg {
            return Err(
                crate::crane_data::errors::CraneConfigError::CounterweightInvalid {
                    weight_kg: total,
                    min: self.min_counterweight_kg,
                    max: self.max_counterweight_kg,
                },
            );
        }

        if self.slabs.len() > self.max_slabs {
            return Err(
                crate::crane_data::errors::CraneConfigError::UnsafeConfiguration {
                    reason: format!(
                        "Too many counterweight slabs: {} > {}",
                        self.slabs.len(),
                        self.max_slabs
                    ),
                },
            );
        }

        Ok(())
    }
}

/// Preset counterweight configurations
impl CounterweightConfig {
    /// Maximum counterweight (heaviest lifts at long radius)
    pub fn preset_max(&mut self) -> crate::crane_data::errors::Result<()> {
        self.set_slab_count(self.max_slabs)
    }

    /// Medium counterweight (typical lifts)
    pub fn preset_medium(&mut self) -> crate::crane_data::errors::Result<()> {
        let count = self.max_slabs / 2;
        self.set_slab_count(count)
    }

    /// Minimum counterweight (light lifts at close radius)
    pub fn preset_min(&mut self) -> crate::crane_data::errors::Result<()> {
        self.set_slab_count(1)
    }
}
