use crane_core::math::{Quaternion, Vec3};
use proptest::prelude::*;

fn valid_f64() -> impl Strategy<Value = f64> {
    (-1000.0..1000.0f64).prop_filter("Must be finite", |x| x.is_finite())
}

fn valid_vec3() -> impl Strategy<Value = Vec3> {
    (valid_f64(), valid_f64(), valid_f64())
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
        .prop_filter("Must be non-zero", |v| !v.is_zero())
}

fn valid_unit_vec3() -> impl Strategy<Value = Vec3> {
    valid_vec3().prop_map(|v| v.normalized())
}

fn valid_angle() -> impl Strategy<Value = f64> {
    (-std::f64::consts::PI..std::f64::consts::PI)
}

fn valid_quaternion() -> impl Strategy<Value = Quaternion> {
    (valid_unit_vec3(), valid_angle())
        .prop_map(|(axis, angle)| Quaternion::from_axis_angle(axis, angle))
}

proptest! {
    // ========================================================================
    // NORMALIZATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_normalized_is_unit_length(axis in valid_unit_vec3(), angle in valid_angle()) {
        let q = Quaternion::from_axis_angle(axis, angle);
        prop_assert!(q.is_normalized());
    }
    
    #[test]
    fn prop_normalize_idempotent(q in valid_quaternion()) {
        let n1 = q.normalized();
        let n2 = n1.normalized();
        prop_assert!(n1.approx_eq(n2, 1e-10));
    }
    
    // ========================================================================
    // MULTIPLICATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_mul_associative(q1 in valid_quaternion(), q2 in valid_quaternion(), q3 in valid_quaternion()) {
        let left = (q1 * q2) * q3;
        let right = q1 * (q2 * q3);
        prop_assert!(left.approx_eq(right, 1e-9));
    }
    
    #[test]
    fn prop_identity_mul(q in valid_quaternion()) {
        let result = q * Quaternion::IDENTITY;
        prop_assert!(result.approx_eq(q, 1e-10));
        
        let result2 = Quaternion::IDENTITY * q;
        prop_assert!(result2.approx_eq(q, 1e-10));
    }
    
    #[test]
    fn prop_inverse_mul(q in valid_quaternion()) {
        let inv = q.inverse();
        let result = q * inv;
        prop_assert!(result.approx_eq(Quaternion::IDENTITY, 1e-6));
    }
    
    // ========================================================================
    // ROTATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_rotation_preserves_length(q in valid_quaternion(), v in valid_vec3()) {
        let rotated = q.rotate_vector(v);
        let len_before = v.length();
        let len_after = rotated.length();
        prop_assert!((len_before - len_after).abs() < 1e-6);
    }
    
    #[test]
    fn prop_identity_rotation_unchanged(v in valid_vec3()) {
        let rotated = Quaternion::IDENTITY.rotate_vector(v);
        prop_assert!(rotated.approx_eq(v, 1e-10));
    }
    
    #[test]
    fn prop_inverse_rotation_undoes(q in valid_quaternion(), v in valid_vec3()) {
        let rotated = q.rotate_vector(v);
        let back = q.inverse().rotate_vector(rotated);
        prop_assert!(back.approx_eq(v, 1e-6));
    }
    
    #[test]
    fn prop_180_degree_rotation_twice_is_identity(axis in valid_unit_vec3(), v in valid_vec3()) {
        let q = Quaternion::from_axis_angle(axis, std::f64::consts::PI);
        let rotated = q.rotate_vector(v);
        let twice = q.rotate_vector(rotated);
        prop_assert!(twice.approx_eq(v, 1e-6));
    }
    
    // ========================================================================
    // SLERP PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_slerp_at_zero(q1 in valid_quaternion(), q2 in valid_quaternion()) {
        let result = q1.slerp(q2, 0.0);
        prop_assert!(result.approx_eq(q1, 1e-6) || result.approx_eq(-q1, 1e-6));
    }
    
    #[test]
    fn prop_slerp_at_one(q1 in valid_quaternion(), q2 in valid_quaternion()) {
        let result = q1.slerp(q2, 1.0);
        prop_assert!(result.approx_eq(q2, 1e-6) || result.approx_eq(-q2, 1e-6));
    }
    
    #[test]
    fn prop_slerp_result_normalized(q1 in valid_quaternion(), q2 in valid_quaternion(), t in 0.0..1.0f64) {
        let result = q1.slerp(q2, t);
        prop_assert!(result.is_normalized());
    }
    
    // ========================================================================
    // AXIS-ANGLE PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_axis_angle_roundtrip(axis in valid_unit_vec3(), angle in valid_angle()) {
        let q = Quaternion::from_axis_angle(axis, angle);
        let (axis2, angle2) = q.to_axis_angle();
        
        // Reconstruct and compare rotations
        let q2 = Quaternion::from_axis_angle(axis2, angle2);
        prop_assert!(q.approx_eq(q2, 1e-6));
    }
    
    #[test]
    fn prop_zero_angle_is_identity(axis in valid_unit_vec3()) {
        let q = Quaternion::from_axis_angle(axis, 0.0);
        prop_assert!(q.approx_eq(Quaternion::IDENTITY, 1e-10));
    }
    
    // ========================================================================
    // DOT PRODUCT PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_dot_commutative(q1 in valid_quaternion(), q2 in valid_quaternion()) {
        let dot1 = q1.dot(q2);
        let dot2 = q2.dot(q1);
        prop_assert!((dot1 - dot2).abs() < 1e-10);
    }
    
    #[test]
    fn prop_dot_self_equals_length_squared(q in valid_quaternion()) {
        let dot = q.dot(q);
        let len_sq = q.length_squared();
        prop_assert!((dot - len_sq).abs() < 1e-10);
    }
}
