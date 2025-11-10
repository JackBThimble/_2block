use super::{Quaternion, Vec3};

/// Affine transformation with position, rotation, and scale
///
/// Represents a 3D transformation with translation, rotation, and uniform scale.
/// Used for object positioning and hierarchical transformations.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: f64,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        position: Vec3::ZERO,
        rotation: Quaternion::IDENTITY,
        scale: 1.0,
    };

    /// Create a transform with position, rotation, and scale
    #[inline]
    pub const fn new(position: Vec3, rotation: Quaternion, scale: f64) -> Self {
        Self {
            position,
            rotation,
            scale
        }
    }

    /// Create a transform with a position only
    #[inline]
    pub const fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quaternion::IDENTITY,
            scale: 1.0,
        }
    }

    /// Create a transform with rotation only
    #[inline]
    pub const fn from_rotation(rotation: Quaternion) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation,
            scale: 1.0
        }
    }

    /// Create a transform with scale only
    #[inline]
    pub const fn from_scale(scale: f64) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale,
        }
    }

    /// Create a transform from position and rotation
    #[inline]
    pub const fn from_position_rotation(position: Vec3, rotation: Quaternion) -> Self {
        Self {
            position,
            rotation,
            scale: 1.0,
        }
    }

    /// Convert this transform into a 4x4 matrix (scale -> rotation -> transform)
    pub fn to_mat4(self) -> [[f64; 4]; 4] {
        let r = self.rotation.to_mat3();
        let s = self.scale;
        [
            [r[0][0] * s, r[0][1] * s, r[0][2] * s, 0.0],
            [r[1][0] * s, r[1][1] * s, r[1][2] * s, 0.0],
            [r[2][0] * s, r[2][1] * s, r[2][2] * s, 0.0],
            [self.position.x, self.position.y, self.position.z, 1.0],
        ]
    }

    // ========================================================================
    // TRANSFORM OPERATIONS
    // ========================================================================


    /// Transform a point from local space to world space
    /// Applies scale, rotation, and translation
    #[inline]
    pub fn transform_point(self, point: Vec3) -> Vec3 {
        self.rotation.rotate_vector(point * self.scale) + self.position
    }

    /// Transform a vector
    /// Applies scale and rotation, no translation
    #[inline]
    pub fn transform_vector(self, vector: Vec3) -> Vec3 {
        self.rotation.rotate_vector(vector * self.scale)
    }

    /// Transform a direction vector 
    /// Applies rotation only, no scale or translation
    #[inline]
    pub fn transform_direction(self, direction: Vec3) -> Vec3 {
        self.rotation.rotate_vector(direction)
    }

    /// Inverse transform (undo this transformation)
    pub fn inverse(self) -> Self {
        let inv_rotation = self.rotation.conjugate();
        let inv_scale = 1.0 / self.scale;
        let inv_position = inv_rotation.rotate_vector(-self.position) * inv_scale;

        Self {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// Combine two transforms (parent * child)
    ///
    /// Applies `other` first, then `self`
    /// Equivalent to: self.transform_point(other.transform_point(p))
    #[inline]
    pub fn combine(self, other: Self) -> Self {
        Self {
            position: self.rotation.rotate_vector(other.position * self.scale) + self.position,
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    /// Interpolate between two transforms
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale + (other.scale - self.scale) * t,
        }
    }

    /// Get the forward direction (+Z)
    #[inline]
    pub fn forward(self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::FORWARD)
    }

    /// Get the up direction (+Y)
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::UP)
    }

    /// Get the right direction (+X)
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.rotate_vector(Vec3::RIGHT)
    }

    /// Translate by a vector
    #[inline]
    pub fn translate(mut self, translation: Vec3) -> Self {
        self.position += translation;
        self
    }

    /// Rotate by a quaternion
    #[inline]
    pub fn rotate(mut self, rotation: Quaternion) -> Self {
        self.rotation = rotation * self.rotation;
        self
    }

    /// Scale uniformly
    #[inline]
    pub fn mul_scale(mut self, scale: f64) -> Self {
        self.scale *= scale;
        self
    }

    /// Look at a target position
    ///
    /// Rotates this transform to look at target with `up` as the up direction.
    #[inline]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.rotation = Quaternion::look_at(self.position, target, up);
        self
    }

    /// Check if transform is identity
    #[inline]
    pub fn is_identity(self) -> bool {
        self.position.is_zero()
        && self.rotation.approx_eq(Quaternion::IDENTITY, 1e-6)
        && (self.scale - 1.0).abs() < 1e-6
    }

    /// Check if transform is finite
    #[inline]
    pub fn approx_eq(self, other: Self, epsilon: f64) -> bool {
        self.position.approx_eq(other.position, epsilon)
            && self.rotation.approx_eq(other.rotation, epsilon)
            && (self.scale - other.scale).abs() < epsilon
    }
}


impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    const EPSILON: f64 = 1e-6;
    
    #[test]
    fn test_identity() {
        let t = Transform::IDENTITY;
        assert!(t.is_identity());
        
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = t.transform_point(point);
        assert!(transformed.approx_eq(point, EPSILON));
    }
    
    #[test]
    fn test_translation() {
        let t = Transform::from_position(Vec3::new(10.0, 20.0, 30.0));
        
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = t.transform_point(point);
        
        assert!(transformed.approx_eq(Vec3::new(11.0, 22.0, 33.0), EPSILON));
    }
    
    #[test]
    fn test_rotation() {
        let rotation = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
        let t = Transform::from_rotation(rotation);
        
        let point = Vec3::X;
        let transformed = t.transform_point(point);
        
        let is_z = transformed.approx_eq(Vec3::Z, EPSILON);
        let is_neg_z = transformed.approx_eq(Vec3::NEG_Z, EPSILON);

        assert!(is_z || is_neg_z, "Expected +/-Z, got {:?}", transformed);
    }
    
    #[test]
    fn test_scale() {
        let t = Transform::from_scale(2.0);
        
        let point = Vec3::new(1.0, 2.0, 3.0);
        let transformed = t.transform_point(point);
        
        assert!(transformed.approx_eq(Vec3::new(2.0, 4.0, 6.0), EPSILON));
    }
    
    #[test]
    fn test_combined_transform() {
        let t = Transform::new(
            Vec3::new(10.0, 0.0, 0.0),
            Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2),
            2.0,
        );
        
        let point = Vec3::X;
        let transformed = t.transform_point(point);
        
        // Scale (2x), rotate (90° around Y), translate (+10 X)
        // X -> (2, 0, 0) -> (0, 0, 2) -> (10, 0, 2)
        assert!(transformed.approx_eq(Vec3::new(10.0, 0.0, 2.0), EPSILON));
    }
    
    #[test]
    fn test_inverse() {
        let t = Transform::new(
            Vec3::new(5.0, 10.0, 15.0),
            Quaternion::from_axis_angle(Vec3::Y, 0.5),
            2.0,
        );
        
        let inv = t.inverse();
        let combined = t.combine(inv);
        
        assert!(combined.is_identity());
    }
    
    #[test]
    fn test_combine() {
        let t1 = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
        let t2 = Transform::from_position(Vec3::new(0.0, 5.0, 0.0));
        
        let combined = t1.combine(t2);
        
        let point = Vec3::ZERO;
        let transformed = combined.transform_point(point);
        
        assert!(transformed.approx_eq(Vec3::new(10.0, 5.0, 0.0), EPSILON));
    }
    
    #[test]
    fn test_transform_vector_no_translation() {
        let t = Transform::from_position(Vec3::new(100.0, 200.0, 300.0));
        
        let vector = Vec3::new(1.0, 0.0, 0.0);
        let transformed = t.transform_vector(vector);
        
        // Vector should not be translated
        assert!(transformed.approx_eq(vector, EPSILON));
    }
    
    #[test]
    fn test_transform_direction_no_scale() {
        let t = Transform::from_scale(5.0);
        
        let dir = Vec3::X;
        let transformed = t.transform_direction(dir);
        
        // Direction should not be scaled
        assert!(transformed.approx_eq(dir, EPSILON));
    }
    
    #[test]
    fn test_lerp() {
        let t1 = Transform::from_position(Vec3::ZERO);
        let t2 = Transform::from_position(Vec3::new(10.0, 10.0, 10.0));
        
        let mid = t1.lerp(t2, 0.5);
        
        assert!(mid.position.approx_eq(Vec3::new(5.0, 5.0, 5.0), EPSILON));
    }
    
    #[test]
    fn test_directions() {
        let rotation = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
        let t = Transform::from_rotation(rotation);
        
        // After 90° rotation around Y:
        // Forward (Z) becomes -X
        // Right (X) becomes Z
        // Up (Y) stays Y
        
        assert!(t.forward().approx_eq(Vec3::NEG_X, EPSILON));
        assert!(t.right().approx_eq(Vec3::Z, EPSILON));
        assert!(t.up().approx_eq(Vec3::Y, EPSILON));
    }
    
    #[test]
    fn test_builder_pattern() {
        let t = Transform::IDENTITY
            .translate(Vec3::new(10.0, 0.0, 0.0))
            .rotate(Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2))
            .mul_scale(2.0);
        
        assert_eq!(t.position, Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(t.scale, 2.0);
    }
}
    
