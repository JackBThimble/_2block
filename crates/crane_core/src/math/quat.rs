use crate::math::Vec3;
use std::ops::{Mul, MulAssign, Neg};

#[cfg(feature = "simd")]
use wide::f64x4;

/// Unit quaternion for representing 3D rotations
///
/// Stored as (x, y, z, w) where (x, y, z) is the vector part
/// and w is the scalar part. For a unit quaternion representing
/// a rotation, w^2 + x^2 + y^2 + z^2 = 1.
/// Uses f64 for accumulated precision over time
/// Normalized quaternions represent rotations without gimbal lock.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Quaternion {
    /// Idntity quaternion (no rotation)
    pub const IDENTITY: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Create a quaternino from components (x, y, z, w)
    #[inline(always)]
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    /// Create from a vector and scalar parts
    #[inline(always)]
    pub const fn from_parts(vector: Vec3, scalar: f64) -> Self {
        Self::new(vector.x, vector.y, vector.z, scalar)
    }

    /// Create from axis and angle
    ///
    /// Axis must be normalized. Angle in radians
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle_rad: f64) -> Self {
        let half_angle = angle_rad * 0.5;
        let s = half_angle.sin();
        let c = half_angle.cos();

        let mut q = Self::new(axis.x * s, axis.y * s, axis.z * s, c);

        if q.w < 0.0 {
            q = -q;
        }
        q
    }

    /// Create rotation from euler angles (yaw pitch, roll in radians)
    ///
    /// Order: Yaw (Y) -> Pitch (X) -> Roll (Z)
    #[inline]
    pub fn from_euler(yaw: f64, pitch: f64, roll: f64) -> Self {
        let (sz, cz) = (yaw * 0.5).sin_cos();
        let (sy, cy) = (pitch * 0.5).sin_cos();
        let (sx, cx) = (roll * 0.5).sin_cos();

        Self::new(
            sx * cy * cz - cx * sy * sz,
            cx * sy * cz + sx * cy * sz,
            cx * cy * sz - sx * sy * cz,
            cx * cy * cz + sx * sy * sz,
        )
    }

    /// Create rotation that rotates from one vector to another
    #[inline]
    pub fn from_rotation_arc(from: Vec3, to: Vec3) -> Self {
        let from = from.normalized();
        let to = to.normalized();
        let dot = from.dot(to);

        let q = if dot > 0.999999 {
            Self::IDENTITY
        } else if dot < -0.999999 {
            let axis = if from.x.abs() > from.z.abs() {
                Vec3::new(-from.y, from.x, 0.0).normalized()
            } else {
                Vec3::new(0.0, -from.z, from.y).normalized()
            };
            Self::from_axis_angle(axis, std::f64::consts::PI)
        } else {
let axis = from.cross(to);
let w = 1.0 + dot;
Self::new(axis.x, axis.y, axis.z, w).normalized()
        };

        if q.w < 0.0 { - q } else { q}
    }

    /// Create look-at rotation (camera-style)
    ///
    /// Creates a rotation that looks from `eye` toward `target`
    /// with `up` as the up direction
    #[inline]
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - eye).normalized();
        let right = up.cross(forward).normalized();
        let up = forward.cross(right);

        Self::from_rotation_matrix(right, up, forward)
    }

    /// Create from rotation matrix columns
    #[inline]
    fn from_rotation_matrix(right: Vec3, up: Vec3, forward: Vec3) -> Self {
        let trace = right.x + up.y + forward.z;

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self::new(
                (up.z - forward.y) / s,
                (forward.x - right.z) / s,
                (right.y - up.x) / s,
                0.25 * s,
            )
        } else if right.x > up.y && right.x > forward.z {
            let s = (1.0 + right.x - up.y - forward.z).sqrt() * 2.0;
            Self::new(
                0.25 * s,
                (up.x + right.y) / s,
                (forward.x + right.z) / s,
                (up.z - forward.y) / s,
            )
        } else if up.y > forward.z {
            let s = (1.0 + up.y - right.x - forward.z).sqrt() * 2.0;
            Self::new(
                (up.x + right.y) / s,
                0.25 * s,
                (forward.y + up.z) / s,
                (forward.x - right.z) / s,
            )
        } else {
            let s = (1.0 + forward.z - right.x - up.y).sqrt() * 2.0;
            Self::new(
                (forward.x + right.z) / s,
                (forward.y + up.z) / s,
                0.25 * s,
                (right.y - up.x) / s,
            )
        }
    }

    /// Get vector part
    #[inline(always)]
    pub const fn vector(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Get scalar part
    #[inline(always)]
    pub const fn scalar(self) -> f64 {
        self.w
    }

    // ========================================================================
    // OPERATIONS
    // ========================================================================

    /// Squared length
    #[inline]
    pub fn length_squared(self) -> f64 {
        #[cfg(feature = "simd")]
        {
            let q = f64x4::new([self.x, self.y, self.z, self.w]);
            let sq = q * q;
            let arr = sq.to_array();
            arr[0] + arr[1] + arr[2] + arr[3]
        }

        #[cfg(not(feature = "simd"))]
        {
            self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
        }
    }

    /// Length
    #[inline]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Create a quaternion from an array [x, y, z, w]
    #[inline]
    pub const fn from_array(arr: [f64; 4]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
            w: arr[3],
        }
    }

    /// Convert to array [x, y, z ,w]
    #[inline]
    pub const fn to_array(self) -> [f64; 4] {
        [self.x, self.y, self.z, self.w]
    }

    /// Convert to axis-angle representation
    ///
    /// Returns (axis, angle_radians). Axis is normalized
    #[inline]
    pub fn to_axis_angle(self) -> (Vec3, f64) {
        let q = self.normalized();

        let w = q.w.clamp(-1.0, 1.0);
        let angle = 2.0 * w.acos();

        let s = (1.0 - w * w).sqrt();

        if s < 1e-10 {
            (Vec3::X, 0.0)
        } else {
            let axis = Vec3::new(q.x / s, q.y / s, q.z / s);
            (axis, angle)
        }
    }

    /// Get euler angles (yaw, pitch, roll in radians)
    #[inline]
    pub fn to_euler(self) -> (f64, f64, f64) {
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            sinp.signum() * std::f64::consts::FRAC_PI_2
        } else {
            sinp.asin()
        };

        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        (yaw, pitch, roll)
    }

    /// Construct a quaternion from orthonormal basis vectors (right, up, forward)
    pub fn from_basis(right: Vec3, up: Vec3, forward: Vec3) -> Self {
        // Column major 3x3 rotation matrix
        let m00 = right.x;
        let m01 = up.x;
        let m02 = forward.x;
        let m10 = right.y;
        let m11 = up.y;
        let m12 = forward.y;
        let m20 = right.z;
        let m21 = up.z;
        let m22 = forward.z;

        let trace = m00 + m11 + m22;
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self {
                w: 0.25 * s,
                x: (m21 - m12) / s,
                y: (m02 - m20) / s,
                z: (m10 - m01) / s,
            }
        } else if (m00 > m11) && (m00 > m22) {
            let s = ((1.0 + m00 - m11 - m22).sqrt()) * 2.0;
            Self {
                w: (m21 - m12) / s,
                x: 0.25 * s,
                y: (m01 + m10) / s,
                z: (m02 + m20) / s,
            }
        } else if m11 > m22 {
            let s = ((1.0 + m11 - m00 - m22).sqrt()) * 2.0;
            Self {
                w: (m02 - m20) / s,
                x: (m01 + m10) / s,
                y: 0.25 * s,
                z: (m12 + m21) / s,
            }
        } else {
            let s = ((1.0 + m22 - m00 - m11).sqrt()) * 2.0;
            Self {
                w: (m10 - m01) / s,
                x: (m02 + m20) / s,
                y: (m12 + m21) / s,
                z: 0.25 * s,
            }
        }
        .normalized()
    }

    pub fn to_mat3(self) -> [[f64; 3]; 3] {
        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;

        let xx = self.x * x2;
        let yy = self.y * y2;
        let zz = self.z * z2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yz = self.y * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        [
            [1.0 - (yy + zz), xy + wz, xz - wy],
            [xy - wz, 1.0 - (xx + zz), yz + wx],
            [xz + wy, yz - wx, 1.0 - (xx + yy)],
        ]
    }

    /// Rotate a vector by this quaternion
    #[inline]
    pub fn rotate_vector(self, v: Vec3) -> Vec3 {
        let qv = self.vector();
        let t = qv.cross(v) * 2.0;

        v + t * self.w + qv.cross(t)
    }

    /// Multiply by another quaternion (comcatenate rotations)
    #[inline]
    pub fn mul_quat(self, other: Self) -> Self {
        Self::new(
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        )
    }

    /// Dot product
    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        #[cfg(feature = "simd")]
        {
            let a = f64x4::new([self.x, self.y, self.z, self.w]);
            let b = f64x4::new([other.x, other.y, other.z, other.w]);
            let product = a * b;
            let arr = product.to_array();
            arr[0] + arr[1] + arr[2] + arr[3]
        }

        #[cfg(not(feature = "simd"))]
        {
            self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
        }
    }

    /// Normalize to unit quaternion
    #[inline]
    pub fn normalized(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq < 1e-10 {
            return Self::IDENTITY;
        }

        let inv_len = 1.0 / len_sq.sqrt();

        #[cfg(feature = "simd")]
        {
            let q = f64x4::new([self.x, self.y, self.z, self.w]);
            let inv = f64x4::splat(inv_len);
            let result = q * inv;
            let arr = result.to_array();
            Self::new(arr[0], arr[1], arr[2], arr[3])
        }

        #[cfg(not(feature = "simd"))]
        {
            Self::new(
                self.x * inv_len,
                self.y * inv_len,
                self.z * inv_len,
                self.w * inv_len,
            )
        }
    }

    /// Normalize in place
    #[inline]
    pub fn normalize(&mut self) {
        *self = self.normalized();
    }

    /// Check if quaternion is normalized
    #[inline]
    pub fn is_normalized(self) -> bool {
        (self.length_squared() - 1.0).abs() < 1e-6
    }

    /// Check if quaternion is finite
    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }

    /// Check if approximately equal
    #[inline]
    pub fn approx_eq(self, other: Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon
            && (self.y - other.y).abs() < epsilon
            && (self.z - other.z).abs() < epsilon
            && (self.w - other.w).abs() < epsilon
    }

    /// Conjugate (inverse for unit quaternions)
    #[inline(always)]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    /// Inverse
    #[inline]
    pub fn inverse(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq < 1e-10 {
            return Self::IDENTITY;
        }

        let inv_len_sq = 1.0 / len_sq;
        Self::new(
            -self.x * inv_len_sq,
            -self.y * inv_len_sq,
            -self.z * inv_len_sq,
            self.w * inv_len_sq,
        )
    }

    /// Spherical linear interpolation
    ///
    /// Smoothly interpolate between two quaternions
    /// t should be in [0, 1].
    #[inline]
    pub fn slerp(self, other: Self, t: f64) -> Self {
        let dot = self.dot(other);

        // If dot < 0, negate one quaternion to take shorter path
        let (other, dot) = if dot < 0.0 {
            (-other, -dot)
        } else {
            (other, dot)
        };

        // If quaternions are very close, use linear interpolation
        if dot > 0.9995 {
            return self.lerp(other, t).normalized();
        }

        // Spherical interpolation
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();

        let a = ((1.0 - t) * theta).sin() / sin_theta;
        let b = (t * theta).sin() / sin_theta;

        Self::new(
            a * self.x + b * other.x,
            a * self.y + b * other.y,
            a * self.z + b * other.z,
            a * self.w + b * other.w,
        )
    }

    /// Linear interpolation (cheaper than slerp, but not constant angular velocity)
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
            self.w + (other.w - self.w) * t,
        )
    }

    // ========================================================================
    // CRANE-SPECIFIC HELPERS
    // ========================================================================

    /// Create rotation around Y axis (slewing/swing)
    #[inline]
    pub fn from_y_rotation(angle_radians: f64) -> Self {
        Self::from_axis_angle(Vec3::Y, angle_radians)
    }

    /// Create rotation around X axis (boom angle)
    #[inline]
    pub fn from_x_rotation(angle_radians: f64) -> Self {
        Self::from_axis_angle(Vec3::X, angle_radians)
    }

    /// Create rotation around Z axis
    #[inline]
    pub fn from_z_rotation(angle_radians: f64) -> Self {
        Self::from_axis_angle(Vec3::Z, angle_radians)
    }
}

impl Mul for Quaternion {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.mul_quat(other)
    }
}

impl MulAssign for Quaternion {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Neg for Quaternion {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
}

use std::fmt;
impl fmt::Display for Quaternion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (axis, angle) = self.to_axis_angle();
        write!(f, "Quat(axis:{}, angle:{:.3}°)", axis, angle.to_degrees())
    }
}

#[cfg(feature = "bevy")]
impl From<Quaternion> for bevy_math::Quat {
    #[inline]
    fn from(q: Quaternion) -> Self {
        bevy_math::Quat::from_xyzw(q.x as f32, q.y as f32, q.z as f32, q.w as f32)
    }
}

#[cfg(feature = "bevy")]
impl From<bevy_math::Quat> for Quaternion {
    #[inline]
    fn from(q: bevy_math::Quat) -> Self {
        Quaternion::new(q.x as f64, q.y as f64, q.z as f64, q.w as f64)
    }
}

impl Default for Quaternion {
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
        let q = Quaternion::IDENTITY;
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
        assert_eq!(q.w, 1.0);
        assert!(q.is_normalized());
    }

    #[test]
    fn test_from_axis_angle() {
        // 90 degree rotation around Y axis
        let q = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
        assert!(q.is_normalized());

        // Rotate X vector
        let rotated = q.rotate_vector(Vec3::X);
        assert!(rotated.approx_eq(Vec3::NEG_Z, EPSILON) || rotated.approx_eq(Vec3::Z, EPSILON));
    }

    #[test]
    fn test_normalization() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let normalized = q.normalized();

        assert!(normalized.is_normalized());

        let len = normalized.length();
        assert_relative_eq!(len, 1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_conjugate() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let conj = q.conjugate();

        assert_eq!(conj.x, -1.0);
        assert_eq!(conj.y, -2.0);
        assert_eq!(conj.z, -3.0);
        assert_eq!(conj.w, 4.0);
    }

    #[test]
    fn test_inverse() {
        let q = Quaternion::from_axis_angle(Vec3::Y, 0.5).normalized();
        let inv = q.inverse();

        let product = q * inv;
        assert!(product.approx_eq(Quaternion::IDENTITY, EPSILON));
    }

    #[test]
    fn test_multiplication() {
        // Two 90° rotations around Y should equal 180° rotation
        let q1 = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
        let q2 = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
        let combined = q1 * q2;

        let expected = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::PI);

        // Compare rotations (quaternions q and -q represent same rotation)
        assert!(combined.approx_eq(expected, EPSILON) || combined.approx_eq(-expected, EPSILON));
    }

    #[test]
    fn test_rotate_vector() {
        // 90° rotation around Y
        let q = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);

        // X becomes Z
        let rotated_x = q.rotate_vector(Vec3::X);
        let is_z = rotated_x.approx_eq(Vec3::Z, EPSILON);
        let is_neg_z = rotated_x.approx_eq(Vec3::NEG_Z, EPSILON);
        assert!(is_z || is_neg_z, "Expected Z or -Z, got {:?}", rotated_x);

        // Y unchanged
        let rotated_y = q.rotate_vector(Vec3::Y);
        assert!(rotated_y.approx_eq(Vec3::Y, EPSILON));
    }

    #[test]
    fn test_rotation_preserves_length() {
        let q = Quaternion::from_axis_angle(Vec3::new(1.0, 1.0, 1.0).normalized(), 1.23);
        let v = Vec3::new(3.0, 4.0, 5.0);

        let rotated = q.rotate_vector(v);

        let len_before = v.length();
        let len_after = rotated.length();

        assert_relative_eq!(len_before, len_after, epsilon = EPSILON);
    }

    #[test]
    fn test_axis_angle_roundtrip() {
        let axis = Vec3::new(1.0, 2.0, 3.0).normalized();
        let angle = 1.5;

        let q = Quaternion::from_axis_angle(axis, angle);
        let (axis2, angle2) = q.to_axis_angle();

        let same_axis = axis.approx_eq(axis2, EPSILON) || axis.approx_eq(-axis2, EPSILON);

        assert!(same_axis, "Axis mismatch: {:?} vs {:?}", axis, axis2);

        assert_relative_eq!(angle, angle2, epsilon = EPSILON);
    }

    #[test]
    fn test_slerp() {
        let q1 = Quaternion::IDENTITY;
        let q2 = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);

        // At t=0, should be q1
        let s0 = q1.slerp(q2, 0.0);
        assert!(s0.approx_eq(q1, EPSILON));

        // At t=1, should be q2
        let s1 = q1.slerp(q2, 1.0);
        assert!(s1.approx_eq(q2, EPSILON) || s1.approx_eq(-q2, EPSILON));

        // At t=0.5, should be 45° rotation
        let s_mid = q1.slerp(q2, 0.5);
        let expected = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_4);
        assert!(s_mid.approx_eq(expected, EPSILON) || s_mid.approx_eq(-expected, EPSILON));
    }

    #[test]
    fn test_from_rotation_arc() {
        let from = Vec3::X;
        let to = Vec3::Y;

        let q = Quaternion::from_rotation_arc(from, to);
        let rotated = q.rotate_vector(from);

        assert!(rotated.approx_eq(to, EPSILON));
    }

    #[test]
    fn test_euler_identity() {
        let (yaw, pitch, roll) = (0.0, 0.0, 0.0);
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_euler();

        assert_relative_eq!(y, yaw, epsilon = EPSILON);
        assert_relative_eq!(p, pitch, epsilon = EPSILON);
        assert_relative_eq!(r, roll, epsilon = EPSILON);

        assert_relative_eq!(q.w, 1.0, epsilon = EPSILON);
        assert_relative_eq!(q.x, 0.0, epsilon = EPSILON);
        assert_relative_eq!(q.y, 0.0, epsilon = EPSILON);
        assert_relative_eq!(q.z, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn test_euler_yaw_90() {
        let (yaw, pitch, roll) = (90.0f64.to_radians(), 0.0, 0.0);
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_euler();

        assert_relative_eq!(y, yaw, epsilon = EPSILON);
        assert_relative_eq!(p, pitch, epsilon = EPSILON);
        assert_relative_eq!(r, roll, epsilon = EPSILON);

        assert_relative_eq!(q.w, (45.0f64).to_radians().cos(), epsilon = EPSILON);
        assert_relative_eq!(q.z, (45.0f64).to_radians().sin(), epsilon = EPSILON);
    }

    #[test]
    fn test_euler_pitch_90() {
        let (yaw, pitch, roll) = (0.0, 90.0f64.to_radians(), 0.0);
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_euler();

        assert_relative_eq!(y, yaw, epsilon = EPSILON);
        assert_relative_eq!(p, pitch, epsilon = EPSILON);
        assert_relative_eq!(r, roll, epsilon = EPSILON);
    }

    #[test]
    fn test_euler_roll_90() {
        let (yaw, pitch, roll) = (0.0, 0.0, 90.0f64.to_radians());
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_euler();

        assert_relative_eq!(y, yaw, epsilon = EPSILON);
        assert_relative_eq!(p, pitch, epsilon = EPSILON);
        assert_relative_eq!(r, roll, epsilon = EPSILON);
    }

    #[test]
    fn test_euler_roundtrip() {
        let test_cases: Vec<(f64, f64, f64)> = vec![
            (30.0, 15.0, 45.0),
            (90.0, -30.0, 60.0),
            (-120.0, 45.0, 10.0),
            (180.0, 89.0, -90.0),
        ];

        for &(y_deg, p_deg, r_deg) in &test_cases {
            let (yaw, pitch, roll) = (y_deg.to_radians(), p_deg.to_radians(), r_deg.to_radians());
            let q = Quaternion::from_euler(yaw, pitch, roll);
            let (y, p, r) = q.to_euler();

            assert_relative_eq!(y, yaw, epsilon = 1e-5);
            assert_relative_eq!(p, pitch, epsilon = 1e-5);
            assert_relative_eq!(r, roll, epsilon = 1e-5);
        }
    }

    #[test]
    fn test_euler_gimbal_lock() {
        // Test gimbal lock case (pitch = ±90°)
        // At gimbal lock, yaw and roll become dependent
        let (yaw, pitch, roll) = (
            45.0f64.to_radians(),
            90.0f64.to_radians(),
            30.0f64.to_radians(),
        );
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (_, p, _) = q.to_euler();

        assert_relative_eq!(p, 90.0f64.to_radians(), epsilon = 1e-4);
    }

    #[test]
    fn test_euler_negative_angles() {
        let (yaw, pitch, roll) = (
            -45.0f64.to_radians(),
            -30.0f64.to_radians(),
            -60.0f64.to_radians(),
        );
        let q = Quaternion::from_euler(yaw, pitch, roll);
        let (y, p, r) = q.to_euler();

        assert_relative_eq!(y, yaw, epsilon = EPSILON);
        assert_relative_eq!(p, pitch, epsilon = EPSILON);
        assert_relative_eq!(r, roll, epsilon = EPSILON);
    }

    #[test]
    fn test_euler_composition_equivalence() {
        let q1 = Quaternion::from_euler(90.0f64.to_radians(), 0.0, 0.0);
        let q2 = Quaternion::from_euler(0.0, 0.0, 90.0f64.to_radians())
            * Quaternion::from_euler(90.0f64.to_radians(), 0.0, 0.0);

        assert!((q1.w - q2.w).abs() > 1e-3 || (q1.x - q2.x).abs() > 1e-3);
    }
}
