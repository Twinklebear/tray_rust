//! Provides a Quaternion type for properly interpolating rotations

use std::f32;
use std::ops::{Add, Sub, Mul, Div};

use linalg::{self, Vector, Transform, Matrix4};

/// Quaternions describe a rotation in 3d space but can be
/// properly interpolated unlike rotation matrices. The quaternion
/// also provides methods to compute the corresponding rotation
/// transform so that it can be composed with scaling and translation
#[derive(Debug, Copy, Clone)]
pub struct Quaternion {
    pub v: Vector,
    pub w: f32,
}

impl Quaternion {
    /// Construct an identity quaternion
    pub fn identity() -> Quaternion {
        Quaternion { v: Vector::broadcast(0.0), w: 1.0 }
    }
    /// Construct a quaternion from the rotation matrix
    /// Based on Shoemake 1991
    pub fn from_matrix(m: &Matrix4) -> Quaternion {
        let trace = *m.at(0, 0) + *m.at(1, 1) + *m.at(2, 2);
        if trace > 0.0 {
            // Compute w from matrix trace, then the vector
            let mut s = f32::sqrt(trace + 1.0);
            let w = s / 2.0;
            s = 0.5 / s;
            Quaternion { v: Vector::new(s * (*m.at(2, 1) - *m.at(1, 2)), s * (*m.at(0, 2) - *m.at(2, 0)),
                                        s * (*m.at(1, 0) - *m.at(0, 1))),
                         w: w
            }
        } else {
            // Compute largest of x, y or z then the remaining components
            let next = [1, 2, 0];
            let mut q = Vector::broadcast(0.0);
            let mut i = 0;
            if *m.at(1, 1) > *m.at(0, 0) {
                i = 1;
            }
            if *m.at(2, 2) > *m.at(i, i) {
                i = 2;
            }
            let j = next[i];
            let k = next[j];
            let mut s = f32::sqrt((*m.at(i, i) - (*m.at(j, j) + *m.at(k, k))) + 1.0);
            q[i] = s * 0.5;
            if s != 0.0 {
                s = 0.5 / s;
            }
            let w = (*m.at(k, j) - *m.at(j, k)) * s;
            q[j] = (*m.at(j, i) + *m.at(i, j)) * s;
            q[k] = (*m.at(k, i) + *m.at(i, k)) * s;
            Quaternion { v: q, w: w }
        }
    }
    /// Construct the quaternion from the transform
    pub fn from_transform(t: &Transform) -> Quaternion {
        Quaternion::from_matrix(&t.mat)
    }
    /// Get the rotation transform described by this quaternion
    pub fn to_matrix(&self) -> Matrix4 {
        Matrix4::new(
            [1.0 - 2.0 * (f32::powf(self.v.y, 2.0) + f32::powf(self.v.x, 2.0)),
             2.0 * (self.v.x * self.v.y + self.v.z * self.w),
             2.0 * (self.v.x * self.v.z - self.v.y * self.w),
             0.0,

             2.0 * (self.v.x * self.v.y - self.v.z * self.w),
             1.0 - 2.0 * (f32::powf(self.v.x, 2.0) + f32::powf(self.v.z, 2.0)),
             2.0 * (self.v.y * self.v.z + self.v.x * self.w),
             0.0,

             2.0 * (self.v.x * self.v.z + self.v.y * self.w),
             2.0 * (self.v.y * self.v.z - self.v.x * self.w),
             1.0 - 2.0 * (f32::powf(self.v.x, 2.0) + f32::powf(self.v.y, 2.0)),
             0.0,

             0.0, 0.0, 0.0, 1.0
            ])
    }
    /// Get the rotation transform described by this quaternion
    pub fn to_transform(&self) -> Transform {
        Transform::from_mat(&self.to_matrix())
    }
    /// Get the normalized quaternion for this rotation
    pub fn normalized(&self) -> Quaternion {
        *self / f32::sqrt(dot(self, self))
    }
}

/// Compute the dot product of the two quaternions
pub fn dot(a: &Quaternion, b: &Quaternion) -> f32 {
    linalg::dot(&a.v, &b.v) + a.w * b.w
}

/// Use spherical linear interpolation to interpolate between the two quaternions
pub fn slerp(t: f32, a: &Quaternion, b: &Quaternion) -> Quaternion {
    // Check if a and b are nearly parallel. To avoid numerical instability we do
    // regular linear interpolation in this case
    let cos_theta = dot(a, b);
    if cos_theta > 0.9995 {
        ((1.0 - t) * *a + t * *b).normalized()
    } else {
        let theta = f32::acos(linalg::clamp(cos_theta, -1.0, 1.0));
        let theta_t = theta * t;
        let q_perp = (*b - *a * cos_theta).normalized();
        *a * f32::cos(theta_t) + q_perp * f32::sin(theta_t)
    }
}

impl Add for Quaternion {
    type Output = Quaternion;
    /// Add two quaternions
    fn add(self, rhs: Quaternion) -> Quaternion {
        Quaternion { v: self.v + rhs.v, w: self.w + rhs.w }
    }
}

impl Sub for Quaternion {
    type Output = Quaternion;
    /// Subtract two quaternions
    fn sub(self, rhs: Quaternion) -> Quaternion {
        Quaternion { v: self.v - rhs.v, w: self.w - rhs.w }
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;
    /// Multiply the quaternion by a scalar
    fn mul(self, rhs: f32) -> Quaternion {
        Quaternion { v: self.v * rhs, w: self.w * rhs }
    }
}

impl Mul<Quaternion> for f32 {
    type Output = Quaternion;
    /// Multiply the quaternion by a scalar
    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion { v: self * rhs.v, w: self * rhs.w }
    }
}

impl Div<f32> for Quaternion {
    type Output = Quaternion;
    /// Divide the quaternion by a scalar
    fn div(self, rhs: f32) -> Quaternion {
        Quaternion { v: self.v * rhs, w: self. w * rhs }
    }
}

