use std::num::Float;

/// Vector is a standard 3 component vector
#[deriving(Show, Copy)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

impl Vector {
    /// Initialize the vector and set values for x, y, z
    pub fn new(x: f32, y: f32, z: f32) -> Vector {
        Vector { x: x, y: y, z: z }
    }
    /// Initialize the vector with the same value of x, y, z
    pub fn broadcast(x: f32) -> Vector {
        Vector { x: x, y: x, z: x }
    }
    /// Compute the squared length of the vector
    pub fn length_sqr(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.x
    }
    /// Compute the length of the vector
    pub fn length(&self) -> f32 {
        Float::sqrt(self.length_sqr())
    }
}

#[test]
fn test_len_sqr() {
    let v = Vector::broadcast(1f32);
    assert!(v.length_sqr() == 3f32);
}

