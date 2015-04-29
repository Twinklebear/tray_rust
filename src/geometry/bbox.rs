//! Provide an Axis-Aligned Bounding Box type, BBox, with an optimized intersection test
//! targeted for usage in a BVH
//! TODO: Should I also implement the Geometry trait?

use std::f32;
use std::ops::{Index, IndexMut};

use linalg;
use linalg::{Point, Vector, Ray, Axis};

/// A box between the min and max points
#[derive(Clone, Copy, Debug)]
pub struct BBox {
    pub min: Point,
    pub max: Point,
}

impl BBox {
    /// Create a new degenerate box
    pub fn new() -> BBox {
        BBox { min: Point::broadcast(f32::INFINITY), max: Point::broadcast(f32::NEG_INFINITY) }
    }
    /// Create a new box containing only the point passed
    pub fn singular(p: Point) -> BBox {
        BBox { min: p, max: p }
    }
    /// Create a new box spanning [min, max]
    pub fn span(min: Point, max: Point) -> BBox {
        BBox { min: min, max: max }
    }
    /// Get a box representing the union of this box with the one passed
    pub fn box_union(&self, b: &BBox) -> BBox {
        BBox { min: Point::new(f32::min(self.min.x, b.min.x), f32::min(self.min.y, b.min.y),
                               f32::min(self.min.z, b.min.z)),
               max: Point::new(f32::max(self.max.x, b.max.x), f32::max(self.max.y, b.max.y),
                               f32::max(self.max.z, b.max.z))
        }
    }
    /// Get a box that contains the passed point, by expanding this box to reach the point
    pub fn point_union(&self, p: &Point) -> BBox {
        BBox { min: Point::new(f32::min(self.min.x, p.x), f32::min(self.min.y, p.y),
                               f32::min(self.min.z, p.z)),
               max: Point::new(f32::max(self.max.x, p.x), f32::max(self.max.y, p.y),
                               f32::max(self.max.z, p.z))
        }
    }
    /// Compute the axis along which the box is longest
    pub fn max_extent(&self) -> Axis {
        let d = self.max - self.min;
        if d.x > d.y && d.x > d.z {
            Axis::X
        } else if d.y > d.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }
    /// Compute the point in the box at some t value along each axis
    pub fn lerp(&self, tx: f32, ty: f32, tz: f32) -> Point {
        Point::new(linalg::lerp(tx, &self.min.x, &self.max.x), linalg::lerp(ty, &self.min.y, &self.max.y),
                   linalg::lerp(tz, &self.min.z, &self.max.z))
    }
    /// Find the position of the point relative to the box, with `min` being the origin
    pub fn offset(&self, p: &Point) -> Vector {
        (*p - self.min) / (self.max - self.min)
    }
    /// Optimized ray-box intersection test, for use in the BVH traversal where we have
    /// pre-computed the ray's inverse direction and which directions are negative, indicated
    /// by a 1 for negative and 0 for non-negative
    /// Returns true if the box was hit
    pub fn fast_intersect(&self, r: &Ray, inv_dir: &Vector, neg_dir: &[usize; 3]) -> bool {
        // Check X & Y intersection
        let mut tmin = (self[neg_dir[0]].x - r.o.x) * inv_dir.x;
        let mut tmax = (self[1 - neg_dir[0]].x - r.o.x) * inv_dir.x;
        let tymin = (self[neg_dir[1]].y - r.o.y) * inv_dir.y;
        let tymax = (self[1 - neg_dir[1]].y - r.o.y) * inv_dir.y;
        if tmin > tymax || tymin > tmax {
            return false;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        // Check Z intersection
        let tzmin = (self[neg_dir[2]].z - r.o.z) * inv_dir.z;
        let tzmax = (self[1 - neg_dir[2]].z - r.o.z) * inv_dir.z;
        if tmin > tzmax || tzmin > tmax {
            return false;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }
        tmin < r.max_t && tmax > r.min_t
    }
}

impl Index<usize> for BBox {
    type Output = Point;
    /// Access the BBox's min/max points by index
    ///
    /// - 0 = min
    /// - 1 = max
    fn index(&self, i: usize) -> &Point {
        match i {
            0 => &self.min,
            1 => &self.max,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl IndexMut<usize> for BBox {
    /// Access the BBox's min/max points by index
    ///
    /// - 0 = min
    /// - 1 = max
    fn index_mut(&mut self, i: usize) -> &mut Point {
        match i {
            0 => &mut self.min,
            1 => &mut self.max,
            _ => panic!("Invalid index into point"),
        }
    }
}

