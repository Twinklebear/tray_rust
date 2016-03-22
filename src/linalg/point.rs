use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};
use linalg::{Vector, Axis};

/// Point is a standard 3 component point but transforms as a point
/// point when transformations are applied
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    /// Initialize the point and set values for x, y, z
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point { x: x, y: y, z: z }
    }
    /// Initialize the point with the same value of x, y, z
    pub fn broadcast(x: f32) -> Point {
        Point { x: x, y: x, z: x }
    }
    /// Initialize a point to be all 0 values
    pub fn origin() -> Point {
        Point::broadcast(0.0)
    }
    /// Compute the squared distance between this point and another
    pub fn distance_sqr(&self, a: &Point) -> f32 {
        (*self - *a).length_sqr()
    }
    /// Compute the distance between this point and another
    pub fn distance(&self, a: &Point) -> f32 {
        (*self - *a).length()
    }
}

impl Add for Point {
    type Output = Point;
    /// Add two points together
    fn add(self, rhs: Point) -> Point {
        Point { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    /// Add two points together
    fn add(self, rhs: Vector) -> Point {
        Point { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub for Point {
    type Output = Vector;
    /// Subtract two points to get the vector between them
    fn sub(self, rhs: Point) -> Vector {
        Vector { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;
    /// Subtract a vector from a point, translating the point by -vector
    fn sub(self, rhs: Vector) -> Point {
        Point { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    /// Scale the point by some value
    fn mul(self, rhs: f32) -> Point {
        Point { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;
    /// Scale the vector by some value
    fn mul(self, rhs: Point) -> Point {
        Point { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z }
    }
}

impl Mul<Vector> for Point {
    type Output = Point;
    /// Scale the vector by some value
    fn mul(self, rhs: Vector) -> Point {
        Point { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl Div for Point {
    type Output = Point;
    /// Divide the points components by the right hand side's components
    fn div(self, rhs: Point) -> Point {
        Point { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z }
    }
}

impl Div<f32> for Point {
    type Output = Point;
    /// Divide the points components by scalar
    fn div(self, rhs: f32) -> Point {
        Point { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Neg for Point {
    type Output = Point;
    /// Negate the point
    fn neg(self) -> Point {
        Point { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl Index<usize> for Point {
    type Output = f32;
    /// Access the point by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2 = z
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl Index<Axis> for Point {
    type Output = f32;
    /// Access the point by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2 = z
    fn index(&self, i: Axis) -> &f32 {
        match i {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl IndexMut<usize> for Point {
    /// Access the point by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2 = z
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}

#[test]
fn test_distance_sqr() {
    let a = Point::new(0f32, 0f32, 0f32);
    let b = Point::new(3f32, 4f32, 0f32);
    assert!(b.distance_sqr(&a) == 25f32);
}

