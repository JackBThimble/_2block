use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[cfg(feature = "simd")]
use wide::f64x4;

/// 3D vector with f64 precision
///
/// Used for positions, directions, and offsets in world space
/// All coordinates are implicitly in meters when used for positions.
/// Optimized for both scalar and SIMD operations
/// SIMD can be enabled with the "simd" feature flag.

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Zero vector (0, 0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    /// One vector (1, 1, 1)
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);
    /// Unit X vector (1, 0, 0)
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    /// Unit Y vector (0, 1, 0)
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    /// Unit Z vector (0, 0, 1)
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);
    /// Negative unit X negative vector (-1, 0, 0)
    pub const NEG_X: Self = Self::new(-1.0, 0.0, 0.0);
    /// Negative unit Y negative vector (0, -1, 0)
    pub const NEG_Y: Self = Self::new(0.0, -1.0, 0.0);
    /// Negative unit Z negative vector (0, 0, -1)
    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0);
    /// Up direction (0, 1, 0)
    pub const UP: Vec3 = Vec3::Y;
    /// Down direction (0, -1, 0)
    pub const DOWN: Vec3 = Vec3::NEG_Y;
    /// Right direction (1, 0, 0)
    pub const RIGHT: Vec3 = Vec3::X;
    /// Left direction (-1, 0, 0)
    pub const LEFT: Vec3 = Vec3::NEG_X;
    /// Forward direction (0, 0, 1)
    pub const FORWARD: Vec3 = Vec3::Z;
    /// Back direction (0, 0, -1)
    pub const BACKWARD: Vec3 = Vec3::NEG_Z;

    /// Create a  new vector
    #[inline(always)]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a vector with all components set to the same value
    #[inline(always)]
    pub const fn splat(value: f64) -> Self {
        Self::new(value, value, value)
    }

    /// Create a vector from an array
    #[inline(always)]
    pub const fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    /// Convert to array
    #[inline(always)]
    pub const fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    // ========================================================================
    // MATHEMATICAL OPERATIONS
    // ========================================================================

    /// Dot product with another vector
    ///
    /// Optimized with SIMD when available
    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        #[cfg(feature = "simd")]
        {
            // SIMD path
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let b = f64x4::new([other.x, other.y, other.z, 0.0]);
            let product = a * b;
            let arr = product.to_array();
            arr[0] + arr[1] + arr[2]
        }

        #[cfg(not(feature = "simd"))]
        {
            self.x * other.x + self.y * other.y + self.z * other.z
        }
    }

    /// Cross product with another vector(right-handed)
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Squared length
    ///
    /// Faster than length - avoids sqrt
    #[inline]
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    /// Length (magnitude of the vector)
    #[inline]
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Alias for length() - more intuitive in some contexts
    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.length()
    }

    /// Distance squared to another point
    #[inline]
    pub fn distance_squared(self, other: Self) -> f64 {
        (self - other).length_squared()
    }

    /// Distance to another point
    #[inline]
    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    /// Distance to another point (alias)
    #[inline]
    pub fn distance_to(&self, other: Self) -> f64 {
        (*self - other).length()
    }

    /// Normalize the vector to unit length
    ///
    /// Returns zero vector if length is too small
    #[inline]
    pub fn normalized(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq < 1e-10 {
            Self::ZERO
        } else {
            let inv_len = 1.0 / len_sq.sqrt();
            self * inv_len
        }
    }

    /// Try to normalize, returns None if vector is too small
    #[inline]
    pub fn try_normalize(self) -> Option<Self> {
        let len_sq = self.length_squared();
        if len_sq < 1e-10 {
            None
        } else {
            let inv_len = 1.0 / len_sq.sqrt();
            Some(self * inv_len)
        }
    }

    /// Normalize in place
    #[inline]
    pub fn normalize(&mut self) {
        *self = self.normalized();
    }

    /// Linear interpolation between two vectors
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let b = f64x4::new([other.x, other.y, other.z, 0.0]);
            let t_vec = f64x4::splat(t);
            let one_minus_t = f64x4::splat(1.0 - t);
            let result = a * one_minus_t + b * t_vec;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2])
        }

        #[cfg(not(feature = "simd"))]
        {
            self + (other - self) * t
        }
    }

    /// Component-wise minimum
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    /// Component-wise maximum
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    /// Component-wise absolute value
    #[inline]
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    /// Clamp each component between min and max
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
            z: self.z.clamp(min.z, max.z),
        }
    }

    /// Component-wise multiplication
    #[inline]
    pub fn mul_components(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    /// Component-wise division
    #[inline]
    pub fn div_components(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }

    /// Project this vector onto another vector
    #[inline]
    pub fn project_onto(self, onto: Self) -> Self {
        let onto_len_sq = onto.length_squared();
        if onto_len_sq < 1e-10 {
            Self::ZERO
        } else {
            onto * (self.dot(onto) / onto_len_sq)
        }
    }

    /// Reject this vector from another (perpendicular component)
    #[inline]
    pub fn reject_from(self, from: Self) -> Self {
        self - self.project_onto(from)
    }

    /// Reflect this vector across a normal
    #[inline]
    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    /// Angle between two vectors (in radians)
    #[inline]
    pub fn angle_between(self, other: Self) -> f64 {
        let dot = self.dot(other);
        let len_product = self.length() * other.length();

        if len_product < 1e-10 {
            0.0
        } else {
            (dot / len_product).clamp(-1.0, 1.0).acos()
        }
    }

    /// Check if vector is finite (no NaN or infinity)
    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Check if vector is normalized (unit length)
    #[inline]
    pub fn is_normalized(self) -> bool {
        (self.length_squared() - 1.0).abs() < 1e-6
    }

    /// Check if vector is approximately zero
    #[inline]
    pub fn is_zero(self) -> bool {
        self.length_squared() < 1e-10
    }

    /// Check if approximately equal to another vector
    #[inline]
    pub fn approx_eq(self, other: Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
    }

    // ========================================================================
    // CRANE-SPECIFIC OPERATIONS
    // ========================================================================

    /// Project onto horizontal plane (XZ plane, Y=0)
    /// Usefule for calculating horizontal distances
    #[inline]
    pub fn horizontal_projection(&self) -> Self {
        Self::new(self.x, 0.0, self.z)
    }

    /// Horizontal distance to another point (ignoring Y)
    #[inline]
    pub fn horizontal_distance_to(&self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }

    /// Get the horizontal angle (in radians) from this vector
    /// Returns angle in range [-π, π] where 0 = +Z, π/2 = +X
    #[inline]
    pub fn horizontal_angle(&self) -> f64 {
        self.z.atan2(self.x)
    }

    /// Vertical angle from horizontal (in radians)
    /// Returns angle in range [-π/2, π/2]
    #[inline]
    pub fn vertical_angle(&self) -> f64 {
        let horizontal_len = self.horizontal_projection().length();
        self.y.atan2(horizontal_len)
    }

    /// Create a vector form horizontal and vertical angles
    /// Useful for boom positioning
    pub fn from_angles(horizontal_rad: f64, vertical_rad: f64, length: f64) -> Self {
        let horizontal_len = length * vertical_rad.cos();
        Self {
            x: horizontal_len * horizontal_rad.sin(),
            y: length * vertical_rad.sin(),
            z: horizontal_len * horizontal_rad.cos(),
        }
    }
}

// ============================================================================
// STANDARD OPERATIONS
// ============================================================================

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let b = f64x4::new([rhs.x, rhs.y, rhs.z, 0.0]);
            let result = a + b;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2])
        }

        #[cfg(not(feature = "simd"))]
        {
            Self {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
                z: self.z + rhs.z,
            }
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let b = f64x4::new([rhs.x, rhs.y, rhs.z, 0.0]);
            let result = a - b;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2])
        }

        #[cfg(not(feature = "simd"))]
        {
            Self {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
                z: self.z - rhs.z,
            }
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: f64) -> Self {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let s = f64x4::splat(scalar);
            let result = a * s;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2])
        }

        #[cfg(not(feature = "simd"))]
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    #[inline]
    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    #[inline]
    fn div(self, scalar: f64) -> Self {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, 0.0]);
            let s = f64x4::splat(scalar);
            let result = a / s;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2])
        }

        #[cfg(not(feature = "simd"))]
        {
            let inv = 1.0 / scalar;
            Self {
                x: self.x * inv,
                y: self.y * inv,
                z: self.z * inv,
            }
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl DivAssign<f64> for Vec3 {
    #[inline]
    fn div_assign(&mut self, scalar: f64) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
    }
}

impl Default for Vec3 {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(f64, f64, f64)> for Vec3 {
    #[inline]
    fn from(tuple: (f64, f64, f64)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

impl From<[f64; 3]> for Vec3 {
    #[inline]
    fn from(arr: [f64; 3]) -> Self {
        Self::from_array(arr)
    }
}

impl From<Vec3> for [f64; 3] {
    #[inline]
    fn from(vec: Vec3) -> Self {
        vec.to_array()
    }
}

#[cfg(feature = "bevy")]
impl From<Vec3> for bevy_math::Vec3 {
    #[inline]
    fn from(vec: Vec3) -> Self {
        bevy_math::Vec3::new(vec.x as f32, vec.y as f32, vec.z as f32)
    }
}

#[cfg(feature = "bevy")]
impl From<bevy_math::Vec3> for Vec3 {
    #[inline]
    fn from(vec: bevy_math::Vec3) -> Self {
        Vec3::new(vec.x as f64, vec.y as f64, vec.z as f64)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    
    const EPSILON: f64 = 1e-10;
    
    #[test]
    fn test_constants() {
        assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec3::X, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(Vec3::Y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(Vec3::Z, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(Vec3::ONE, Vec3::new(1.0, 1.0, 1.0));
    }
    
    #[test]
    fn test_creation() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
        
        let v2 = Vec3::splat(5.0);
        assert_eq!(v2, Vec3::new(5.0, 5.0, 5.0));
        
        let v3 = Vec3::from_array([1.0, 2.0, 3.0]);
        assert_eq!(v3, v);
        
        assert_eq!(v.to_array(), [1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_basic_arithmetic() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        
        let sum = v1 + v2;
        assert_eq!(sum, Vec3::new(5.0, 7.0, 9.0));
        
        let diff = v2 - v1;
        assert_eq!(diff, Vec3::new(3.0, 3.0, 3.0));
        
        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vec3::new(2.0, 4.0, 6.0));
        
        let scaled2 = 2.0 * v1;
        assert_eq!(scaled2, scaled);
        
        let divided = v2 / 2.0;
        assert_eq!(divided, Vec3::new(2.0, 2.5, 3.0));
        
        let negated = -v1;
        assert_eq!(negated, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_assign_ops() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        
        v += Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v, Vec3::new(2.0, 3.0, 4.0));
        
        v -= Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
        
        v *= 2.0;
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
        
        v /= 2.0;
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }
    #[test]
    fn test_dot_product() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        
        let dot = v1.dot(v2);
        assert_relative_eq!(dot, 32.0, epsilon = EPSILON);
        
        // Dot product of perpendicular vectors
        let x = Vec3::X;
        let y = Vec3::Y;
        assert_relative_eq!(x.dot(y), 0.0, epsilon = EPSILON);
        
        // Dot product with self equals length squared
        assert_relative_eq!(v1.dot(v1), v1.length_squared(), epsilon = EPSILON);
    }
    
    #[test]
    fn test_cross_product() {
        let x = Vec3::X;
        let y = Vec3::Y;
        let z = Vec3::Z;
        
        // Right-hand rule
        assert_eq!(x.cross(y), z);
        assert_eq!(y.cross(z), x);
        assert_eq!(z.cross(x), y);
        
        // Anti-commutative
        assert_eq!(x.cross(y), -y.cross(x));
        
        // Cross product with self is zero
        assert_eq!(x.cross(x), Vec3::ZERO);
        
        // Cross product is perpendicular
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let cross = v1.cross(v2);
        assert_relative_eq!(cross.dot(v1), 0.0, epsilon = EPSILON);
        assert_relative_eq!(cross.dot(v2), 0.0, epsilon = EPSILON);
    }
    
    #[test]
    fn test_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_relative_eq!(v.length(), 5.0, epsilon = EPSILON);
        assert_relative_eq!(v.length_squared(), 25.0, epsilon = EPSILON);
        
        let unit = Vec3::new(1.0, 0.0, 0.0);
        assert_relative_eq!(unit.length(), 1.0, epsilon = EPSILON);
    }
    
    #[test]
    fn test_distance() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(3.0, 4.0, 0.0);
        
        assert_relative_eq!(v1.distance(v2), 5.0, epsilon = EPSILON);
        assert_relative_eq!(v1.distance_squared(v2), 25.0, epsilon = EPSILON);
        assert_relative_eq!(v1.distance_to(v2), 5.0, epsilon = EPSILON);
    }
    
    #[test]
    fn test_normalization() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized = v.normalized();
        
        assert_relative_eq!(normalized.length(), 1.0, epsilon = EPSILON);
        assert_relative_eq!(normalized.x, 0.6, epsilon = EPSILON);
        assert_relative_eq!(normalized.y, 0.8, epsilon = EPSILON);
        
        // Zero vector normalizes to zero
        assert_eq!(Vec3::ZERO.normalized(), Vec3::ZERO);
        
        // try_normalize
        assert!(v.try_normalize().is_some());
        assert!(Vec3::ZERO.try_normalize().is_none());
        
        // In-place normalization
        let mut v2 = Vec3::new(3.0, 4.0, 0.0);
        v2.normalize();
        assert!(v2.is_normalized());
    }
    
    #[test]
    fn test_lerp() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(10.0, 10.0, 10.0);
        
        let mid = v1.lerp(v2, 0.5);
        assert_eq!(mid, Vec3::new(5.0, 5.0, 5.0));
        
        let start = v1.lerp(v2, 0.0);
        assert_eq!(start, v1);
        
        let end = v1.lerp(v2, 1.0);
        assert_eq!(end, v2);
    }
    
    #[test]
    fn test_min_max_clamp() {
        let v1 = Vec3::new(1.0, 5.0, 3.0);
        let v2 = Vec3::new(4.0, 2.0, 6.0);
        
        let min = v1.min(v2);
        assert_eq!(min, Vec3::new(1.0, 2.0, 3.0));
        
        let max = v1.max(v2);
        assert_eq!(max, Vec3::new(4.0, 5.0, 6.0));
        
        let v = Vec3::new(-1.0, 5.0, 10.0);
        let clamped = v.clamp(Vec3::ZERO, Vec3::ONE * 5.0);
        assert_eq!(clamped, Vec3::new(0.0, 5.0, 5.0));
    }
    
    #[test]
    fn test_abs() {
        let v = Vec3::new(-1.0, -2.0, 3.0);
        let abs = v.abs();
        assert_eq!(abs, Vec3::new(1.0, 2.0, 3.0));
    }
    
    #[test]
    fn test_component_wise_ops() {
        let v1 = Vec3::new(2.0, 3.0, 4.0);
        let v2 = Vec3::new(1.0, 2.0, 2.0);
        
        let mul = v1.mul_components(v2);
        assert_eq!(mul, Vec3::new(2.0, 6.0, 8.0));
        
        let div = v1.div_components(v2);
        assert_eq!(div, Vec3::new(2.0, 1.5, 2.0));
    }
    
    #[test]
    fn test_projection() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let onto = Vec3::X;
        
        let proj = v.project_onto(onto);
        assert_eq!(proj, Vec3::new(3.0, 0.0, 0.0));
        
        let reject = v.reject_from(onto);
        assert_eq!(reject, Vec3::new(0.0, 4.0, 0.0));
        
        // Projection + rejection equals original
        assert_eq!(proj + reject, v);
    }
    
    #[test]
    fn test_reflection() {
        let v = Vec3::new(1.0, -1.0, 0.0);
        let normal = Vec3::Y;
        
        let reflected = v.reflect(normal);
        assert_eq!(reflected, Vec3::new(1.0, 1.0, 0.0));
    }
    
    #[test]
    fn test_angle_between() {
        let x = Vec3::X;
        let y = Vec3::Y;
        
        let angle = x.angle_between(y);
        assert_relative_eq!(angle, std::f64::consts::FRAC_PI_2, epsilon = EPSILON);
        
        let same = x.angle_between(x);
        assert_relative_eq!(same, 0.0, epsilon = EPSILON);
        
        let opposite = x.angle_between(-x);
        assert_relative_eq!(opposite, std::f64::consts::PI, epsilon = EPSILON);
    }
    
    #[test]
    fn test_predicates() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert!(v.is_finite());
        
        let inf = Vec3::new(f64::INFINITY, 0.0, 0.0);
        assert!(!inf.is_finite());
        
        let nan = Vec3::new(f64::NAN, 0.0, 0.0);
        assert!(!nan.is_finite());
        
        assert!(Vec3::X.is_normalized());
        assert!(!v.is_normalized());
        
        assert!(Vec3::ZERO.is_zero());
        assert!(!v.is_zero());
        
        let v2 = Vec3::new(1.0001, 2.0001, 3.0001);
        assert!(v.approx_eq(v2, 0.001));
        assert!(!v.approx_eq(v2, 0.00001));
    }
    
    #[test]
    fn test_conversions() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        
        let tuple: (f64, f64, f64) = (1.0, 2.0, 3.0);
        assert_eq!(Vec3::from(tuple), v);
        
        let arr: [f64; 3] = [1.0, 2.0, 3.0];
        assert_eq!(Vec3::from(arr), v);
        
        let arr2: [f64; 3] = v.into();
        assert_eq!(arr2, arr);
    }
}
