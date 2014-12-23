use std::f32;
use linalg::{point, vector};

/// Ray is a standard 3D ray, starting at origin `o` and heading in direction `d`
/// The min and max points along the ray can be specified with `min_t` and `max_t`
/// `depth` is the recursion depth of the ray
#[deriving(Show, Copy)]
pub struct Ray {
    /// Origin of the ray
    pub o: point::Point,
    /// Direction the ray is heading
    pub d: vector::Vector,
    /// Point along the ray that the actual ray starts at, `p = o + min_t * d`
    pub min_t: f32,
    /// Point along the ray at which it stops, will be inf if the ray is infinite
    pub max_t: f32,
    /// Recursion depth of the ray
    pub depth: i32,
}

impl Ray {
    /// Create a new ray from `o` heading in `d` with infinite length
    pub fn new(o: point::Point, d: vector::Vector) -> Ray {
        Ray { o: o, d: d, min_t: 0f32, max_t: f32::INFINITY, depth: 0 }
    }
    /// Create a new segment ray from `o + min_t * d` to `o + max_t * d`
    pub fn segment(o: point::Point, d: vector::Vector, min_t: f32, max_t: f32) -> Ray {
        Ray { o: o, d: d, min_t: min_t, max_t: max_t, depth: 0}
    }
    /// Create a child ray from the parent starting at `o` and heading in `d`
    pub fn child(&self, o: point::Point, d: vector::Vector) -> Ray {
        Ray { o: o, d: d, min_t: 0f32, max_t: f32::INFINITY, depth: self.depth + 1 }
    }
    /// Create a child ray segment from `o + min_t * d` to `o + max_t * d`
    pub fn child_segment(&self, o: point::Point, d: vector::Vector, min_t: f32, max_t: f32) -> Ray {
        Ray { o: o, d: d, min_t: min_t, max_t: max_t, depth: self.depth + 1}
    }
    /// Evaulate the ray equation at some t value and return the point
    /// returns result of `self.o + t * self.d`
    fn at(&self, t: f32) -> point::Point {
        self.o + self.d * t
    }
}

