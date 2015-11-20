//! Defines the DifferentialGeometry type which is used to pass information
//! about the hit piece of geometry back from the intersection to the shading

use linalg::{self, Point, Normal, Vector};
use geometry::Geometry;

/// Stores information about a hit piece of geometry of some object in the scene
#[derive(Clone, Copy)]
pub struct DifferentialGeometry<'a> {
    /// The hit point
    pub p: Point,
    /// The shading normal
    pub n: Normal,
    /// The geometry normal
    pub ng: Normal,
    /// Derivative of the point with respect to the u parameterization coord of the surface
    pub dp_du: Vector,
    /// Derivative of the point with respect to the v parameterization coord of the surface
    pub dp_dv: Vector,
    /// The geometry that was hit
    pub geom: &'a (Geometry + 'a),
}

impl<'a> DifferentialGeometry<'a> {
    /// Setup the differential geometry. Note that the normal will be computed
    /// using cross(dp_du, dp_dv)
    pub fn new(p: &Point, ng: &Normal, dp_du: &Vector, dp_dv: &Vector,
               geom: &'a (Geometry + 'a))
               -> DifferentialGeometry<'a> {
        let n = linalg::cross(dp_du, dp_dv).normalized();
        DifferentialGeometry { p: *p, n: Normal::new(n.x, n.y, n.z), ng: ng.normalized(),
                               dp_du: *dp_du, dp_dv: *dp_dv, geom: geom }
    }
    /// Setup the differential geometry using the normal passed for the surface normal
    pub fn with_normal(p: &Point, n: &Normal, dp_du: &Vector, dp_dv: &Vector,
               geom: &'a (Geometry + 'a))
               -> DifferentialGeometry<'a> {
        let nn = n.normalized();
        DifferentialGeometry { p: *p, n: nn, ng: nn,
                               dp_du: *dp_du, dp_dv: *dp_dv, geom: geom }
    }
}

