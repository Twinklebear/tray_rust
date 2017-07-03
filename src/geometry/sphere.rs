//! Defines a Sphere at the origin which implements the Geometry, Boundable and Sampleable traits
//!
//! # Scene Usage Example
//! The sphere takes a single parameter to specify its radius.
//!
//! ```json
//! "geometry": {
//!     "type": "sphere",
//!     "radius": 2.5
//! }
//! ```

use std::f32;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox, Sampleable};
use linalg::{self, Normal, Vector, Ray, Point};
use mc;

/// A sphere with user-specified radius located at the origin.
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
        // TODO: It doesn't make sense that dp_du x dp_dv and n point it such different
        // directions, they should at least point in a similar direction
        // Doing dp_dv x dp_du gives the same as normal, kind of as we'd expect since they're
        // facing opposite directions, but it doesn't explain why this would be wrong
        let u = match f32::atan2(p.x, p.y) / (2.0 * f32::consts::PI) {
            x if x < 0.0 => x + 1.0,
            x => x,
        };
        let v = theta / f32::consts::PI;
        let dp_du = Vector::new(-f32::consts::PI * 2.0 * p.y, f32::consts::PI * 2.0 * p.x, 0.0);
        let dp_dv = Vector::new(p.z * cos_phi, p.z * sin_phi,
                                -self.radius * f32::sin(theta)) * f32::consts::PI;

        Some(DifferentialGeometry::with_normal(&p, &n, u, v, ray.time, &dp_du, &dp_dv, self))
    }
}

impl Boundable for Sphere {
    fn bounds(&self, _: f32, _: f32) -> BBox {
        BBox::span(Point::new(-self.radius, -self.radius, -self.radius),
                   Point::new(self.radius, self.radius, self.radius))
    }
}

impl Sampleable for Sphere {
    fn sample_uniform(&self, samples: &(f32, f32)) -> (Point, Normal) {
        let p = Point::broadcast(0.0) + self.radius * mc::uniform_sample_sphere(samples);
        (p, Normal::new(p.x, p.y, p.z).normalized())
    }
    /// Sample the object using the probability density of the solid angle
    /// from `p` to the sampled point on the surface.
    /// Returns the sampled point and the surface normal at that point
    fn sample(&self, p: &Point, samples: &(f32, f32)) -> (Point, Normal) {
        // If the point is inside the sphere just sample uniformly
        let dist_sqr = p.distance_sqr(&Point::broadcast(0.0));
        // The PDF is uniform if we're insidfe the sphere
        if dist_sqr - self.radius * self.radius < 0.0001 {
            self.sample_uniform(samples)
        }
        else {
            let w_z = (Point::broadcast(0.0) - *p).normalized();
            let (w_x, w_y) = linalg::coordinate_system(&w_z);
            // Compute theta and phi for samples in the cone of the sphere seen from `p`
            let cos_theta_max = f32::sqrt(f32::max(0.0, 1.0 - self.radius * self.radius / dist_sqr));
            let mut ray = Ray::new(p, &mc::uniform_sample_cone_frame(samples, cos_theta_max,
                                                                     &w_x, &w_y, &w_z).normalized(), 0.0);
            // Try to hit the sphere with this ray to sample it
            match self.intersect(&mut ray) {
                Some(dg) => (dg.p, dg.ng),
                None => {
                    // If we miss find where we kind of hit at the edge
                    let t = linalg::dot(&(Point::broadcast(0.0) - *p), &ray.d);
                    let p = ray.at(t);
                    (p, Normal::new(p.x, p.y, p.z).normalized())
                }
            }
        }
    }
    /// Compute the sphere's surface area
    fn surface_area(&self) -> f32 {
        4.0 * f32::consts::PI * self.radius
    }
    /// Compute the PDF that the ray from `p` with direction `w_i` intersects
    /// the shape
    fn pdf(&self, p: &Point, _: &Vector) -> f32 {
        let dist_sqr = p.distance_sqr(&Point::broadcast(0.0));
        // The PDF is uniform if we're insidfe the sphere
        if dist_sqr - self.radius * self.radius < 0.0001 {
            1.0 / self.surface_area()
        } else {
            let cos_theta_max = f32::sqrt(f32::max(0.0, 1.0 - self.radius * self.radius / dist_sqr));
            mc::uniform_cone_pdf(cos_theta_max)
        }
    }
}

