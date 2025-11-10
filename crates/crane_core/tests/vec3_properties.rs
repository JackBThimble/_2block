use crane_core::math::Vec3;
use proptest::prelude::*;

// Helper to generate valid f64 values (no NaN, no Inf)
fn valid_f64() -> impl Strategy<Value = f64> {
    (-1000.0..1000.0f64).prop_filter("Must be finite", |x| x.is_finite())
}

fn valid_vec3() -> impl Strategy<Value = Vec3> {
    (valid_f64(), valid_f64(), valid_f64())
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

proptest! {
    // ========================================================================
    // ADDITION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_add_commutative(a in valid_vec3(), b in valid_vec3()) {
        let sum1 = a + b;
        let sum2 = b + a;
        prop_assert!(sum1.approx_eq(sum2, 1e-10));
    }
    
    #[test]
    fn prop_add_associative(a in valid_vec3(), b in valid_vec3(), c in valid_vec3()) {
        let sum1 = (a + b) + c;
        let sum2 = a + (b + c);
        prop_assert!(sum1.approx_eq(sum2, 1e-9));
    }
    
    #[test]
    fn prop_add_identity(v in valid_vec3()) {
        let result = v + Vec3::ZERO;
        prop_assert!(result.approx_eq(v, 1e-10));
    }
    
    // ========================================================================
    // MULTIPLICATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_scalar_mul_distributive(v in valid_vec3(), a in valid_f64(), b in valid_f64()) {
        let left = v * (a + b);
        let right = v * a + v * b;
        prop_assert!(left.approx_eq(right, 1e-9));
    }
    
    #[test]
    fn prop_scalar_mul_associative(v in valid_vec3(), a in valid_f64(), b in valid_f64()) {
        let left = (v * a) * b;
        let right = v * (a * b);
        prop_assert!(left.approx_eq(right, 1e-9));
    }
    
    // ========================================================================
    // DOT PRODUCT PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_dot_commutative(a in valid_vec3(), b in valid_vec3()) {
        let dot1 = a.dot(b);
        let dot2 = b.dot(a);
        prop_assert!((dot1 - dot2).abs() < 1e-10);
    }
    
    #[test]
    fn prop_dot_distributive(a in valid_vec3(), b in valid_vec3(), c in valid_vec3()) {
        let left = a.dot(b + c);
        let right = a.dot(b) + a.dot(c);
        prop_assert!((left - right).abs() < 1e-9);
    }
    
    #[test]
    fn prop_dot_self_equals_length_squared(v in valid_vec3()) {
        let dot = v.dot(v);
        let len_sq = v.length_squared();
        prop_assert!((dot - len_sq).abs() < 1e-10);
    }
    
    // ========================================================================
    // CROSS PRODUCT PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_cross_anti_commutative(a in valid_vec3(), b in valid_vec3()) {
        let cross1 = a.cross(b);
        let cross2 = -(b.cross(a));
        prop_assert!(cross1.approx_eq(cross2, 1e-9));
    }
    
    #[test]
    fn prop_cross_perpendicular(a in valid_vec3(), b in valid_vec3()) {
        if a.is_zero() || b.is_zero() {
            return Ok(());
        }
        
        let cross = a.cross(b);
        if !cross.is_zero() {
            let dot_a = cross.dot(a).abs();
            let dot_b = cross.dot(b).abs();
            prop_assert!(dot_a < 1e-8, "Cross product not perpendicular to a: {}", dot_a);
            prop_assert!(dot_b < 1e-8, "Cross product not perpendicular to b: {}", dot_b);
        }
    }
    
    #[test]
    fn prop_cross_self_is_zero(v in valid_vec3()) {
        let cross = v.cross(v);
        prop_assert!(cross.is_zero());
    }
    
    // ========================================================================
    // NORMALIZATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_normalized_has_unit_length(v in valid_vec3()) {
        if v.is_zero() {
            return Ok(());
        }
        
        let normalized = v.normalized();
        let len = normalized.length();
        prop_assert!((len - 1.0).abs() < 1e-6, "Length {} is not 1.0", len);
    }
    
    #[test]
    fn prop_normalize_preserves_direction(v in valid_vec3()) {
        if v.is_zero() {
            return Ok(());
        }
        
        let normalized = v.normalized();
        // Normalized vector should point in same direction
        let dot = v.dot(normalized);
        prop_assert!(dot > 0.0, "Direction reversed after normalization");
    }
    
    // ========================================================================
    // LERP PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_lerp_at_zero(a in valid_vec3(), b in valid_vec3()) {
        let result = a.lerp(b, 0.0);
        prop_assert!(result.approx_eq(a, 1e-10));
    }
    
    #[test]
    fn prop_lerp_at_one(a in valid_vec3(), b in valid_vec3()) {
        let result = a.lerp(b, 1.0);
        prop_assert!(result.approx_eq(b, 1e-10));
    }
    
    #[test]
    fn prop_lerp_midpoint(a in valid_vec3(), b in valid_vec3()) {
        let mid = a.lerp(b, 0.5);
        let expected = (a + b) * 0.5;
        prop_assert!(mid.approx_eq(expected, 1e-9));
    }
    
    // ========================================================================
    // DISTANCE PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_distance_symmetric(a in valid_vec3(), b in valid_vec3()) {
        let dist1 = a.distance(b);
        let dist2 = b.distance(a);
        prop_assert!((dist1 - dist2).abs() < 1e-10);
    }
    
    #[test]
    fn prop_distance_self_is_zero(v in valid_vec3()) {
        let dist = v.distance(v);
        prop_assert!(dist < 1e-10);
    }
    
    #[test]
    fn prop_triangle_inequality(a in valid_vec3(), b in valid_vec3(), c in valid_vec3()) {
        let ab = a.distance(b);
        let bc = b.distance(c);
        let ac = a.distance(c);
        // Triangle inequality: d(a,c) <= d(a,b) + d(b,c)
        prop_assert!(ac <= ab + bc + 1e-9);
    }
}
