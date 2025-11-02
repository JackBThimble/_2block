use core::f32;

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportPoint {
    pub position: Point3<f32>,
    pub load_kg: f32,
    pub support_type: SupportType,
}

impl SupportPoint {
    /// Create support point with just an outrigger pad (good ground)
    pub fn with_pad(
        position: Point3<f32>,
        load_kg: f32,
        pad_diameter_m: f32,
        pad_material: PadMaterial,
    ) -> Self {
        Self {
            position,
            load_kg,
            support_type: SupportType::OutriggerPad {
                pad_diameter_m,
                pad_material,
            },
        }
    }

    /// Create support point with mat and pad (soft ground)
    pub fn with_mat_and_pad(
        position: Point3<f32>,
        load_kg: f32,
        mat_length_m: f32,
        mat_width_m: f32,
        mat_material: MatMaterial,
        pad_diameter_m: f32,
        pad_material: PadMaterial,
    ) -> Self {
        Self {
            position,
            load_kg,
            support_type: SupportType::MatWithPad {
                mat_length_m,
                mat_width_m,
                mat_material,
                pad_diameter_m,
                pad_material,
            },
        }
    }

    /// Get contact area with ground
    pub fn contact_area_m2(&self) -> f32 {
        self.support_type.contact_area_m2()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportType {
    /// Direct support with just a pad (good ground conditions)
    OutriggerPad {
        pad_diameter_m: f32,
        pad_material: PadMaterial,
    },

    /// Tire support (crane on tires, much lower capacity)
    Tire {
        tire_width_m: f32,
        tire_diameter_m: f32,
    },

    /// Mat with pad on top (soft ground, needs load spreading)
    MatWithPad {
        mat_length_m: f32,
        mat_width_m: f32,
        mat_material: MatMaterial,
        pad_diameter_m: f32,
        pad_material: PadMaterial,
    },

    /// Just a mat (no separate pad, rare but possible)
    Mat {
        mat_length_m: f32,
        mat_width_m: f32,
        mat_material: MatMaterial,
    },
}

impl SupportType {
    /// Calculate total contact area with ground
    pub fn contact_area_m2(&self) -> f32 {
        match self {
            SupportType::OutriggerPad { pad_diameter_m, .. } => {
                std::f32::consts::PI * (pad_diameter_m / 2.0).powi(2)
            }
            SupportType::Tire {
                tire_width_m,
                tire_diameter_m,
            } => {
                // Approximate contact patch
                tire_width_m * (tire_diameter_m * 0.15)
            }
            SupportType::MatWithPad {
                mat_length_m,
                mat_width_m,
                ..
            } => {
                // Mat determines contact area (spreads load)
                mat_length_m * mat_width_m
            }
            SupportType::Mat {
                mat_length_m,
                mat_width_m,
                ..
            } => mat_length_m * mat_width_m,
        }
    }

    /// Get a description of the support setup
    pub fn description(&self) -> String {
        match self {
            SupportType::OutriggerPad {
                pad_diameter_m,
                pad_material,
            } => {
                format!("{:.1}m {:?} pad", pad_diameter_m, pad_material)
            }
            SupportType::Tire { .. } => "Tire support".to_string(),
            SupportType::MatWithPad {
                mat_length_m,
                mat_width_m,
                mat_material,
                pad_diameter_m,
                pad_material,
            } => {
                format!(
                    "{:.1}m×{:.1}m {:?} mat + {:.1}m {:?} pad",
                    mat_length_m, mat_width_m, mat_material, pad_diameter_m, pad_material
                )
            }
            SupportType::Mat {
                mat_length_m,
                mat_width_m,
                mat_material,
            } => {
                format!(
                    "{:.1}m×{:.1}m {:?} mat",
                    mat_length_m, mat_width_m, mat_material
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PadMaterial {
    Steel,
    Hardwood,
    Composite,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MatMaterial {
    TimberMat,    // Hardwood beams, most common
    CompositeMat, // Modern synthetic materials
    SteelPlate,   // Heavy steel plates for extreme loads
}

/// Soil/ground types with typical bearing capacities
/// Default capacities are generalized and not to be used
/// for actual lift plans
/// All ground pressure capacities must be tested and verified
/// by qualified personnel in the field
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SoilType {
    /// Hard rock (granite, trap, diorite, gneiss)
    HardRock,
    /// Medium rock (marble, some limestone and shist)
    MediumRock,
    /// Intermediate rock (shale, sandstone)
    IntermediateRock,
    /// Soft rock (weathered rock, soft shale)
    SoftRock,
    /// Dense-graded aggregate, compacted tightly
    DenseGravel,
    /// Medium-graded gravel, more stone than fine components
    MediumGravel,
    /// Loose gravel, not compacted, not graded
    LooseGravel,
    /// Dense, compacted sand
    DenseSand,
    /// Medium compacted sand
    MediumSand,
    /// Loose sand, not compacted
    LooseSand,
    /// Hard, natural earthy material
    HardClay,
    /// Stiff, natural earthy material
    StiffClay,
    /// Medium, natural earthy material
    MediumClay,
    /// Medium, natural earthy material
    SoftClay,
    /// Dense sediment (sediment, finer than sand grain but larger than clay)
    /// e.g. riverbeds, channels, harbors
    DenseSilt,
    /// Medium sediment (sediment, finer than sand grain but larger than clay)
    /// e.g. riverbeds, channels, harbors
    MediumSilt,
    /// Loose sediment (sediment, finer than sand grain but larger than clay)
    /// e.g. riverbeds, channels, harbors
    LooseSilt,
    /// Soft soil
    Peat,
    Custom {
        capacity_kpa: f32,
    },
}

impl SoilType {
    /// Get typical allowable bearing pressure in kPa
    pub fn allowable_bearing_capacity_kpa(&self) -> f32 {
        match self {
            SoilType::HardRock => 5746.0,
            SoilType::MediumRock => 3830.0,
            SoilType::IntermediateRock => 1915.0,
            SoilType::SoftRock => 766.0,
            SoilType::DenseGravel => 600.0,
            SoilType::MediumGravel => 400.0,
            SoilType::LooseGravel => 200.0,
            SoilType::DenseSand => 600.0,
            SoilType::MediumSand => 300.0,
            SoilType::LooseSand => 100.0,
            SoilType::HardClay => 479.0,
            SoilType::StiffClay => 287.0,
            SoilType::MediumClay => 192.0,
            SoilType::SoftClay => 100.0,
            SoilType::DenseSilt => 287.0,
            SoilType::MediumSilt => 150.0,
            SoilType::LooseSilt => 75.0,
            SoilType::Peat => 25.0,
            SoilType::Custom { capacity_kpa } => *capacity_kpa,
        }
    }

    /// Get description for UI display
    pub fn description(&self) -> &'static str {
        match self {
            SoilType::SoftClay => {
                "Easily penetrated several inches by the thumb. Can be molded by light finger pressure."
            }
            SoilType::MediumClay => {
                "Penetrated over (1/2 inch) 10mm by thumb with moderate effort. Molded by strong finger pressure."
            }
            SoilType::StiffClay => {
                "Indented about (1/2 inch) 10mm by thumb, but penetrated only with great effort. Readily indented by thumbnail."
            }
            SoilType::HardClay => {
                "Indented with difficulty by thumbnail. Cannot be indented with fingers but can be peeled with knife."
            }
            SoilType::HardRock => {
                "Massive rocks without laminations or defects, such as granite, trap, and diorite. They are unweathered, difficult to break with a hammer, and cannot be molded by the fingers."
            }
            SoilType::MediumRock => {
                "Slightly lower strenghth than hard rock, possibly with laminations, such as limestone and sandstone. They are still very strong and difficult to remove."
            }
            SoilType::IntermediateRock => {
                "Rock mass with possible discontinuities or slight weathering compared to medium rock. Includes residual deposits of shattered and broken bedrock or hard shale."
            }
            SoilType::SoftRock => {
                "Rocks may soften on exposure to air or water, can be removed by picking or spading, and fresh samples may be molded with substantial finger pressure. Includes partially weathered or highly jointed condition, such as uncemented shales and some sandstones. Can require pretreatment in some conditions."
            }
            SoilType::DenseGravel => {
                "Densely compacted gravel. Particles are tightly packed with minimal space between them, making them very stable and strong."
            }
            SoilType::MediumGravel => {
                "Moderately compacted gravel. Particles have some space between them, providing moderate stability."
            }
            SoilType::LooseGravel => {
                "Loose gravel, not compacted. Particles have large, open spaces between them, resulting in low stability and strength. Often needs to be pretreated."
            }
            SoilType::DenseSand => {
                "Densely packed sand. Highly stable, often characterized by a high number of blows per foot during a Standard Penetration Test (SPT)."
            }
            SoilType::MediumSand => {
                "Moderately packed sand. The Standard Penetration Test (SPT) blow count is in the medium range for sands, and often suitable for construction."
            }
            SoilType::LooseSand => {
                "Loosely packed sand. The Standard Penetration Test (SPT) blow count is very low, and the sand is prone to excessive settlement and not suitable for heavy loading without pretreatment."
            }
            SoilType::DenseSilt => {
                "Provides strong resistance to load and penetration. Often found in preloaded or naturally very compact conditions. Comparable to medium-dense sand"
            }
            SoilType::MediumSilt => {
                "Medium sediment. Offers moderate resistance to penetration and load. Often considered 'firm'."
            }
            SoilType::LooseSilt => {
                "Loose sediment. Exhibits high settlement potential under load and can be easily molded or crushed in fingers. Often requires pre-treatement before loading."
            }
            SoilType::Peat => {
                "Soft soil. Extremely  low bearing capacity, high compressibility, low shear strenght, and high organic and water content. Not suitable for loading without pre-treatment."
            }
            SoilType::Custom { capacity_kpa: _ } => "Custom Bearing Capacity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundConfiguration {
    pub support_points: Vec<SupportPoint>,
    pub soil_type: SoilType,
    pub safety_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearingPressure {
    pub support_index: usize,
    pub pressure_kpa: f32,
    pub allowable_kpa: f32,
    pub is_safe: bool,
    pub utilization_percent: f32,
}

/// Configuration of a crane mat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraneMat {
    pub material: MatMaterial,
    pub length_m: f32,
    pub width_m: f32,
    pub thickness_m: f32,
    pub weight_kg: f32,
    pub stacked_count: usize, // Can stack multiple mats
}

/// Outrigger pad configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutriggerPad {
    pub diameter_m: Option<f32>, // for circular pads
    pub width_m: Option<f32>,    // For rectangular pads
    pub length_m: Option<f32>,   // for rectangular pads
    pub mat_configuration: Option<Vec<CraneMat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundBearingAnalysis {
    pub is_safe: bool,
    pub soil_type: SoilType,
    pub bearing_pressures: Vec<BearingPressure>,
}

pub struct GroundBearingCalculator;

impl GroundBearingCalculator {
    pub fn analyze(config: &GroundConfiguration) -> Result<GroundBearingAnalysis, String> {
        if config.support_points.is_empty() {
            return Err("No support points provided".to_string());
        }

        if config.safety_factor < 1.0 {
            return Err("Safety factor must be >= 1.0".to_string());
        }

        let allowable_pressure =
            config.soil_type.allowable_bearing_capacity_kpa() / config.safety_factor;

        let mut bearing_pressures = Vec::new();
        let mut all_safe = true;

        for (i, support_point) in config.support_points.iter().enumerate() {
            if support_point.contact_area_m2() <= 0.0 {
                return Err(format!("Support point {} has invalid contact area", i));
            }

            let load_kn = support_point.load_kg * 9.81 / 1000.0;
            let pressure_kpa = load_kn / support_point.contact_area_m2();

            let is_safe = pressure_kpa <= allowable_pressure;
            if !is_safe {
                all_safe = false;
            }

            let utilization_percent = (pressure_kpa / allowable_pressure) * 100.0;

            bearing_pressures.push(BearingPressure {
                support_index: i,
                pressure_kpa,
                allowable_kpa: allowable_pressure,
                is_safe,
                utilization_percent,
            });
        }

        Ok(GroundBearingAnalysis {
            is_safe: all_safe,
            soil_type: config.soil_type,
            bearing_pressures,
        })
    }
}
