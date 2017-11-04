//! Defines the trait implemented by all textured values

use std::ops::{Add, Mul};

use film::Colorf;

pub use self::image::Image;
pub use self::animated_image::AnimatedImage;

pub mod image;
pub mod animated_image;

/// scalars or Colors can be computed on some image texture
/// or procedural generator
pub trait Texture {
    /// Sample the textured value at texture coordinates u,v
    /// at some time. u and v should be in [0, 1]
    fn sample_f32(&self, u: f32, v: f32, time: f32) -> f32;
    fn sample_color(&self, u: f32, v: f32, time: f32) -> Colorf;
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

/// A single valued, solid scalar texture
pub struct ConstantScalar {
    val: f32,
}
impl ConstantScalar {
    pub fn new(val: f32) -> ConstantScalar {
        ConstantScalar { val: val }
    }
}
impl Texture for ConstantScalar {
    fn sample_f32(&self, _: f32, _: f32, _: f32) -> f32 {
        self.val
    }
    fn sample_color(&self, _: f32, _: f32, _: f32) -> Colorf {
        Colorf::broadcast(self.val)
    }
}

/// A single valued, solid color texture
pub struct ConstantColor {
    val: Colorf,
}
impl ConstantColor {
    pub fn new(val: Colorf) -> ConstantColor {
        ConstantColor { val: val }
    }
}
impl Texture for ConstantColor {
    fn sample_f32(&self, _: f32, _: f32, _: f32) -> f32 {
        self.val.luminance()
    }
    fn sample_color(&self, _: f32, _: f32, _: f32) -> Colorf {
        self.val
    }
}

pub struct UVColor;
impl Texture for UVColor {
    fn sample_f32(&self, u: f32, v: f32, _: f32) -> f32 {
        Colorf::new(u, v, 0.0).luminance()
    }
    fn sample_color(&self, u: f32, v: f32, _: f32) -> Colorf {
        Colorf::new(u, v, 0.0)
    }
}

