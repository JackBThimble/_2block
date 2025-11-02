// crates/scene_3d/src/coordinate_conversion.rs - NEW FILE

use bevy::prelude::*;
use nalgebra::Point3;

/// Convert nalgebra Point3 (engineering coords) to Bevy Vec3 (graphics coords)
///
/// Engineering: X=side, Y=forward, Z=up
/// Bevy:        X=side, Y=up,      Z=forward
#[inline]
pub fn nalgebra_to_bevy_point(p: Point3<f32>) -> Vec3 {
    Vec3::new(p.x, p.z, p.y)
}

/// Convert Bevy Vec3 (graphics coords) to nalgebra Point3 (engineering coords)
#[inline]
pub fn bevy_to_nalgebra_point(v: Vec3) -> Point3<f32> {
    Point3::new(v.x, v.z, v.y)
}

/// Convert nalgebra Vector3 (engineering) to Bevy Vec3 (graphics)
#[inline]
pub fn nalgebra_to_bevy_vector(v: nalgebra::Vector3<f32>) -> Vec3 {
    Vec3::new(v.x, v.z, v.y)
}

/// Convert Bevy Vec3 (graphics) to nalgebra Vector3 (engineering)
#[inline]
pub fn bevy_to_nalgebra_vector(v: Vec3) -> nalgebra::Vector3<f32> {
    nalgebra::Vector3::new(v.x, v.z, v.y)
}

// Extension trait for convenience
pub trait CoordinateConversion {
    type Output;
    fn to_bevy(self) -> Self::Output;
    fn to_nalgebra(self) -> Self::Output;
}

impl CoordinateConversion for Point3<f32> {
    type Output = Vec3;

    #[inline]
    fn to_bevy(self) -> Vec3 {
        nalgebra_to_bevy_point(self)
    }

    #[inline]
    fn to_nalgebra(self) -> Vec3 {
        nalgebra_to_bevy_point(self)
    }
}

impl CoordinateConversion for Vec3 {
    type Output = Point3<f32>;

    #[inline]
    fn to_nalgebra(self) -> Point3<f32> {
        bevy_to_nalgebra_point(self)
    }

    #[inline]
    fn to_bevy(self) -> Point3<f32> {
        bevy_to_nalgebra_point(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        let nalg = Point3::new(1.0, 2.0, 3.0); // x=1, y_forward=2, z_up=3
        let bevy = nalgebra_to_bevy_point(nalg);

        assert_eq!(bevy.x, 1.0); // X stays same
        assert_eq!(bevy.y, 3.0); // Z becomes Y (up)
        assert_eq!(bevy.z, 2.0); // Y becomes Z (forward)

        // Round trip
        let back = bevy_to_nalgebra_point(bevy);
        assert_eq!(back.x, nalg.x);
        assert_eq!(back.y, nalg.y);
        assert_eq!(back.z, nalg.z);
    }

    #[test]
    fn test_extension_trait() {
        let nalg = Point3::new(1.0, 2.0, 3.0);
        let bevy = nalg.to_bevy();
        let back = bevy.to_nalgebra();

        assert_eq!(back.x, nalg.x);
        assert_eq!(back.y, nalg.y);
        assert_eq!(back.z, nalg.z);
    }
}
