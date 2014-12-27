use std::f32;
use std::num::{FloatMath, Float};

// Re-export the linalg types from the internal modules
pub use self::vector::Vector;
pub use self::normal::Normal;
pub use self::point::Point;
pub use self::ray::Ray;
pub use self::matrix4::Matrix4;
pub use self::transform::Transform;

pub mod vector;
pub mod normal;
pub mod point;
pub mod ray;
pub mod matrix4;
pub mod transform;

/// Compute the cross product of two vectors
pub fn cross<A: Index<uint, f32>, B: Index<uint, f32>>(a: &A, b: &B) -> vector::Vector {
    vector::Vector::new(a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0])
}
/// Compute the dot product of two vectors
pub fn dot<A: Index<uint, f32>, B: Index<uint, f32>>(a: &A, b: &B) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// Lerp between `a` and `b` at some distance `t` where t is in [0, 1]
/// and t = 0 returns `a` and t = 1 returns `b`
pub fn lerp<T: Mul<f32, T> + Add<T, T> + Copy<T>>(t: f32, a: &T, b: &T) -> T {
    *a * (1f32 - t) + *b * t
}
/// Clamp `x` to be between `min` and `max`
pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min { min } else if x > max { max } else { x }
}
/// Convert `d` in degrees to radians
pub fn radians(d: f32) -> f32 {
    f32::consts::PI / 180f32 * d
}
/// Convert `r` in radians to degrees
pub fn degrees(r: f32) -> f32 {
    180f32 / f32::consts::PI * r
}
/// Compute the direction specified by `theta` and `phi` in the spherical coordinate system
pub fn spherical_dir(sin_theta: f32, cos_theta: f32, phi: f32) -> vector::Vector {
    vector::Vector::new(sin_theta * FloatMath::cos(phi), sin_theta * FloatMath::sin(phi),
                cos_theta)
}
/// Compute the value of theta for the vector in the spherical coordinate system
pub fn spherical_theta(v: &vector::Vector) -> f32 {
    FloatMath::acos(clamp(v.z, -1f32, 1f32))
}
/// Compute the value of phi for the vector in the spherical coordinate system
pub fn spherical_phi(v: &vector::Vector) -> f32 {
    match FloatMath::atan2(v.y, v.x) {
        x if x < 0f32 => x + f32::consts::PI_2,
        x             => x,
    }
}
/// Try to solve the quadratic equation `a*t^2 + b*t + c = 0` and return the two
/// real roots if a solution exists
pub fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    let discrim = Float::sqrt(b * b - 4f32 * a * c);
    if Float::is_nan(discrim) {
        None
    } else {
        let q = if b < 0f32 { -0.5f32 * (b - discrim) } else { -0.5f32 * (b + discrim) };
        match (q / a, c / q) {
            (x, y) if x > y => Some((y, x)),
            (x, y)          => Some((x, y)),
        }
    }
}

#[test]
fn test_cross() {
    let a = vector::Vector::new(1f32, 0f32, 0f32);
    let b = vector::Vector::new(0f32, 1f32, 0f32);
    let c = cross(&a, &b);
    assert!(c == vector::Vector::new(0f32, 0f32, 1f32));
}

#[test]
fn test_dot() {
    let a = vector::Vector::new(1f32, 2f32, 3f32);
    let b = vector::Vector::new(4f32, 5f32, 6f32);
    assert!(dot(&a, &b) == 1f32 * 4f32 + 2f32 * 5f32 + 3f32 * 6f32);
}

