
use crane_core::math::{Transform, Vec3, Quaternion};
use proptest::prelude::*;

fn valid_f64() -> impl Strategy<Value = f64> {
    (-1000.0..1000.0f64).prop_filter("Must be finite", |x| x.is_finite())
}

fn valid_vec3() -> impl Strategy<Value = Vec3> {
    (valid_f64(), valid_f64(), valid_f64())
        .prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

fn valid_unit_vec3() -> impl Strategy<Value = Vec3> {
    valid_vec3()
        .prop_filter("Must be non-zero", |v| !v.is_zero())
        .prop_map(|v| v.normalized())
}

fn valid_angle() -> impl Strategy<Value = f64> {
    (-std::f64::consts::PI..std::f64::consts::PI)
}

fn valid_quaternion() -> impl Strategy<Value = Quaternion> {
    (valid_unit_vec3(), valid_angle())
        .prop_map(|(axis, angle)| Quaternion::from_axis_angle(axis, angle))
}

fn valid_scale() -> impl Strategy<Value = f64> {
    (0.1..10.0f64)
}

fn valid_transform() -> impl Strategy<Value = Transform> {
    (valid_vec3(), valid_quaternion(), valid_scale())
        .prop_map(|(pos, rot, scale)| Transform::new(pos, rot, scale))
}

proptest! {
    // ========================================================================
    // IDENTITY PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_identity_transform_unchanged(p in valid_vec3()) {
        let result = Transform::IDENTITY.transform_point(p);
        prop_assert!(result.approx_eq(p, 1e-10));
    }
    
    #[test]
    fn prop_identity_combine(t in valid_transform()) {
        let result = t.combine(Transform::IDENTITY);
        prop_assert!(result.approx_eq(t, 1e-9));
        
        let result2 = Transform::IDENTITY.combine(t);
        prop_assert!(result2.approx_eq(t, 1e-9));
    }
    
    // ========================================================================
    // INVERSE PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_inverse_undoes_transform(t in valid_transform(), p in valid_vec3()) {
        let transformed = t.transform_point(p);
        let back = t.inverse().transform_point(transformed);
        prop_assert!(back.approx_eq(p, 1e-6));
    }
    
    #[test]
    fn prop_inverse_combine_is_identity(t in valid_transform()) {
        let combined = t.combine(t.inverse());
        prop_assert!(combined.is_identity());
    }
    
    #[test]
    fn prop_inverse_inverse_is_original(t in valid_transform()) {
        let inv_inv = t.inverse().inverse();
        prop_assert!(inv_inv.approx_eq(t, 1e-9));
    }
    
    // ========================================================================
    // COMBINE PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_combine_associative(t1 in valid_transform(), t2 in valid_transform(), t3 in valid_transform()) {
        let left = t1.combine(t2).combine(t3);
        let right = t1.combine(t2.combine(t3));
        prop_assert!(left.approx_eq(right, 1e-7));
    }
    
    #[test]
    fn prop_combine_equivalent_to_nested_transform(
        t1 in valid_transform(),
        t2 in valid_transform(),
        p in valid_vec3()
    ) {
        let combined = t1.combine(t2);
        let result1 = combined.transform_point(p);
        let result2 = t1.transform_point(t2.transform_point(p));
        
        prop_assert!(result1.approx_eq(result2, 1e-6));
    }
    
    // ========================================================================
    // TRANSLATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_translation_only_affects_position(offset in valid_vec3(), p in valid_vec3()) {
        let t = Transform::from_position(offset);
        let result = t.transform_point(p);
        let expected = p + offset;
        prop_assert!(result.approx_eq(expected, 1e-10));
    }
    
    #[test]
    fn prop_translate_is_additive(t in valid_transform(), offset in valid_vec3(), p in valid_vec3()) {
        let t2 = t.translate(offset);
        let result = t2.transform_point(p);
        let expected = t.transform_point(p) + offset;
        prop_assert!(result.approx_eq(expected, 1e-9));
    }
    
    // ========================================================================
    // ROTATION PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_rotation_preserves_distance_from_origin(q in valid_quaternion(), p in valid_vec3()) {
        let t = Transform::from_rotation(q);
        let transformed = t.transform_point(p);
        
        let dist_before = p.length();
        let dist_after = transformed.length();
        
        prop_assert!((dist_before - dist_after).abs() < 1e-6);
    }
    
    #[test]
    fn prop_rotation_preserves_distances(q in valid_quaternion(), p1 in valid_vec3(), p2 in valid_vec3()) {
        let t = Transform::from_rotation(q);
        
        let dist_before = p1.distance(p2);
        let t1 = t.transform_point(p1);
        let t2 = t.transform_point(p2);
        let dist_after = t1.distance(t2);
        
        prop_assert!((dist_before - dist_after).abs() < 1e-6);
    }
    
    // ========================================================================
    // SCALE PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_scale_multiplies_distance(scale in valid_scale(), p in valid_vec3()) {
        let t = Transform::from_scale(scale);
        let transformed = t.transform_point(p);
        
        let dist_before = p.length();
        let dist_after = transformed.length();
        
        prop_assert!((dist_after - dist_before * scale).abs() < 1e-6);
    }
    
    #[test]
    fn prop_scale_is_multiplicative(t in valid_transform(), scale in valid_scale()) {
        let t2 = t.mul_scale(scale);
        prop_assert!((t2.scale - t.scale * scale).abs() < 1e-10);
    }
    
    // ========================================================================
    // VECTOR VS POINT TRANSFORM
    // ========================================================================
    
    #[test]
    fn prop_vector_ignores_translation(offset in valid_vec3(), v in valid_vec3()) {
        let t = Transform::from_position(offset);
        let result = t.transform_vector(v);
        prop_assert!(result.approx_eq(v, 1e-10));
    }
    
    #[test]
    fn prop_direction_ignores_scale(scale in valid_scale(), v in valid_vec3()) {
        if v.is_zero() {
            return Ok(());
        }
        
        let t = Transform::from_scale(scale);
        let dir = v.normalized();
        let result = t.transform_direction(dir);
        
        prop_assert!(result.approx_eq(dir, 1e-10));
    }
    
    // ========================================================================
    // LERP PROPERTIES
    // ========================================================================
    
    #[test]
    fn prop_lerp_at_zero(t1 in valid_transform(), t2 in valid_transform()) {
        let result = t1.lerp(t2, 0.0);
        prop_assert!(result.approx_eq(t1, 1e-9));
    }
    
    #[test]
    fn prop_lerp_at_one(t1 in valid_transform(), t2 in valid_transform()) {
        let result = t1.lerp(t2, 1.0);
        prop_assert!(result.approx_eq(t2, 1e-9));
    }
    
    #[test]
    fn prop_lerp_position_is_linear(
        pos1 in valid_vec3(),
        pos2 in valid_vec3(),
        t in 0.0..1.0f64
    ) {
        let t1 = Transform::from_position(pos1);
        let t2 = Transform::from_position(pos2);
        let lerped = t1.lerp(t2, t);
        
        let expected = pos1.lerp(pos2, t);
        prop_assert!(lerped.position.approx_eq(expected, 1e-9));
    }
}
