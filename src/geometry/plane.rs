//! Defines a plane centered at the origin spanning [-1, -1] to [1, 1] with a normal along [0, 0, 1]
//!
//! # Scene Usage Example
//! The plane takes no parameters, to configure its size and position specify the
//! transformations to apply to the instance of geometry.
//!
//! ```json
//! "geometry": {
//!     "type": "plane"
//! }
//! ```

use std::f32;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox};
use linalg::{Normal, Vector, Ray, Point};

/// A plane centered at the origin spanning [-1, -1] to [1, 1] with a normal along [0, 0, 1]
#[derive(Clone, Copy)]
pub struct Plane;

impl Geometry for Plane {
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
        if p.x >= -1.0 && p.x <= 1.0 && p.y >= -1.0 && p.y <= 1.0 {
            ray.max_t = t;
            let n = Normal::new(0.0, 0.0, 1.0);
            let dp_du = Vector::new(2.0, 0.0, 0.0);
            let dp_dv = Vector::new(0.0, 2.0, 0.0);
            Some(DifferentialGeometry::new(&p, &n, &dp_du, &dp_dv, self))
        } else {
            None
        }
    }
}

impl Boundable for Plane {
    fn bounds(&self, _: f32, _: f32) -> BBox {
        BBox::span(Point::new(-1.0, -1.0, 0.0), Point::new(1.0, 1.0, 0.0))
    }
}

