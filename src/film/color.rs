//! Defines types for operating with floating point and 8 bit RGB colors

use std::num::Float;
use linalg;

/// Colorf is a floating point RGB color type
#[deriving(Show, Copy, PartialEq)]
pub struct Colorf {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Colorf {
    /// Create an RGB color
    pub fn new(r: f32, g: f32, b: f32) -> Colorf {
        Colorf { r: r, g: g, b: b }
    }
    /// Create an RGB color using the same value for all channels
    pub fn broadcast(r: f32) -> Colorf {
        Colorf { r: r, g: r, b: r }
    }
    /// Clamp the color values between [0, 1]
    pub fn clamp(&self) -> Colorf {
        Colorf { r: linalg::clamp(self.r, 0.0, 1.0),
                 g: linalg::clamp(self.g, 0.0, 1.0),
                 b: linalg::clamp(self.b, 0.0, 1.0) }
    }
    /// Compute the luminance of the color
    pub fn luminance(&self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
    /// Check if the color is black
    pub fn is_black(&self) -> bool {
        self.r == 0f32 && self.g == 0f32 && self.b == 0f32
    }
    /// Convert the linear RGB color to sRGB
    pub fn to_srgb(&self) -> Colorf {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = Colorf::broadcast(0.0);
        for i in range(0u, 3) {
            if self[i] <= 0.0031308 {
                srgb[i] = 12.92 * self[i];
            } else {
                srgb[i] = (1.0 + a) * Float::powf(self[i], b) - a;
            }
        }
        srgb
    }
    /// Return the color with values { e^r, e^g, e^b }
    pub fn exp(&self) -> Colorf {
        Colorf { r: Float::exp(self.r), g: Float::exp(self.g), b: Float::exp(self.b) }
    }
}

impl Add<Colorf, Colorf> for Colorf {
    /// Add two colors together
    fn add(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b }
    }
}

impl Sub<Colorf, Colorf> for Colorf {
    /// Subtract the two colors
    fn sub(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r - rhs.r, g: self.g - rhs.g, b: self.b - rhs.b }
    }
}

impl Mul<Colorf, Colorf> for Colorf {
    /// Multiply the two colors
    fn mul(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r * rhs.r, g: self.g * rhs.g, b: self.b * rhs.b }
    }
}

impl Mul<f32, Colorf> for Colorf {
    /// Scale the color by the float
    fn mul(self, rhs: f32) -> Colorf {
        Colorf { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs }
    }
}

impl Div<Colorf, Colorf> for Colorf {
    /// Divide the channels of one color by another
    fn div(self, rhs: Colorf) -> Colorf {
        Colorf { r: self.r / rhs.r, g: self.g / rhs.g, b: self.b / rhs.b }
    }
}

impl Div<f32, Colorf> for Colorf {
    /// Divide the channels of the color by the float
    fn div(self, rhs: f32) -> Colorf {
        Colorf { r: self.r / rhs, g: self.g / rhs, b: self.b / rhs }
    }
}

impl Neg<Colorf> for Colorf {
    /// Negate the color channels
    fn neg(self) -> Colorf {
        Colorf { r: -self.r, g: -self.g, b: -self.b }
    }
}

impl Index<uint, f32> for Colorf {
    /// Access the channels by index
    /// 
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    fn index(&self, i: &uint) -> &f32 {
        match *i {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            _ => panic!("Invalid index into color"),
        }
    }
}

impl IndexMut<uint, f32> for Colorf {
    /// Access the channels by index
    /// 
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    fn index_mut(&mut self, i: &uint) -> &mut f32 {
        match *i {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            _ => panic!("Invalid index into color"),
        }
    }
}

