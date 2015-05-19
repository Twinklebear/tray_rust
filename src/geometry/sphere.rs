//! Defines a Sphere type which implements the Geometry trait

use std::f32;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox};
use linalg::{self, Normal, Vector, Ray, Point};

/// A sphere with user-specified radius
#[derive(Clone, Copy)]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    /// Create a sphere with the desired radius
    pub fn new(radius: f32) -> Sphere {
        Sphere { radius: radius }
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        // Compute quadratic coefficients for sphere intersection equation
        let a = ray.d.length_sqr();
        let b = 2.0 * linalg::dot(&ray.d, &ray.o);
        let c = linalg::dot(&ray.o, &ray.o) - self.radius * self.radius;
        // Try to solve the quadratic equation to find the candidate hit t values
        // if there are no solutions then we definitely don't hit the sphere
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
        // We have a valid hit if we get here, so fill out the ray max_t and
        // differential geometry info to send back
        ray.max_t = t_hit;
        let p = ray.at(t_hit);
        let n = Normal::new(p.x, p.y, p.z);
        let theta = f32::acos(linalg::clamp(p.z / self.radius, -1.0, 1.0));

        // Compute derivatives for point vs. parameterization
        let inv_z = 1.0 / f32::sqrt(p.x * p.x + p.y * p.y);
        let cos_phi = p.x * inv_z;
        let sin_phi = p.y * inv_z;
        let dp_du = Vector::new(-p.y, p.x, 0.0) * f32::consts::PI_2;
        let dp_dv = Vector::new(p.z * cos_phi, p.z * sin_phi,
                                -self.radius * f32::sin(theta)) * f32::consts::PI;

        Some(DifferentialGeometry::new(&p, &n, &dp_du, &dp_dv, self))
    }
}

impl Boundable for Sphere {
    fn bounds(&self) -> BBox {
        BBox::span(Point::new(-self.radius, -self.radius, -self.radius),
                   Point::new(self.radius, self.radius, self.radius))
    }
}

