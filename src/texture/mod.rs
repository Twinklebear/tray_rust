//! Defines the trait implemented by all textured values

use std::ops::{Add, Mul};

use film::Colorf;

pub use self::image::Image;

pub mod image;

/// Any T which can be copied to return can be computed
/// based on some texture or procedural value.
pub trait Texture<T: Copy> {
    /// Sample the textured value at texture coordinates u,v
    /// at some time. u and v should be in [0, 1]
    fn sample(&self, u: f32, v: f32, time: f32) -> T;
}

fn bilinear_interpolate<T, F>(x: f32, y: f32, get: F) -> T
    where T: Copy + Add<T, Output=T> + Mul<f32, Output=T>,
          F: Fn(u32, u32) -> T
{
    let p00 = (x as u32, y as u32);
    let p10 = (p00.0 + 1, p00.1);
    let p01 = (p00.0, p00.1 + 1);
    let p11 = (p00.0 + 1, p00.1 + 1);

    let s00 = get(p00.0, p00.1);
    let s10 = get(p10.0, p10.1);
    let s01 = get(p01.0, p01.1);
    let s11 = get(p11.0, p11.1);

    let sx = x - p00.0 as f32;
    let sy = y - p00.1 as f32;
    s00 * (1.0 - sx) * (1.0 - sy) + s10 * sx * (1.0 - sy)
        + s01 * (1.0 - sx) * sy + s11 * sx * sy
}

/// A single valued, solid 'color' texture for any T
pub struct Constant<T: Copy> {
    val: T,
}
impl<T: Copy> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant { val: val }
    }
}
impl<T: Copy> Texture<T> for Constant<T> {
    fn sample(&self, _: f32, _: f32, _: f32) -> T {
        self.val
    }
}

pub struct UVColor;
impl Texture<Colorf> for UVColor {
    fn sample(&self, u: f32, v: f32, _: f32) -> Colorf {
        Colorf::new(u, v, 0.0)
    }
}

