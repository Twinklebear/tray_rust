//! The geometry module defines the Geometry trait implemented by
//! the various geometry in the ray tracer and provides some standard
//! geometry for rendering

use linalg;

pub use self::differential_geometry::DifferentialGeometry;
pub use self::instance::Instance;
pub use self::sphere::Sphere;

pub mod differential_geometry;
pub mod instance;
pub mod sphere;

pub trait Geometry {
    /// Test a ray for intersection with the geometry
    /// The ray should have been previously transformed into the geometry's
    /// object space otherwise the test will be incorrect. Returns true if a
    /// hit is found, fills out the differential geometry with the hit point
    /// information and sets the ray's `max_t` to the hit `t`
    /// Note: the values set on `dg` will be in object space and must be transformed
    /// back out to world space before shading
    fn intersect(&self, ray: &mut linalg::Ray, dg: &mut DifferentialGeometry) -> bool;
}

