//! The geometry module defines the Geometry trait implemented by
//! the various geometry in the ray tracer and provides some standard
//! geometry for rendering
//!
//! # Scene Usage Example
//! All geometry will appear within an object specification and requires the type
//! of geometry being specified along with any parameters for that geometry.
//!
//! An instance has a geometry along with additional information like a material
//! and transformation to place it in the world, see the instance module for more.
//!
//! ```json
//! "objects": [
//!     {
//!          "type": "The_Instance_Type",
//!          ...
//!          "geometry": {
//!              "type": "The_Geometry_Type",
//!              ...
//!          }
//!     },
//!     ...
//! ]
//! ```

use linalg::{Point, Vector, Ray, Normal};

pub use self::differential_geometry::DifferentialGeometry;
pub use self::intersection::Intersection;
pub use self::instance::Instance;
pub use self::sphere::Sphere;
pub use self::disk::Disk;
pub use self::plane::Plane;
pub use self::cone::Cone;
pub use self::bbox::BBox;
pub use self::bvh::BVH;
pub use self::mesh::Mesh;
pub use self::receiver::Receiver;
pub use self::emitter::Emitter;

pub mod differential_geometry;
pub mod intersection;
pub mod instance;
pub mod sphere;
pub mod disk;
pub mod plane;
pub mod cone;
pub mod bbox;
pub mod bvh;
pub mod mesh;
pub mod receiver;
pub mod emitter;

/// Trait implemented by geometric primitives
pub trait Geometry {
    /// Test a ray for intersection with the geometry.
    /// The ray should have been previously transformed into the geometry's
    /// object space otherwise the test will be incorrect.
    /// Returns the differential geometry containing the hit information if the
    /// ray hit the object and set's the ray's `max_t` member accordingly
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry>;
}

/// Trait implemented by scene objects that can report an AABB describing their bounds
pub trait Boundable {
    /// Get an AABB reporting the object's bounds over the time period
    /// The default implementation assumes the object isn't animated and
    /// simply returns its bounds. This is kind of a hack to use
    /// the BVH for animated geomtry (instances) and non-animated geometry (triangles).
    fn bounds(&self, start: f32, end: f32) -> BBox;
}

/// Trait implemented by geometry that can sample a point on its surface
pub trait Sampleable {
    /// Uniformly sample a position and normal on the surface using the samples passed
    fn sample_uniform(&self, samples: &(f32, f32)) -> (Point, Normal);
    /// Sample the object using the probability density of the solid angle
    /// from `p` to the sampled point on the surface.
    /// Returns the sampled point and the surface normal at that point
    fn sample(&self, p: &Point, samples: &(f32, f32)) -> (Point, Normal);
    /// Return the surface area of the shape
    fn surface_area(&self) -> f32;
    /// Compute the PDF that the ray from `p` with direction `w_i` intersects
    /// the shape
    fn pdf(&self, p: &Point, w_i: &Vector) -> f32;
}

pub trait BoundableGeom: Geometry + Boundable {}
impl<T: ?Sized> BoundableGeom for T where T: Geometry + Boundable {}

pub trait SampleableGeom: Geometry + Boundable + Sampleable {}
impl<T: ?Sized> SampleableGeom for T where T: Geometry + Boundable + Sampleable {}

