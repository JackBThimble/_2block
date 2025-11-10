mod bevy_conv;
mod quat;
mod transform;
mod vec3;

pub use quat::Quaternion;
pub use transform::Transform;
pub use vec3::Vec3;

// #[cfg(feature = "bevy")]
// mod bevy_conv;


pub mod utils {
    /// Clamp a value between min and max
    #[inline]
    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        value.max(min).min(max)
    }

    /// Linear interpolation
    #[inline]
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    /// Check if two f64 values are approximately equal
    #[inline]
    pub fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    /// Remap value from one range to another
    #[inline]
    pub fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64)  -> f64 {

        let t = (value - from_min) / (from_max - from_min);
        lerp(to_min, to_max, t)
    }

    /// Smoothstep interpolation (cubic hermite)
    #[inline]
    pub fn smoothstep(t: f64) -> f64 {
        let t = clamp(t, 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    #[inline]
    pub fn smootherstep(t: f64) -> f64 {
        let t = clamp(t, 0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utils() {
        assert_eq!(utils::clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(utils::clamp(-1.0, 0.0, 10.0), 0.0);
        assert_eq!(utils::clamp(15.0, 0.0, 10.0), 10.0);
        
        assert_eq!(utils::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(utils::lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(utils::lerp(0.0, 10.0, 1.0), 10.0);
        
        assert!(utils::approx_eq(1.0, 1.0000001, 0.001));
        assert!(!utils::approx_eq(1.0, 1.1, 0.001));
    }
}
