use core::f32;

use nalgebra as na;
use nalgebra::{Matrix3, Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Type of sling hitch
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HitchType {
    /// Vertical hitch - straight up and down
    Vertical,
    /// Choker hitch - wrapped around load (capacity reduction factor ~0.75)
    Choker,
    /// Basket hitch - sling goes under load and both ends to hook (doubles capacity)
    Basket,
    /// Bridle hitch - multiple slings to a single point
    Bridle,
}

impl HitchType {
    /// Capacity factor relative to vertical hitch
    pub fn capacity_factor(&self) -> f32 {
        match self {
            HitchType::Vertical => 1.0,
            HitchType::Choker => 0.75, // 25% reduction due to bending
            HitchType::Basket => 2.0,  // Double capacity (two legs)
            HitchType::Bridle => 1.0,  // Depends on configuration
        }
    }
}

/// Sling material and specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlingSpec {
    pub id: String,
    pub material: SlingMaterial,
    pub diameter_mm: Option<f32>, // for wire rope
    pub width_mm: Option<f32>,    // for synthetic slings
    pub length_m: f32,
    pub rated_capacity_kg: f32, // Vertical hitch capacity
    pub safety_factor: f32,     // Typically 5:1 for lifting
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlingMaterial {
    WireRope { grade: WireRopeGrade },
    Chain { grade: ChainGrade },
    Synthetic { material: SyntheticMaterial },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WireRopeGrade {
    ImprovedPlowSteel,      // IPS
    ExtraImprovedPlowSteel, // EIPS
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ChainGrade {
    Grade80,
    Grade100,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SyntheticMaterial {
    Nylon,
    Polyester,
    Dyneema,
}

/// A single sling in the rigging configuration
#[derive(Debug, Clone)]
pub struct Sling {
    pub spec: SlingSpec,
    pub hitch_type: HitchType,
    /// Where sling attaches to load (relative to load origin)
    pub attachment_point: Point3<f32>,
    /// Where sling attaches to crane hook (can be same point for multiple slings)
    pub hook_point: Point3<f32>,
    /// Calculated angle from vertical (computed during analysis)
    pub angle_from_vertical: Option<f32>,
    /// Calculated tension in sling (computed during analysis)
    pub tension_kg: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiggingHardware {
    pub hardware_type: HardwareType,
    pub rated_capacity_kg: f32,
    pub weight_kg: f32,
    pub position: Point3<f32>, // position in rigging assembly
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HardwareType {
    Shackle { size_mm: f32 },
    Hook { type_name: String },
    SpreaderBeam { length_m: f32 },
    SpreaderFrame { width_m: f32, length_m: f32 },
    LiftingBeam { length_m: f32, beam_weight_kg: f32 },
    SnatchBlock { sheave_diameter_mm: f32 },
    Swivel,
}

/// The load being lifted
#[derive(Debug, Clone)]
pub struct Load {
    pub weight_kg: f32,
    /// Center of gravity relative to load origin (0, 0, 0)
    pub center_of_gravity: Point3<f32>,
    /// Bounding box for collision detection
    pub dimensions: Vector3<f32>, // length, width, height
    /// Pick points where slings attach
    pub pick_points: Vec<PickPoint>,
}

#[derive(Debug, Clone)]
pub struct PickPoint {
    pub id: String,
    pub position: Point3<f32>,
    pub active: bool, // is this pick point being used?
}

/// Complete rigging configuration
#[derive(Debug, Clone)]
pub struct RiggingConfiguration {
    pub load: Load,
    pub slings: Vec<Sling>,
    pub hardware: Vec<RiggingHardware>,
    pub crane_hook_position: Point3<f32>,
}

/// Result of rigging analysis
#[derive(Debug, Clone)]
pub struct RiggingAnalysis {
    pub sling_tensions: Vec<SlingTensionAnalysis>,
    pub total_rigging_weight_kg: f32,
    pub is_balanced: bool,
    pub titl_angle_deg: Option<Vector3<f32>>,
    pub safety_analysis: SafetyAnalysis,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SlingTensionAnalysis {
    pub sling_id: String,
    pub tension_kg: f32,
    pub tension_kn: f32,
    pub angle_from_vertical_deg: f32,
    pub capacity_kg: f32,
    pub utilization_percent: f32,
    pub is_safe: bool,
}

#[derive(Debug)]
pub struct SpreaderBeamAnalysis {
    pub max_bending_moment_nm: f32,
    pub max_shear_force_n: f32,
    pub required_section_modulus_m3: f32,
}

#[derive(Debug, Clone)]
pub struct SafetyAnalysis {
    pub overall_safety_factor: f32,
    pub critical_sling_id: Option<String>,
    pub is_configuration_safe: bool,
    pub cog_offset_from_hook_m: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct DynamicFactors {
    pub impact_loading: bool,
    pub wind_speed_ms: f32,
}

/// Errors in rigging calculations
#[derive(Debug, Clone)]
pub enum RiggingError {
    InsufficientPickPoints,
    SlingOverloaded {
        sling_id: String,
        utilization_percent: f32,
    },
    UnbalancedLoad {
        tilt_angle_deg: f32,
    },
    InvalidConfiguration(String),
    MathError(String),
}

pub struct RiggingCalculator;

impl RiggingCalculator {
    /// Analyze a complete rigging configuration
    pub fn analyze(config: &RiggingConfiguration) -> Result<RiggingAnalysis, RiggingError> {
        // Calculate sling angles and tensions
        let sling_tensions = Self::calculate_sling_tensions(
            &config.load,
            &config.slings,
            config.crane_hook_position,
        )?;

        let (is_balanced, tilt_angle) =
            Self::check_balance(&config.load, &config.slings, &sling_tensions);

        let total_rigging_weight = Self::calculate_rigging_weight(&config.slings, &config.hardware);

        let safety_analysis = Self::analyze_safety(
            &config.load,
            &config.slings,
            &sling_tensions,
            config.crane_hook_position,
        )?;

        // generate warnings
        let warnings = Self::generate_warnings(&sling_tensions, &safety_analysis);

        Ok(RiggingAnalysis {
            sling_tensions,
            total_rigging_weight_kg: total_rigging_weight,
            is_balanced,
            titl_angle_deg: tilt_angle,
            safety_analysis,
            warnings,
        })
    }

    /// Calculate tension in each sling using static equilibrium
    fn calculate_sling_tensions(
        load: &Load,
        slings: &[Sling],
        hook_position: Point3<f32>,
    ) -> Result<Vec<SlingTensionAnalysis>, RiggingError> {
        if slings.is_empty() {
            return Err(RiggingError::InsufficientPickPoints);
        }

        const G: f32 = 9.81;
        let _load_force = load.weight_kg * G;

        let mut analyses = Vec::new();

        match slings.len() {
            1 => {
                let tension_kg = load.weight_kg;
                analyses.push(Self::analyze_single_sling(&slings[0], tension_kg)?);
            }
            2 => {
                let tensions = Self::solve_two_sling_system(load, slings, hook_position)?;
                for (sling, tension_kg) in slings.iter().zip(tensions.iter()) {
                    analyses.push(Self::analyze_single_sling(sling, *tension_kg)?);
                }
            }
            3 => {
                let tensions = Self::solve_three_sling_system(load, slings, hook_position)?;
                for (sling, tension_kg) in slings.iter().zip(tensions.iter()) {
                    analyses.push(Self::analyze_single_sling(sling, *tension_kg)?);
                }
            }
            4..=6 => {
                // 4+ slings - statically indeterminate, use least squares
                let tensions = Self::solve_multi_sling_system(load, slings, hook_position)?;
                for (sling, tension_kg) in slings.iter().zip(tensions.iter()) {
                    analyses.push(Self::analyze_single_sling(sling, *tension_kg)?);
                }
            }
            _ => {
                return Err(RiggingError::InvalidConfiguration(
                    "Too many slings (max 6 supported)".to_string(),
                ));
            }
        }
        Ok(analyses)
    }

    /// Analyze a single sling
    fn analyze_single_sling(
        sling: &Sling,
        tension_kg: f32,
    ) -> Result<SlingTensionAnalysis, RiggingError> {
        let sling_vector = sling.hook_point - sling.attachment_point;
        let sling_length = sling_vector.norm();

        if sling_length < 0.001 {
            return Err(RiggingError::InvalidConfiguration(
                "Sling length too short".to_string(),
            ));
        }

        let vertical = Vector3::new(0.0, 0.0, 0.1);
        let sling_unit = sling_vector.normalize();
        let cos_angle = sling_unit.dot(&vertical);
        let angle_rad = cos_angle.acos();
        let angle_deg = angle_rad.to_degrees();

        let angle_factor = Self::angle_capacity_factor(angle_deg);
        let hitch_factor = sling.hitch_type.capacity_factor();
        let capacity_kg = sling.spec.rated_capacity_kg * angle_factor * hitch_factor;

        let tension_kn = tension_kg * 9.81 / 1000.0;
        let utilization_percent = (tension_kg / capacity_kg) * 100.0;
        let is_safe = utilization_percent <= 100.0;

        // TODO: Generate warning if angle is too shallow
        if angle_deg > 60.0 {
            // Angle is more than 60 deg from vertical (less than 30 deg from horizontal)
            // Dangerous!
        }

        Ok(SlingTensionAnalysis {
            sling_id: sling.spec.id.clone(),
            tension_kg,
            tension_kn,
            angle_from_vertical_deg: angle_deg,
            capacity_kg,
            utilization_percent,
            is_safe,
        })
    }

    /// Calculate capacity reduction factor based on sling angle
    /// At 0 deg (vertical): factor = 1.0
    /// At 45 deg: factor = 0.7
    /// At 60 deg: factor = 0.5
    fn angle_capacity_factor(angle_from_vertical_deg: f32) -> f32 {
        let angle_rad = angle_from_vertical_deg.to_radians();

        // Tension increases as: T = W / (n * cos(theta))
        // Where n is number of slings, theta is angle from vertical
        // So capacity factor is cos(theta)
        angle_rad.cos()
    }

    /// Solve for two-sling system
    fn solve_two_sling_system(
        load: &Load,
        slings: &[Sling],
        _hook_position: Point3<f32>,
    ) -> Result<Vec<f32>, RiggingError> {
        // For two slings, we have:
        // T1 * T2 = W (vertical equilibrium)
        // T1 * r1 = T2 * r2 (moment equilibrium about CoG)

        let cog = load.center_of_gravity;
        let weight = load.weight_kg;

        // calculate moment arms
        let r1 = (slings[0].attachment_point - cog).norm();
        let r2 = (slings[1].attachment_point - cog).norm();

        if r1 + r2 < 0.001 {
            return Err(RiggingError::InvalidConfiguration(
                "Pick points too close to center of gravity".to_string(),
            ));
        }

        // calculate tensions using moment equilibrium
        // T1 * r1 = T2 * r2
        // T1 + T2 = W
        // Solving: T1 = W * r2 / (r1 + r2)
        //          T2 = W * r1 / (r1 + r2)

        let t1 = weight * r2 / (r1 + r2);
        let t2 = weight * r1 / (r1 + r2);

        // Adjust for sling angles
        let angle1 = Self::calculate_sling_angle(&slings[0]);
        let angle2 = Self::calculate_sling_angle(&slings[1]);

        let t1_adjusted = t1 / angle1.to_radians().cos();
        let t2_adjusted = t2 / angle2.to_radians().cos();

        Ok(vec![t1_adjusted, t2_adjusted])
    }

    /// Solve for three-sling system (statically determinate)
    fn solve_three_sling_system(
        load: &Load,
        slings: &[Sling],
        _hook_position: Point3<f32>,
    ) -> Result<Vec<f32>, RiggingError> {
        // Three slings create a statically determinate system
        // we need to solve 3 equilibrium equations
        // EFx = 0, EFy = 0, EFz = 0

        const G: f32 = 9.81;
        let load_force = Vector3::new(0.0, 0.0, -load.weight_kg * G);

        // Build matrix of sling unit vectors
        let mut a_matrix = Matrix3::zeros();

        for (i, sling) in slings.iter().enumerate() {
            let sling_vec = (sling.hook_point - sling.attachment_point).normalize();
            a_matrix.set_column(i, &sling_vec);
        }

        // check if a matrix is invertible
        if a_matrix.determinant().abs() < 1e-6 {
            return Err(RiggingError::InvalidConfiguration(
                "Slings are coplanar or collinear - cannot solve".to_string(),
            ));
        }

        // solve A * T = -F for tensions T
        let tensions_n = match a_matrix.try_inverse() {
            Some(inv) => inv * (-load_force),
            None => {
                return Err(RiggingError::MathError(
                    "Cannot invert sling geometry matrix".to_string(),
                ));
            }
        };

        // convert from Newtons to kg
        let tensions_kg: Vec<f32> = tensions_n.iter().map(|&t_n| (t_n / G).abs()).collect();

        // check for negative tensions (means load wants to push, not pull)
        if tensions_kg.iter().any(|&t| t < 0.0) {
            return Err(RiggingError::InvalidConfiguration(
                "Configuration produces negative tension - check pick point locations".to_string(),
            ));
        }

        Ok(tensions_kg)
    }

    /// solve for multi-sling system (4+ slings, statically indeterminate)
    fn solve_multi_sling_system(
        load: &Load,
        slings: &[Sling],
        _hook_position: Point3<f32>,
    ) -> Result<Vec<f32>, RiggingError> {
        // For 4+ slings, the system is statically indeterminate
        // we use least squares to find the "best" tension distribution
        // that minimizes the sum of squared tensions (most even distribution)

        const G: f32 = 9.81;
        let n_slings = slings.len();

        // build a matrix of sling unit vectors ( 3 x n )
        let mut a_matrix = na::DMatrix::zeros(3, n_slings);

        for (i, sling) in slings.iter().enumerate() {
            let sling_vec = (sling.hook_point - sling.attachment_point).normalize();
            a_matrix[(0, i)] = sling_vec.x;
            a_matrix[(1, i)] = sling_vec.y;
            a_matrix[(2, i)] = sling_vec.z;
        }

        // load vector
        let load_vec = na::DVector::from_vec(vec![0.0, 0.0, -load.weight_kg * G]);

        // Solve using lease squares: minimize ||A*T - b||^2
        // Solution: T = (A^T * A)^-1 * A^T * b

        let at_a = a_matrix.transpose() * &a_matrix;
        let at_b = a_matrix.transpose() * load_vec;

        // use pseudo-inverse for stability
        let svd = at_a.svd(true, true);
        let tensions_n = svd
            .solve(&at_b, 1e-6)
            .map_err(|_| RiggingError::MathError("SVD solve failed".to_string()))?;

        let tensions_kg: Vec<f32> = tensions_n.iter().map(|&t_n| (t_n / G).abs()).collect();

        Ok(tensions_kg)
    }

    /// Calculate angle of sling from vertical
    fn calculate_sling_angle(sling: &Sling) -> f32 {
        let sling_vector = sling.hook_point - sling.attachment_point;
        let vertical = Vector3::new(0.0, 0.0, 1.0);
        let sling_unit = sling_vector.normalize();
        let cos_angle = sling_unit.dot(&vertical);
        let angle_rad = cos_angle.acos();
        angle_rad.to_degrees()
    }

    /// Check if load is balanced (COG under hook)
    fn check_balance(
        load: &Load,
        slings: &[Sling],
        tensions: &[SlingTensionAnalysis],
    ) -> (bool, Option<Vector3<f32>>) {
        // Calculate effective center of lift based on tension distribution
        let total_tension: f32 = tensions.iter().map(|t| t.tension_kg).sum();

        if total_tension < 0.001 {
            return (false, None);
        }

        let mut weighted_center = Point3::origin();

        for (sling, tension) in slings.iter().zip(tensions.iter()) {
            let weight = tension.tension_kg / total_tension;
            weighted_center += sling.attachment_point.coords * weight;
        }

        // calculate offset from CoG
        let offset = weighted_center - load.center_of_gravity;
        let offset_magnitude = offset.norm();

        // if offset is small (< 5cm), consider it balanced
        let is_balanced = offset_magnitude < 0.05;

        // calculate tilt angle if unbalanced
        let titl_angle = if !is_balanced {
            let tilt_x = (offset.x / load.dimensions.z).atan().to_degrees();
            let tilt_y = (offset.y / load.dimensions.z).atan().to_degrees();
            Some(Vector3::new(tilt_x, tilt_y, 0.0))
        } else {
            None
        };

        (is_balanced, titl_angle)
    }

    /// Calculate total weight of rigging
    fn calculate_rigging_weight(slings: &[Sling], hardware: &[RiggingHardware]) -> f32 {
        let sling_weight: f32 = slings
            .iter()
            .map(|s| Self::estimate_sling_weight(&s.spec))
            .sum();

        let hardware_weight: f32 = hardware.iter().map(|h| h.weight_kg).sum();

        sling_weight + hardware_weight
    }

    /// Estimate sling weight based on spec
    fn estimate_sling_weight(spec: &SlingSpec) -> f32 {
        match &spec.material {
            SlingMaterial::WireRope { .. } => {
                // Approximate: 1m of wire rope weighs ~0.5kg per mm of diameter
                let dia = spec.diameter_mm.unwrap_or(0.0);
                spec.length_m * dia * 0.5
            }
            SlingMaterial::Chain { .. } => {
                // chain is heavier, ~1kg per mm diameter per meter
                let dia = spec.diameter_mm.unwrap_or(0.0);
                spec.length_m * dia * 1.0
            }
            SlingMaterial::Synthetic { .. } => {
                // much lighter, ~ 0.1kg per cm of width per meter
                let width_cm = spec.width_mm.unwrap_or(0.0) / 10.0;
                spec.length_m * width_cm * 0.1
            }
        }
    }

    /// Perform safety analysis
    fn analyze_safety(
        load: &Load,
        _slings: &[Sling],
        tensions: &[SlingTensionAnalysis],
        hook_position: Point3<f32>,
    ) -> Result<SafetyAnalysis, RiggingError> {
        // Find minimum safety factor
        let mut min_safety_factor = f32::INFINITY;
        let mut critical_sling_id = None;

        for tension in tensions {
            let sf = tension.capacity_kg / tension.tension_kg;
            if sf < min_safety_factor {
                min_safety_factor = sf;
                critical_sling_id = Some(tension.sling_id.clone());
            }
        }

        let is_safe = min_safety_factor >= 5.0; // Typical 5:1 safety factor required

        let cog_offset = hook_position - load.center_of_gravity;

        Ok(SafetyAnalysis {
            overall_safety_factor: min_safety_factor,
            critical_sling_id,
            is_configuration_safe: is_safe,
            cog_offset_from_hook_m: cog_offset,
        })
    }

    /// Generate warnings for user
    fn generate_warnings(
        tensions: &[SlingTensionAnalysis],
        safety: &SafetyAnalysis,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // check for shallow angles ( > 60 deg from vertical = < 30 deg from horizontal)
        for tension in tensions {
            if tension.angle_from_vertical_deg > 60.0 {
                warnings.push(format!(
                "Sling '{}' has shallow angle ({:.1}° from vertical). Angles > 60° significantly increase tension.",
                tension.sling_id,
                tension.angle_from_vertical_deg
            ));
            }

            if tension.utilization_percent > 90.0 {
                warnings.push(format!(
                    "Sling '{}' is highly loaded ({:.0}% of capacity)",
                    tension.sling_id, tension.utilization_percent
                ));
            }

            if !tension.is_safe {
                warnings.push(format!(
                    "Sling '{}' is OVERLOADED ({:.0}% of capacity)!",
                    tension.sling_id, tension.utilization_percent
                ));
            }
        }

        if safety.overall_safety_factor < 5.0 {
            warnings.push(format!(
                "Safety factor ({:.1}:1) is below minimum required (5:1)",
                safety.overall_safety_factor
            ));
        }

        let cog_offset_magnitude = safety.cog_offset_from_hook_m.norm();
        if cog_offset_magnitude > 0.2 {
            warnings.push(format!(
                "Center of gravity is offset {:.1}m from hook - load may swing during lift",
                cog_offset_magnitude
            ));
        }

        warnings
    }

    pub fn analyze_spreader_beam(
        beam_length_m: f32,
        beam_weight_kg: f32,
        _lift_point_spacing_m: f32,
        load_kg: f32,
    ) -> SpreaderBeamAnalysis {
        // Beam acts as simply supported beam
        // calculate bending moment and shear
        // TODO: Implement spreader beam calculations

        let total_load_n = (load_kg + beam_weight_kg) * 9.81;
        let max_moment_nm = (total_load_n * beam_length_m) / 8.0;
        let max_shear_n = total_load_n / 2.0;

        SpreaderBeamAnalysis {
            max_bending_moment_nm: max_moment_nm,
            max_shear_force_n: max_shear_n,
            required_section_modulus_m3: max_moment_nm / 250e6, // assuming steel
        }
    }

    pub fn apply_dynamic_factors(static_tension_kg: f32, factors: DynamicFactors) -> f32 {
        let mut multiplier = 1.0;

        if factors.impact_loading {
            multiplier *= 1.25;
        }

        if factors.wind_speed_ms > 5.0 {
            multiplier *= 1.0 + (factors.wind_speed_ms / 50.0);
        }

        static_tension_kg * multiplier
    }
}

/// Helper functions for rigging design
pub struct RiggingDesigner;

impl RiggingDesigner {
    /// Suggest optimal pick points for load
    pub fn suggest_pick_points(load: &Load, num_points: usize) -> Vec<Point3<f32>> {
        match num_points {
            2 => Self::suggest_two_point_lift(load),
            4 => Self::suggest_four_point_lift(load),
            _ => vec![],
        }
    }

    /// Suggest two pick points (most common)
    fn suggest_two_point_lift(load: &Load) -> Vec<Point3<f32>> {
        let cog = load.center_of_gravity;
        let dims = load.dimensions;

        // place pick points equidistant from CoG along longest axis
        let offset = dims.x * 0.4;

        vec![
            Point3::new(cog.x - offset, cog.y, cog.z + dims.z * 0.5),
            Point3::new(cog.x + offset, cog.y, cog.z + dims.z * 0.5),
        ]
    }

    /// Suggest four pick points (for wider loads)
    fn suggest_four_point_lift(load: &Load) -> Vec<Point3<f32>> {
        let cog = load.center_of_gravity;
        let dims = load.dimensions;

        // place at corners, offset inward slightly for safety
        let x_offset = dims.x * 0.35;
        let y_offset = dims.y * 0.35;
        let z_top = cog.z + dims.z * 0.5;

        vec![
            Point3::new(cog.x + x_offset, cog.y + y_offset, z_top), // Front right
            Point3::new(cog.x + x_offset, cog.y - y_offset, z_top), // Front left
            Point3::new(cog.x - x_offset, cog.y + y_offset, z_top), // Rear right
            Point3::new(cog.x - x_offset, cog.y - y_offset, z_top), // Rear left
        ]
    }

    /// Calculate required sling capacity for a lift
    pub fn required_sling_capacity(
        load_weight_kg: f32,
        num_slings: usize,
        max_angle_from_vertical_deg: f32,
        hitch_type: HitchType,
    ) -> f32 {
        if num_slings == 0 {
            return 0.0;
        }

        // base load per sling
        let load_per_sling = load_weight_kg / num_slings as f32;

        let angle_rad = max_angle_from_vertical_deg.to_radians();
        let angle_factor = angle_rad.cos();

        let hitch_factor = hitch_type.capacity_factor();

        let required_capacity = load_per_sling / (angle_factor * hitch_factor);

        // Add 20% margin for
        required_capacity * 1.2
    }
}
