use crane_core::math::{Vec3, Quaternion, Transform};
use approx::assert_relative_eq;

const EPSILON: f64 = 1e-6;

#[test]
fn test_crane_boom_kinematics() {
    // Simulate simple crane boom rotation
    // Base at origin, boom 10m long at 45°
    
    let base = Transform::IDENTITY;
    let boom_angle = std::f64::consts::FRAC_PI_4; // 45°
    let boom_length = 10.0;
    
    // Boom rotates around X axis (elevation)
    let boom_rotation = Quaternion::from_axis_angle(Vec3::X, boom_angle);
    let boom_transform = Transform::new(
        Vec3::ZERO,
        boom_rotation,
        1.0,
    );
    
    // Boom tip in local space (pointing in +Z)
    let boom_tip_local = Vec3::new(0.0, 0.0, boom_length);
    
    // Transform to world space
    let boom_tip_world = boom_transform.transform_point(boom_tip_local);
    
    // At 45°, Z and Y components should be equal
    let expected_y = boom_length * (boom_angle.sin());
    let expected_z = boom_length * (boom_angle.cos());
    
    assert_relative_eq!(boom_tip_world.y, expected_y, epsilon = EPSILON);
    assert_relative_eq!(boom_tip_world.z, expected_z, epsilon = EPSILON);
    assert_relative_eq!(boom_tip_world.x, 0.0, epsilon = EPSILON);
}

#[test]
fn test_crane_swing_and_boom() {
    // Crane with swing (rotation around Y) and boom angle
    
    let swing_angle = std::f64::consts::FRAC_PI_2; // 90° swing
    let boom_angle = std::f64::consts::FRAC_PI_4;  // 45° boom
    let boom_length = 20.0;
    
    // Swing rotation (around Y axis)
    let swing = Transform::from_rotation(
        Quaternion::from_axis_angle(Vec3::Y, swing_angle)
    );
    
    // Boom rotation (around X axis)
    let boom = Transform::from_rotation(
        Quaternion::from_axis_angle(Vec3::X, boom_angle)
    );
    
    // Combined: swing first, then boom
    let combined = swing.combine(boom);
    
    // Boom tip in local space
    let boom_tip_local = Vec3::new(0.0, 0.0, boom_length);
    let boom_tip = combined.transform_point(boom_tip_local);
    
    // After 90° swing, boom points in -X direction
    // At 45° elevation, should be diagonal in X-Y plane
    let height = boom_length * boom_angle.sin();
    let horizontal = boom_length * boom_angle.cos();
    
    assert_relative_eq!(boom_tip.x, -horizontal, epsilon = EPSILON);
    assert_relative_eq!(boom_tip.y, height, epsilon = EPSILON);
    assert_relative_eq!(boom_tip.z, 0.0, epsilon = EPSILON);
}

#[test]
fn test_hierarchical_transforms() {
    // Test parent-child transform hierarchy
    // Parent at (10, 0, 0), child at (0, 5, 0) relative to parent
    
    let parent = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
    let child_local = Transform::from_position(Vec3::new(0.0, 5.0, 0.0));
    
    // Child in world space
    let child_world = parent.combine(child_local);
    
    assert_relative_eq!(child_world.position.x, 10.0, epsilon = EPSILON);
    assert_relative_eq!(child_world.position.y, 5.0, epsilon = EPSILON);
    assert_relative_eq!(child_world.position.z, 0.0, epsilon = EPSILON);
    
    // Now rotate parent 90° around Y
    let parent_rotated = Transform::new(
        Vec3::new(10.0, 0.0, 0.0),
        Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2),
        1.0,
    );
    
    let child_world_rotated = parent_rotated.combine(child_local);
    
    // Child should now be at (10, 5, 0) (rotation doesn't affect translation in this case)
    // Actually, the child's local offset gets rotated too
    // Local offset (0, 5, 0) rotated 90° around Y stays at (0, 5, 0)
    assert_relative_eq!(child_world_rotated.position.x, 10.0, epsilon = EPSILON);
    assert_relative_eq!(child_world_rotated.position.y, 5.0, epsilon = EPSILON);
    assert_relative_eq!(child_world_rotated.position.z, 0.0, epsilon = EPSILON);
}

#[test]
fn test_look_at_rotation() {
    // Test that look_at creates correct rotation
    
    let eye = Vec3::new(0.0, 0.0, 0.0);
    let target = Vec3::new(10.0, 0.0, 0.0);
    let up = Vec3::Y;
    
    let rotation = Quaternion::look_at(eye, target, up);
    
    // Forward direction should point toward target
    let forward = rotation.rotate_vector(Vec3::FORWARD);
    let to_target = (target - eye).normalized();
    
    assert!(forward.approx_eq(to_target, EPSILON));
}

#[test]
fn test_rotation_composition() {
    // Test that multiple rotations compose correctly
    
    // Rotate 45° around Y, then 45° around resulting X axis
    let rot1 = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_4);
    let rot2 = Quaternion::from_axis_angle(Vec3::X, std::f64::consts::FRAC_PI_4);
    
    let combined = rot1 * rot2;
    
    // Should be equivalent to applying rot2 first, then rot1
    let v = Vec3::new(1.0, 0.0, 0.0);
    let result1 = combined.rotate_vector(v);
    let result2 = rot1.rotate_vector(rot2.rotate_vector(v));
    
    assert!(result1.approx_eq(result2, EPSILON));
}

#[test]
fn test_inverse_transform_chain() {
    // Test that inverse of combined transforms works correctly
    
    let t1 = Transform::new(
        Vec3::new(5.0, 10.0, 15.0),
        Quaternion::from_axis_angle(Vec3::Y, 0.5),
        2.0,
    );
    
    let t2 = Transform::new(
        Vec3::new(2.0, 3.0, 4.0),
        Quaternion::from_axis_angle(Vec3::X, 0.3),
        1.5,
    );
    
    let combined = t1.combine(t2);
    let inv_combined = combined.inverse();
    
    let point = Vec3::new(7.0, 8.0, 9.0);
    let transformed = combined.transform_point(point);
    let back = inv_combined.transform_point(transformed);
    
    assert!(back.approx_eq(point, EPSILON));
}

#[test]
fn test_orthogonal_axes() {
    // Test that rotation maintains orthogonality of axes
    
    let q = Quaternion::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalized(), 1.23);
    
    let x = q.rotate_vector(Vec3::X);
    let y = q.rotate_vector(Vec3::Y);
    let z = q.rotate_vector(Vec3::Z);
    
    // Should be orthogonal
    assert_relative_eq!(x.dot(y), 0.0, epsilon = EPSILON);
    assert_relative_eq!(y.dot(z), 0.0, epsilon = EPSILON);
    assert_relative_eq!(z.dot(x), 0.0, epsilon = EPSILON);
    
    // Should be unit length
    assert_relative_eq!(x.length(), 1.0, epsilon = EPSILON);
    assert_relative_eq!(y.length(), 1.0, epsilon = EPSILON);
    assert_relative_eq!(z.length(), 1.0, epsilon = EPSILON);
    
    // Should form right-handed system
    assert!(x.cross(y).approx_eq(z, EPSILON));
}
