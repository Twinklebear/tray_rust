//! The geometry module defines the Geometry trait implemented by
//! the various geometry in the ray tracer and provides some standard
//! geometry for rendering

use linalg;

pub use self::differential_geometry::DifferentialGeometry;
pub use self::intersection::Intersection;
pub use self::instance::Instance;
pub use self::sphere::Sphere;
pub use self::plane::Plane;
pub use self::bbox::BBox;
pub use self::bvh::BVH;
pub use self::mesh::Mesh;

pub mod differential_geometry;
pub mod intersection;
pub mod instance;
pub mod sphere;
pub mod plane;
pub mod bbox;
pub mod bvh;
pub mod mesh;

/// Trait implemented by geometric primitives
pub trait Geometry {
    /// Test a ray for intersection with the geometry.
    /// The ray should have been previously transformed into the geometry's
    /// object space otherwise the test will be incorrect.
    /// Returns the differential geometry containing the hit information if the
    /// ray hit the object and set's the ray's `max_t` member accordingly
    fn intersect(&self, ray: &mut linalg::Ray) -> Option<DifferentialGeometry>;
}

/// Trait implemented by scene objects that can report an AABB describing their bounds
pub trait Boundable {
    /// Get an AABB reporting the object's bounds in space
    fn bounds(&self) -> BBox;
}

pub trait BoundableGeom: Geometry + Boundable {}
impl<T: ?Sized> BoundableGeom for T where T: Geometry + Boundable {}

