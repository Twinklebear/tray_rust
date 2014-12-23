use std::num::Float;

use linalg;
use linalg::vector;

/// Normal is a standard 3 component normal but transforms as a normal
/// normal when transformations are applied
#[deriving(Show, Copy, PartialEq, PartialOrd)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Normal {
    /// Initialize the normal and set values for x, y, z
    pub fn new(x: f32, y: f32, z: f32) -> Normal {
        Normal { x: x, y: y, z: z }
    }
    /// Initialize the normal with the same value of x, y, z
    pub fn broadcast(x: f32) -> Normal {
        Normal { x: x, y: x, z: x }
    }
    /// Compute the squared length of the normal
    pub fn length_sqr(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.x
    }
    /// Compute the length of the normal
    pub fn length(&self) -> f32 {
        Float::sqrt(self.length_sqr())
    }
    /// Get a normalized copy of this normal
    pub fn normalized(&self) -> Normal {
        let len = self.length();
        Normal { x: self.x / len, y: self.y / len, z: self.z / len }
    }
    /// Return a normal facing along the same direction as v
    pub fn face_forward(&self, v: &vector::Vector) -> Normal {
        if linalg::dot(self, v) < 0f32 { *self } else { -*self }
    }
}

impl Add<Normal, Normal> for Normal {
    /// Add two normals together
    fn add(self, rhs: Normal) -> Normal {
        Normal { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub<Normal, Normal> for Normal {
    /// Subtract two normals
    fn sub(self, rhs: Normal) -> Normal {
        Normal { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<Normal, Normal> for Normal {
    /// Multiply two normals
    fn mul(self, rhs: Normal) -> Normal {
        Normal { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl Mul<f32, Normal> for Normal {
    /// Scale the normal by some value
    fn mul(self, rhs: f32) -> Normal {
        Normal { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Normal, Normal> for f32 {
    /// Scale the normal by some value
    fn mul(self, rhs: Normal) -> Normal {
        rhs * self
    }
}

impl Div<Normal, Normal> for Normal {
    /// Divide the normals components by the right hand side's components
    fn div(self, rhs: Normal) -> Normal {
        Normal { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z }
    }
}

impl Div<f32, Normal> for Normal {
    /// Divide the normals components by scalar
    fn div(self, rhs: f32) -> Normal {
        Normal { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Neg<Normal> for Normal {
    /// Negate the normal
    fn neg(self) -> Normal {
        Normal { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl Index<uint, f32> for Normal {
    /// Access the normal by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2+ = z
    fn index(&self, i: &uint) -> &f32 {
        match *i {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
}

impl IndexMut<uint, f32> for Normal {
    /// Access the normal by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2+ = z
    fn index_mut(&mut self, i: &uint) -> &mut f32 {
        match *i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => &mut self.z,
        }
    }
}

