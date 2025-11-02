// crates/crane_core/src/crane_data/spec.rs

use super::capacity::CapacityChart;
use super::counterweight::CounterweightConfig;
use super::outriggers::OutriggerSystem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraneSpec {
    // Basic info
    pub id: String,
    pub manufacturer: String,
    pub model: String,
    pub year: Option<u16>,
    pub crane_type: CraneType,

    // Weight & dimensions
    pub base_weight_kg: f32,
    pub transport_weight_kg: f32,
    pub length_m: f32,
    pub width_m: f32,
    pub height_m: f32,

    // Boom specifications
    pub boom_length_range: (f32, f32),
    pub boom_sections: usize,
    pub boom_pivot_height_m: f32,
    pub max_boom_angle_deg: f32,
    pub min_boom_angle_deg: f32,

    // Hoist specifications
    pub hoist_length_range: (f32, f32),
    pub max_hoist_speed_m_per_min: Option<f32>,

    // Capacity
    pub max_capacity_kg: f32,
    pub max_radius_m: f32,
    pub min_radius_m: f32,
    pub max_tip_height_m: f32,

    // Outriggers
    pub outrigger_base_width_m: f32,
    pub outrigger_base_length_m: f32,
    pub outrigger_max_extension_m: f32,

    // Counterweight
    pub counterweight_slab_weight_kg: f32,
    pub counterweight_max_slabs: usize,
    pub counterweight_moment_arm_m: f32,

    // Capacity charts
    pub capacity_chart: CapacityChart,

    // Engine & hydraulics
    pub engine_power_kw: Option<f32>,
    pub max_swing_speed_rpm: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraneType {
    AllTerrain,   // Mobile crane with all-wheel drive
    RoughTerrain, // Off-road mobile crane
    TruckMounted, // Crane mounted on truck chassis
    Crawler,      // Tracked crane
    Tower,        // Fixed tower crane
}

impl CraneSpec {
    /// Get all available crane specs
    pub fn all_specs() -> Vec<CraneSpec> {
        vec![
            Self::liebherr_ltm_1100(),
            Self::liebherr_ltm_1500(),
            Self::grove_gmk_5150(),
            Self::grove_gmk_6300(),
            Self::tadano_gr_600xl(),
            Self::tadano_gr_1000xl(),
            Self::terex_rt_780(),
            Self::link_belt_htc_8690(),
        ]
    }

    /// Load capacity chart from CSV file
    pub fn load_capacity_chart_csv(&mut self, csv_path: &str) -> Result<(), String> {
        let csv_data =
            std::fs::read_to_string(csv_path).map_err(|e| format!("Failed to read CSV: {}", e))?;

        self.capacity_chart = crate::crane_data::CapacityChartBuilder::new()
            .add_charts_from_csv(&csv_data)?
            .build();

        Ok(())
    }

    /// Load capacity chart from JSON file
    pub fn load_capacity_chart_json(&mut self, json_path: &str) -> Result<(), String> {
        let json_data = std::fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read JSON: {}", e))?;

        self.capacity_chart = crate::crane_data::CapacityChartBuilder::new()
            .add_charts_from_json(&json_data)?
            .build();

        Ok(())
    }

    /// Liebherr LTM 1100-5.2 (100 tonne all-terrain)
    pub fn liebherr_ltm_1100() -> Self {
        Self {
            id: "liebherr_ltm_1100_5_2".to_string(),
            manufacturer: "Liebherr".to_string(),
            model: "LTM 1100-5.2".to_string(),
            year: Some(2020),
            crane_type: CraneType::AllTerrain,

            base_weight_kg: 48_000.0,
            transport_weight_kg: 60_000.0,
            length_m: 13.6,
            width_m: 2.75,
            height_m: 3.85,

            boom_length_range: (15.0, 52.0),
            boom_sections: 5,
            boom_pivot_height_m: 3.2,
            max_boom_angle_deg: 85.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (2.0, 60.0),
            max_hoist_speed_m_per_min: Some(110.0),

            max_capacity_kg: 100_000.0,
            max_radius_m: 48.0,
            min_radius_m: 3.0,
            max_tip_height_m: 56.0,

            outrigger_base_width_m: 2.75,
            outrigger_base_length_m: 3.0,
            outrigger_max_extension_m: 7.1,

            counterweight_slab_weight_kg: 2_500.0,
            counterweight_max_slabs: 16,
            counterweight_moment_arm_m: 4.5,

            capacity_chart: CapacityChart::example_liebherr_ltm_1100(),

            engine_power_kw: Some(380.0),
            max_swing_speed_rpm: Some(1.8),
        }
    }

    /// Liebherr LTM 1500-8.1 (500 tonne all-terrain)
    pub fn liebherr_ltm_1500() -> Self {
        Self {
            id: "liebherr_ltm_1500_8_1".to_string(),
            manufacturer: "Liebherr".to_string(),
            model: "LTM 1500-8.1".to_string(),
            year: Some(2019),
            crane_type: CraneType::AllTerrain,

            base_weight_kg: 108_000.0,
            transport_weight_kg: 132_000.0,
            length_m: 17.8,
            width_m: 3.0,
            height_m: 4.0,

            boom_length_range: (15.4, 84.0),
            boom_sections: 8,
            boom_pivot_height_m: 4.2,
            max_boom_angle_deg: 85.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (3.0, 100.0),
            max_hoist_speed_m_per_min: Some(145.0),

            max_capacity_kg: 500_000.0,
            max_radius_m: 78.0,
            min_radius_m: 3.5,
            max_tip_height_m: 91.0,

            outrigger_base_width_m: 3.0,
            outrigger_base_length_m: 3.5,
            outrigger_max_extension_m: 9.2,

            counterweight_slab_weight_kg: 5_000.0,
            counterweight_max_slabs: 38,
            counterweight_moment_arm_m: 6.5,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(680.0),
            max_swing_speed_rpm: Some(1.5),
        }
    }

    /// Grove GMK 5150L (150 tonne all-terrain)
    pub fn grove_gmk_5150() -> Self {
        Self {
            id: "grove_gmk_5150l".to_string(),
            manufacturer: "Grove".to_string(),
            model: "GMK 5150L".to_string(),
            year: Some(2019),
            crane_type: CraneType::AllTerrain,

            base_weight_kg: 60_000.0,
            transport_weight_kg: 72_000.0,
            length_m: 15.47,
            width_m: 2.75,
            height_m: 3.98,

            boom_length_range: (15.2, 60.0),
            boom_sections: 6,
            boom_pivot_height_m: 3.5,
            max_boom_angle_deg: 85.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (2.0, 70.0),
            max_hoist_speed_m_per_min: Some(135.0),

            max_capacity_kg: 150_000.0,
            max_radius_m: 54.0,
            min_radius_m: 3.0,
            max_tip_height_m: 66.0,

            outrigger_base_width_m: 3.0,
            outrigger_base_length_m: 3.5,
            outrigger_max_extension_m: 7.5,

            counterweight_slab_weight_kg: 3_000.0,
            counterweight_max_slabs: 20,
            counterweight_moment_arm_m: 5.0,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(450.0),
            max_swing_speed_rpm: Some(2.0),
        }
    }

    /// Grove GMK 6300L (300 tonne all-terrain)
    pub fn grove_gmk_6300() -> Self {
        Self {
            id: "grove_gmk_6300l".to_string(),
            manufacturer: "Grove".to_string(),
            model: "GMK 6300L".to_string(),
            year: Some(2021),
            crane_type: CraneType::AllTerrain,

            base_weight_kg: 84_000.0,
            transport_weight_kg: 108_000.0,
            length_m: 16.7,
            width_m: 3.0,
            height_m: 4.0,

            boom_length_range: (16.0, 80.0),
            boom_sections: 7,
            boom_pivot_height_m: 4.0,
            max_boom_angle_deg: 85.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (3.0, 90.0),
            max_hoist_speed_m_per_min: Some(150.0),

            max_capacity_kg: 300_000.0,
            max_radius_m: 72.0,
            min_radius_m: 3.5,
            max_tip_height_m: 88.0,

            outrigger_base_width_m: 3.0,
            outrigger_base_length_m: 3.8,
            outrigger_max_extension_m: 8.8,

            counterweight_slab_weight_kg: 4_000.0,
            counterweight_max_slabs: 30,
            counterweight_moment_arm_m: 6.0,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(580.0),
            max_swing_speed_rpm: Some(1.6),
        }
    }

    /// Tadano GR-600XL (60 tonne rough terrain)
    pub fn tadano_gr_600xl() -> Self {
        Self {
            id: "tadano_gr_600xl".to_string(),
            manufacturer: "Tadano".to_string(),
            model: "GR-600XL".to_string(),
            year: Some(2021),
            crane_type: CraneType::RoughTerrain,

            base_weight_kg: 36_000.0,
            transport_weight_kg: 42_000.0,
            length_m: 11.5,
            width_m: 2.49,
            height_m: 3.63,

            boom_length_range: (10.9, 42.7),
            boom_sections: 4,
            boom_pivot_height_m: 2.8,
            max_boom_angle_deg: 82.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (1.5, 50.0),
            max_hoist_speed_m_per_min: Some(95.0),

            max_capacity_kg: 60_000.0,
            max_radius_m: 40.0,
            min_radius_m: 2.5,
            max_tip_height_m: 47.0,

            outrigger_base_width_m: 2.49,
            outrigger_base_length_m: 2.8,
            outrigger_max_extension_m: 5.9,

            counterweight_slab_weight_kg: 2_000.0,
            counterweight_max_slabs: 10,
            counterweight_moment_arm_m: 3.8,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(275.0),
            max_swing_speed_rpm: Some(1.5),
        }
    }

    /// Tadano GR-1000XL (100 tonne rough terrain)
    pub fn tadano_gr_1000xl() -> Self {
        Self {
            id: "tadano_gr_1000xl".to_string(),
            manufacturer: "Tadano".to_string(),
            model: "GR-1000XL".to_string(),
            year: Some(2020),
            crane_type: CraneType::RoughTerrain,

            base_weight_kg: 52_000.0,
            transport_weight_kg: 64_000.0,
            length_m: 13.2,
            width_m: 2.99,
            height_m: 3.83,

            boom_length_range: (13.7, 50.0),
            boom_sections: 5,
            boom_pivot_height_m: 3.1,
            max_boom_angle_deg: 83.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (2.0, 65.0),
            max_hoist_speed_m_per_min: Some(120.0),

            max_capacity_kg: 100_000.0,
            max_radius_m: 46.0,
            min_radius_m: 3.0,
            max_tip_height_m: 56.0,

            outrigger_base_width_m: 2.99,
            outrigger_base_length_m: 3.2,
            outrigger_max_extension_m: 7.3,

            counterweight_slab_weight_kg: 2_800.0,
            counterweight_max_slabs: 14,
            counterweight_moment_arm_m: 4.3,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(365.0),
            max_swing_speed_rpm: Some(1.7),
        }
    }

    /// Terex RT 780 (75 tonne rough terrain)
    pub fn terex_rt_780() -> Self {
        Self {
            id: "terex_rt_780".to_string(),
            manufacturer: "Terex".to_string(),
            model: "RT 780".to_string(),
            year: Some(2022),
            crane_type: CraneType::RoughTerrain,

            base_weight_kg: 43_000.0,
            transport_weight_kg: 52_000.0,
            length_m: 12.3,
            width_m: 2.9,
            height_m: 3.76,

            boom_length_range: (11.9, 47.2),
            boom_sections: 5,
            boom_pivot_height_m: 3.0,
            max_boom_angle_deg: 82.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (2.0, 55.0),
            max_hoist_speed_m_per_min: Some(106.0),

            max_capacity_kg: 75_000.0,
            max_radius_m: 44.0,
            min_radius_m: 2.8,
            max_tip_height_m: 52.0,

            outrigger_base_width_m: 2.9,
            outrigger_base_length_m: 3.1,
            outrigger_max_extension_m: 6.7,

            counterweight_slab_weight_kg: 2_300.0,
            counterweight_max_slabs: 12,
            counterweight_moment_arm_m: 4.0,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(335.0),
            max_swing_speed_rpm: Some(1.6),
        }
    }

    /// Link-Belt HTC-8690 (90 tonne truck crane)
    pub fn link_belt_htc_8690() -> Self {
        Self {
            id: "link_belt_htc_8690".to_string(),
            manufacturer: "Link-Belt".to_string(),
            model: "HTC-8690".to_string(),
            year: Some(2021),
            crane_type: CraneType::TruckMounted,

            base_weight_kg: 48_500.0,
            transport_weight_kg: 58_000.0,
            length_m: 13.1,
            width_m: 2.59,
            height_m: 3.81,

            boom_length_range: (12.8, 50.3),
            boom_sections: 5,
            boom_pivot_height_m: 3.0,
            max_boom_angle_deg: 83.0,
            min_boom_angle_deg: 0.0,

            hoist_length_range: (2.0, 60.0),
            max_hoist_speed_m_per_min: Some(115.0),

            max_capacity_kg: 90_000.0,
            max_radius_m: 46.0,
            min_radius_m: 2.8,
            max_tip_height_m: 55.0,

            outrigger_base_width_m: 2.59,
            outrigger_base_length_m: 3.0,
            outrigger_max_extension_m: 7.0,

            counterweight_slab_weight_kg: 2_700.0,
            counterweight_max_slabs: 13,
            counterweight_moment_arm_m: 4.2,

            capacity_chart: CapacityChart::new(),

            engine_power_kw: Some(355.0),
            max_swing_speed_rpm: Some(1.7),
        }
    }

    /// Create outrigger system from spec
    pub fn create_outrigger_system(&self) -> OutriggerSystem {
        OutriggerSystem::new(
            self.outrigger_base_width_m,
            self.outrigger_base_length_m,
            self.outrigger_max_extension_m,
        )
    }

    /// Create counterweight config from spec
    pub fn create_counterweight_config(&self) -> CounterweightConfig {
        CounterweightConfig::new(
            self.counterweight_slab_weight_kg,
            self.counterweight_max_slabs,
            self.counterweight_moment_arm_m,
        )
    }
}

impl std::fmt::Display for CraneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraneType::AllTerrain => write!(f, "All-Terrain"),
            CraneType::RoughTerrain => write!(f, "Rough Terrain"),
            CraneType::TruckMounted => write!(f, "Truck Mounted"),
            CraneType::Crawler => write!(f, "Crawler"),
            CraneType::Tower => write!(f, "Tower"),
        }
    }
}
