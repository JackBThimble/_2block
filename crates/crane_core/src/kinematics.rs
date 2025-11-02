// crates/crane_core/src/kinematics.rs

use nalgebra::{Point3, Vector3};

/// Calculate hook position from crane geometry
///
/// # Arguments
/// * `crane_base` - Position of crane base in world coordinates
/// * `boom_length_m` - Length of boom in meters
/// * `boom_angle_deg` - Angle of boom from horizontal (0° = horizontal, 90° = vertical)
/// * `swing_angle_deg` - Swing angle (0° = forward, 90° = right, etc.)
/// * `hoist_length_m` - Length of cable below boom tip in meters
/// * `boom_pivot_height_m` - Height of boom pivot above crane base
pub fn calculate_hook_position(
    crane_base: Point3<f32>,
    boom_length_m: f32,
    boom_angle_deg: f32,
    swing_angle_deg: f32,
    boom_pivot_height_m: f32,
    hoist_length_m: f32,
) -> Point3<f32> {
    let boom_tip = calculate_boom_tip_position(
        crane_base,
        boom_length_m,
        boom_angle_deg,
        swing_angle_deg,
        boom_pivot_height_m,
    );

    Point3::new(boom_tip.x, boom_tip.y, boom_tip.z - hoist_length_m)
}

/// Calculate boom tip position from crane geometry
///
/// # Arguments
/// * `crane_base` - Position of crane base in world coordinates
/// * `boom_length_m` - Length of boom in meters
/// * `boom_angle_deg` - Angle of boom from horizontal (0° = horizontal, 90° = vertical)
/// * `swing_angle_deg` - Swing angle (0° = forward, 90° = right, etc.)
/// * `boom_pivot_height_m` - Height of boom pivot above crane base
pub fn calculate_boom_tip_position(
    crane_base: Point3<f32>,
    boom_length_m: f32,
    boom_angle_deg: f32,
    swing_angle_deg: f32,
    boom_pivot_height_m: f32,
) -> Point3<f32> {
    let boom_angle_rad = boom_angle_deg.to_radians();
    let swing_angle_rad = swing_angle_deg.to_radians();

    // Horizontal and vertical components
    let horizontal_reach = boom_length_m * boom_angle_rad.cos();
    let vertical_reach = boom_length_m * boom_angle_rad.sin();

    // Apply swing
    let x = horizontal_reach * swing_angle_rad.sin();
    let y = horizontal_reach * swing_angle_rad.cos();
    let z = boom_pivot_height_m + vertical_reach;

    Point3::new(crane_base.x + x, crane_base.y + y, crane_base.z + z)
}

/// Calculate boom angle needed to reach a specific hook height at given radius
///
/// Returns None if the target is unreachable with given boom length
pub fn calculate_boom_angle_for_height(
    boom_length_m: f32,
    target_radius_m: f32,
    target_hook_height_m: f32,
    boom_pivot_height_m: f32,
    hoist_length_m: f32,
) -> Option<f32> {
    // Target boom tip height = hook height + cable length
    let target_boom_tip_height = target_hook_height_m + hoist_length_m;
    let height_from_pivot = target_boom_tip_height - boom_pivot_height_m;

    // Check if reachable
    let distance_from_pivot = (target_radius_m.powi(2) + height_from_pivot.powi(2)).sqrt();
    if distance_from_pivot > boom_length_m {
        return None;
    }

    // Calculate angle
    let angle_rad = height_from_pivot.atan2(target_radius_m);
    Some(angle_rad.to_degrees())
}

/// Calculate the load path as crane swings
///
/// Returns vector of hook positions during swing from start_angle to end_angle
pub fn calculate_swing_path(
    crane_base: Point3<f32>,
    boom_length_m: f32,
    boom_angle_deg: f32,
    swing_deg_range: (f32, f32),
    boom_pivot_height_m: f32,
    hoist_length_m: f32,
    num_steps: usize,
) -> Vec<Point3<f32>> {
    let mut path = Vec::with_capacity(num_steps);

    for i in 0..num_steps {
        let t = i as f32 / (num_steps - 1) as f32;
        let swing_angle = swing_deg_range.0 + t * (swing_deg_range.1 - swing_deg_range.0);

        let pos = calculate_hook_position(
            crane_base,
            boom_length_m,
            boom_angle_deg,
            swing_angle,
            boom_pivot_height_m,
            hoist_length_m,
        );

        path.push(pos);
    }

    path
}

/// Calculate if load will clear an obstacle during swing
pub fn check_clearance(
    hook_path: &[Point3<f32>],
    load_dimensions: Vector3<f32>,
    obstacle_position: Point3<f32>,
    obstacle_dimensions: Vector3<f32>,
    clearance_margin_m: f32,
) -> bool {
    for hook_pos in hook_path {
        // Load hangs below hook
        let load_center = Point3::new(hook_pos.x, hook_pos.y, hook_pos.z - load_dimensions.z / 2.0);

        let load_half = load_dimensions / 2.0 + Vector3::repeat(clearance_margin_m);
        let obs_half = obstacle_dimensions / 2.0 + Vector3::repeat(clearance_margin_m);

        if (load_center.x - obstacle_position.x).abs() < (load_half.x + obs_half.x)
            && (load_center.y - obstacle_position.y).abs() < (load_half.y + obs_half.y)
            && (load_center.z - obstacle_position.z).abs() < (load_half.z + obs_half.z)
        {
            return false; // Collision detected
        }
    }

    true // Clear path
}

/// Calculate hoist length needed to position hook at specific height
pub fn calculate_hoist_length_for_height(boom_tip_height_m: f32, target_hook_height_m: f32) -> f32 {
    (boom_tip_height_m - target_hook_height_m).max(0.0)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_hook_position_vertical_boom() {
        let hook = calculate_hook_position(
            Point3::origin(),
            30.0,
            90.0, // Vertical
            0.0,
            3.0,
            10.0,
        );

        assert!((hook.x).abs() < 0.1);
        assert!((hook.y).abs() < 0.1);
        assert!((hook.z - 23.0).abs() < 0.1); // 30m boom + 3m pivot
    }

    #[test]
    fn test_hook_position_horizontal_boom() {
        let hook = calculate_hook_position(
            Point3::origin(),
            30.0,
            0.0, // Horizontal
            0.0,
            3.0,
            5.0,
        );

        assert!((hook.x).abs() < 0.1);
        assert!((hook.y - 30.0).abs() < 0.1);
        assert!((hook.z + 2.0).abs() < 0.1);
    }

    #[test]
    fn test_boom_angle_calculation() {
        let angle = calculate_boom_angle_for_height(
            30.0, // 30m boom
            15.0, // 15m radius
            24.0, // 29m height
            3.0,  // 3m pivot
            10.0, // 10m cable
        );

        assert!(angle.is_some());
        let angle = angle.unwrap();
        assert!((angle - 60.0).abs() < 1.0); // Should be ~60 degrees
    }

    #[test]
    fn test_unreachable_target() {
        let angle = calculate_boom_angle_for_height(
            30.0, 50.0, // Too far
            50.0, // Too high
            3.0, 10.0,
        );

        assert!(angle.is_none());
    }

    #[test]
    fn test_swing_path_generation() {
        let path = calculate_swing_path(Point3::origin(), 30.0, 45.0, (0.0, 90.0), 3.0, 10.0, 10);
        assert_eq!(path.len(), 10);

        assert!(path[0].x.abs() < 0.1);
        assert!(path[0].y > 0.0);

        assert!(path[9].x > 0.0);
        assert!(path[9].y.abs() < 0.1);
    }

    #[test]
    fn test_clearance_check_clear() {
        let path = vec![
            Point3::new(0.0, 0.0, 10.0),
            Point3::new(1.0, 0.0, 10.0),
            Point3::new(2.0, 0.0, 10.0),
        ];

        let obstacle = Point3::new(10.0, 10.0, 5.0);
        let load_dims = Vector3::new(2.0, 2.0, 2.0);
        let obs_dims = Vector3::new(3.0, 3.0, 3.0);

        assert!(check_clearance(&path, load_dims, obstacle, obs_dims, 0.5));
    }

    #[test]
    fn test_clearance_check_collision() {
        let path = vec![
            Point3::new(0.0, 0.0, 10.0),
            Point3::new(1.0, 0.0, 10.0),
            Point3::new(2.0, 0.0, 10.0),
        ];

        let obstacle = Point3::new(1.0, 0.0, 8.0);
        let load_dims = Vector3::new(2.0, 2.0, 2.0);
        let obs_dims = Vector3::new(3.0, 3.0, 3.0);

        assert!(check_clearance(&path, load_dims, obstacle, obs_dims, 0.5));
    }

    #[test]
    fn test_hoist_length_calculation() {
        let hoist = calculate_hoist_length_for_height(30.0, 15.0);
        assert!((hoist - 15.0).abs() < 0.01);

        let hoist = calculate_hoist_length_for_height(10.0, 20.0);
        assert!((hoist).abs() < 0.01);
    }
}
