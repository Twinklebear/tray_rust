//! The linalg module provides some basic linear algebra functionality for
//! transforming 3D geometry

use std::f32;
use std::ops::{Index, Mul, Add};

// Re-export the linalg types from the internal modules
pub use self::vector::Vector;
pub use self::normal::Normal;
pub use self::point::Point;
pub use self::ray::Ray;
pub use self::matrix4::Matrix4;
pub use self::transform::Transform;
pub use self::quaternion::Quaternion;
pub use self::keyframe::Keyframe;
pub use self::animated_transform::AnimatedTransform;

pub mod vector;
pub mod normal;
pub mod point;
pub mod ray;
pub mod matrix4;
pub mod transform;
pub mod quaternion;
pub mod keyframe;
pub mod animated_transform;

/// Enum representing on of the 3 spatial axes
#[derive(Copy, Clone, Debug)]
pub enum Axis { X, Y, Z }

/// Convert value in degrees to radians
/// TODO: See where `f32::to_radians` ends up, if it gets stabilized into Rust switch back to it
pub fn to_radians(d: f32) -> f32 {
    f32::consts::PI / 180.0 * d
}
/// Compute the cross product of two vectors
pub fn cross<A: Index<usize, Output = f32>, B: Index<usize, Output = f32>>(a: &A, b: &B) -> vector::Vector {
    Vector::new(a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
}
/// Compute the dot product of two vectors
pub fn dot<A: Index<usize, Output = f32>, B: Index<usize, Output = f32>>(a: &A, b: &B) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// Lerp between `a` and `b` at some distance `t` where t is in [0, 1]
/// and t = 0 returns `a` and t = 1 returns `b`
pub fn lerp<T: Mul<f32, Output = T> + Add<Output = T> + Copy>(t: f32, a: &T, b: &T) -> T {
    *a * (1.0 - t) + *b * t
}
/// Clamp `x` to be between `min` and `max`
pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min { min } else if x > max { max } else { x }
}
/// Compute the direction specified by `theta` and `phi` in the spherical coordinate system
pub fn spherical_dir(sin_theta: f32, cos_theta: f32, phi: f32) -> Vector {
    Vector::new(sin_theta * f32::cos(phi), sin_theta * f32::sin(phi),
                cos_theta)
}
/// Compute the direction specified by `theta` and `phi` in the coordinate system
/// formed by `x`, `y` and `z`
pub fn spherical_dir_coords(sin_theta: f32, cos_theta: f32, phi: f32, x: &Vector, y: &Vector, z: &Vector)
                    -> Vector {
    sin_theta * f32::cos(phi) * *x + sin_theta * f32::sin(phi) * *y + cos_theta * *z
}
/// Compute the value of theta for the vector in the spherical coordinate system
pub fn spherical_theta(v: &vector::Vector) -> f32 {
    f32::acos(clamp(v.z, -1f32, 1f32))
}
/// Compute the value of phi for the vector in the spherical coordinate system
pub fn spherical_phi(v: &vector::Vector) -> f32 {
    match f32::atan2(v.y, v.x) {
        x if x < 0f32 => x + f32::consts::PI * 2.0,
        x             => x,
    }
}
/// Try to solve the quadratic equation `a*t^2 + b*t + c = 0` and return the two
/// real roots if a solution exists
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    let discrim_sqr = b * b - 4.0 * a * c;
    if discrim_sqr < 0.0 {
        None
    } else {
        let discrim = f32::sqrt(discrim_sqr);
        let q = if b < 0.0 {
            -0.5 * (b - discrim)
        } else {
            -0.5 * (b + discrim)
        };
        match (q / a, c / q) {
            (x, y) if x > y => Some((y, x)),
            (x, y)          => Some((x, y)),
        }
    }
}
/// Compute a local ortho-normal coordinate system from a single vector.
pub fn coordinate_system(e1: &Vector) -> (Vector, Vector) {
	let e2 =
        if f32::abs(e1.x) > f32::abs(e1.y) {
            let inv_len = 1.0 / f32::sqrt(e1.x * e1.x + e1.z * e1.z);
            Vector::new(-e1.z * inv_len, 0.0, e1.x * inv_len)
        }
        else {
            let inv_len = 1.0 / f32::sqrt(e1.y * e1.y + e1.z * e1.z);
            Vector::new(0.0, e1.z * inv_len, -e1.y * inv_len)
        };
    let e3 = cross(e1, &e2);
	(e2, e3)
}

#[test]
fn test_cross() {
    let a = Vector::new(1f32, 0f32, 0f32);
    let b = Vector::new(0f32, 1f32, 0f32);
    let c = cross(&a, &b);
    assert!(c == Vector::new(0f32, 0f32, 1f32));
}

#[test]
fn test_dot() {
    let a = Vector::new(1f32, 2f32, 3f32);
    let b = Vector::new(4f32, 5f32, 6f32);
    assert!(dot(&a, &b) == 1f32 * 4f32 + 2f32 * 5f32 + 3f32 * 6f32);
}

