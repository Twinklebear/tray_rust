//! Defines a rectangle centered at the origin, specified by its horizontal
//! and vertical lengths
//!
//! # Scene Usage Example
//! The rectangle takes two parameters, specifying its width and height. The
//! rectangle will be centered at the origin and will have its normal facing
//! along [0, 0, 1]
//!
//! ```json
//! "geometry": {
//!     "type": "rectangle",
//!     "width": 1.2,
//!     "height" 2.5
//! }
//! ```

use std::f32;

use geometry::{Geometry, DifferentialGeometry, Boundable, Sampleable, BBox};
use linalg::{self, Normal, Vector, Ray, Point};

/// A rectangle centered at the origin spanning [-width / 2, -height / 2]
/// to [width / 2, height / 2] with a normal along [0, 0, 1]
#[derive(Clone, Copy)]
pub struct Rectangle {
    width: f32,
    height: f32,
}

impl Rectangle {
    /// Create a new rectangle with the desired width and height
    pub fn new(width: f32, height: f32) -> Rectangle {
        Rectangle { width: width, height: height }
    }
}

impl Geometry for Rectangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        // If the ray is perpindicular to the normal it can't intersect
        if f32::abs(ray.d.z) < 1e-8 {
            return None;
        }
        // Test for intersection against an infinite plane. Later we will
        // check that the hit found here is in the finite plane's extent
        let t = -ray.o.z / ray.d.z;
        if t < ray.min_t || t > ray.max_t {
            return None;
        }
        let p = ray.at(t);
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        if p.x >= -half_width && p.x <= half_width && p.y >= -half_height && p.y <= half_height {
            ray.max_t = t;
            let n = Normal::new(0.0, 0.0, 1.0);
            let dp_du = Vector::new(1.0, 0.0, 0.0);
            let dp_dv = Vector::new(0.0, 1.0, 0.0);
            Some(DifferentialGeometry::new(&p, &n, &dp_du, &dp_dv, self))
        } else {
            None
        }
    }
}

impl Boundable for Rectangle {
    fn bounds(&self, _: f32, _: f32) -> BBox {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        BBox::span(Point::new(-half_width, -half_height, 0.0), Point::new(half_width, half_height, 0.0))
    }
}

impl Sampleable for Rectangle {
    /// Uniform sampling for a rect is simple: just scale the two samples into the
    /// rectangle's space and return them as the x,y coordinates of the point chosen
    fn sample_uniform(&self, samples: &(f32, f32)) -> (Point, Normal) {
        (Point::new(samples.0 * self.width - self.width / 2.0, samples.1 * self.height - self.height / 2.0, 0.0),
         Normal::new(0.0, 0.0, 1.0))
    }
    fn sample(&self, _: &Point, samples: &(f32, f32)) -> (Point, Normal) {
        self.sample_uniform(samples)
    }
    /// Compute the sphere's surface area
    fn surface_area(&self) -> f32 {
        self.width * self.height
    }
    /// Compute the PDF that the ray from `p` with direction `w_i` intersects
    /// the shape. This is the same as disk for computing PDF, we just use the
    /// rectangle's surface area instead
    fn pdf(&self, p: &Point, w_i: &Vector) -> f32 {
        // Time doesn't matter here, we're already in the object's space so we're moving
        // with it so to speak
        let mut ray = Ray::segment(&p, &w_i, 0.001, f32::INFINITY, 0.0);
        match self.intersect(&mut ray) {
            Some(d) => {
                let w = -*w_i;
                let pdf = p.distance_sqr(&ray.at(ray.max_t))
                    / (f32::abs(linalg::dot(&d.n, &w)) * self.surface_area());
                if f32::is_finite(pdf) { pdf } else { 0.0 }
            },
            None => 0.0
        }
    }
}

