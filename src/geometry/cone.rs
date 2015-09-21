//! Defines a Cone at the origin lying along the Z axis which implements the
//! Geometry and Boundable traits
//!
//! # Scene Usage Example
//!

use std::f32;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox};
use linalg::{self, Normal, Vector, Ray, Point};

/// A sphere with user-specified radius located at the origin.
#[derive(Clone, Copy)]
pub struct Cone {
    radius: f32,
    height: f32,
}

impl Cone {
    /// Create a cone with the desired radius and height
    pub fn new(radius: f32, height: f32) -> Cone {
        Cone { radius: radius, height: height }
    }
}

impl Geometry for Cone {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let k = f32::powf(self.radius / self.height, 2.0);
        let a = f32::powf(ray.d.x, 2.0) + f32::powf(ray.d.y, 2.0) - k * f32::powf(ray.d.z, 2.0);
        let b = 2.0 * (ray.d.x * ray.o.x + ray.d.y * ray.o.y - k * ray.d.z * (ray.o.z - self.height));
        let c = f32::powf(ray.o.x, 2.0) + f32::powf(ray.o.y, 2.0) - k * f32::powf(ray.o.z - self.height, 2.0);
        // Try to solve the quadratic equation and find candidate hit t values
        let t = match linalg::solve_quadratic(a, b, c) {
            Some(x) => x,
            None => return None,
        };
        // Test that we're within the range of t values the ray is querying
        if t.0 > ray.max_t || t.1 < ray.min_t {
            return None;
        }
        // Find the first t value within the ray's range we hit
        let mut t_hit = t.0;
        if t_hit < ray.min_t {
            t_hit = t.1;
            if t_hit > ray.max_t {
                return None;
            }
        }
        let mut p = ray.at(t_hit);
        // Test that the hit point is also within the z range
        if p.z < 0.0 || p.z > self.height {
            t_hit = t.1;
            if t_hit > ray.max_t {
                return None;
            }
            p = ray.at(t_hit);
            if p.z < 0.0 || p.z > self.height {
                return None;
            }
        }
        let s = p.z / self.height;
        let dp_du = Vector::new(-f32::consts::PI * 2.0 * p.y, f32::consts::PI * 2.0 * p.x, 0.0);
        let dp_dv = Vector::new(-p.x / (1.0 - s), -p.y / (1.0 - s), self.height);
        let norm = linalg::cross(&dp_du, &dp_dv);
        let n = Normal::new(norm.x, norm.y, norm.z);
        Some(DifferentialGeometry::new(&p, &n, &dp_du, &dp_dv, self))
    }
}

impl Boundable for Cone {
    fn bounds(&self, _: f32, _: f32) -> BBox {
        BBox::span(Point::new(-self.radius, -self.radius, 0.0),
                   Point::new(self.radius, self.radius, self.height))
    }
}

