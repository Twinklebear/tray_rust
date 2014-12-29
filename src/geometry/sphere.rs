//! Defines a Sphere type which implements the Geometry trait

use geometry::{Geometry, DifferentialGeometry};
use linalg;
use linalg::{Normal, Point};

/// A sphere with user-specified radius
#[deriving(Copy)]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    /// Create the sphere with the desired radius
    pub fn new(radius: f32) -> Sphere {
        Sphere { radius: radius }
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &mut linalg::Ray, dg: &mut DifferentialGeometry) -> bool {
        // Compute quadratic coefficients for sphere intersection equation
        let a = ray.d.length_sqr();
        let b = 2.0 * linalg::dot(&ray.d, &ray.o);
        let c = ray.o.distance_sqr(&Point::broadcast(0f32)) - self.radius * self.radius;
        // Try to solve the quadratic equation to find the candidate hit t values
        // if there are no solutions then we definitely don't hit the sphere
        let t = match linalg::solve_quadratic(a, b, c) {
            Some(x) => x,
            None => return false,
        };
        // Test that we're within the range of t values the ray is querying
        if t.0 > ray.max_t || t.1 < ray.min_t {
            return false;
        }
        // Find the first t value within the ray's range we hit
        let mut t_hit = t.0;
        if t_hit < ray.min_t {
            t_hit = t.1;
            if t_hit > ray.max_t {
                return false;
            }
        }
        // We have a valid hit if we get here, so fill out the ray max_t and
        // differential geometry info to send back
        ray.max_t = t_hit;
        dg.p = ray.at(t_hit);
        dg.n = Normal::new(dg.p.x, dg.p.y, dg.p.z);
        dg.ng = dg.n;
        true
    }
}

