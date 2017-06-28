//! Defines types for operating with floating point and 8 bit RGB colors

use std::f32;
use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};

use linalg;

/// Colorf is a floating point RGBA color type
/// `a` is typically used to store the weight of a color eg. in the
/// render target for multisampling we need to track the weight to
/// normalize in the end and is always initialized to 0
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Colorf {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Colorf {
    /// Create an RGB color
    pub fn new(r: f32, g: f32, b: f32) -> Colorf {
        Colorf { r: r, g: g, b: b, a: 1.0 }
    }
    /// Create an RGB color
    pub fn with_alpha(r: f32, g: f32, b: f32, a: f32) -> Colorf {
        Colorf { r: r, g: g, b: b, a: a }
    }
    /// Create an RGB color using the same value for all channels
    pub fn broadcast(r: f32) -> Colorf {
        Colorf { r: r, g: r, b: r, a: r }
    }
    /// Create a black color
    pub fn black() -> Colorf { Colorf::broadcast(0.0) }
    /// Clamp the color values between [0, 1]
    pub fn clamp(&self) -> Colorf {
        Colorf { r: linalg::clamp(self.r, 0.0, 1.0),
                 g: linalg::clamp(self.g, 0.0, 1.0),
                 b: linalg::clamp(self.b, 0.0, 1.0),
                 a: linalg::clamp(self.a, 0.0, 1.0) }
    }
    /// Compute the luminance of the color
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
    /// Check if the color is black
    pub fn is_black(&self) -> bool {
        self.r == 0f32 && self.g == 0f32 && self.b == 0f32
    }
    /// Check if any of the color channels are NaN
    pub fn has_nans(&self) -> bool {
        f32::is_nan(self.r) || f32::is_nan(self.g) || f32::is_nan(self.b) || f32::is_nan(self.a)
    }
    /// Check if any of the color channels are infinite
    pub fn has_infs(&self) -> bool {
        f32::is_infinite(self.r) || f32::is_infinite(self.g) || f32::is_infinite(self.b) || f32::is_infinite(self.a)
    }
    /// Convert the linear RGB color to sRGB
    pub fn to_srgb(&self) -> Colorf {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = Colorf::broadcast(0.0);
        for i in 0..3 {
            if self[i] <= 0.0031308 {
                srgb[i] = 12.92 * self[i];
            } else {
                srgb[i] = (1.0 + a) * f32::powf(self[i], b) - a;
            }
        }
        srgb
    }
    /// Return the color with values { e^r, e^g, e^b }
    pub fn exp(&self) -> Colorf {
        Colorf { r: f32::exp(self.r), g: f32::exp(self.g),
                 b: f32::exp(self.b), a: f32::exp(self.a) }
    }
}

impl Add for Colorf {
    type Output = Colorf;
    /// Add two colors together
    fn add(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b, a: self.a + rhs.a }
    }
}

impl Sub for Colorf {
    type Output = Colorf;
    /// Subtract the two colors
    fn sub(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r - rhs.r, g: self.g - rhs.g, b: self.b - rhs.b, a: self.a - rhs.a }
    }
}

impl Mul for Colorf {
    type Output = Colorf;
    /// Multiply the two colors
    fn mul(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r * rhs.r, g: self.g * rhs.g, b: self.b * rhs.b, a: self.a * rhs.a }
    }
}

impl Mul<f32> for Colorf {
    type Output = Colorf;
    /// Scale the color by the float
    fn mul(self, rhs: f32) -> Colorf {
        Colorf { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs, a: self.a * rhs }
    }
}

impl Mul<Colorf> for f32 {
    type Output = Colorf;
    /// Scale the color by the float
    fn mul(self, rhs: Colorf) -> Colorf {
        Colorf { r: self * rhs.r, g: self * rhs.g, b: self * rhs.b, a: self * rhs.a }
    }
}

impl Div for Colorf {
    type Output = Colorf;
    /// Divide the channels of one color by another
    fn div(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r / rhs.r, g: self.g / rhs.g, b: self.b / rhs.b, a: self.a / rhs.a }
    }
}

impl Div<f32> for Colorf {
    type Output = Colorf;
    /// Divide the channels of the color by the float
    fn div(self, rhs: f32) -> Colorf {
        Colorf { r: self.r / rhs, g: self.g / rhs, b: self.b / rhs, a: self.a / rhs }
    }
}

impl Neg for Colorf {
    type Output = Colorf;
    /// Negate the color channels
    fn neg(self) -> Colorf {
        Colorf { r: -self.r, g: -self.g, b: -self.b, a: -self.a }
    }
}

impl Index<usize> for Colorf {
    type Output = f32;
    /// Access the channels by index
    /// 
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    /// - 3 = a
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            _ => panic!("Invalid index into color"),
        }
    }
}

impl IndexMut<usize> for Colorf {
    /// Access the channels by index
    /// 
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    /// - 3 = a
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            _ => panic!("Invalid index into color"),
        }
    }
}

