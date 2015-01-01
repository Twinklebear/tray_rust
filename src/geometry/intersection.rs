//! Defines the Intersection type which stores information about
//! a full intersection, eg. hit info about the geometry and instance
//! that was intersected

use linalg::{Point, Normal};
use geometry::{Geometry, Instance, DifferentialGeometry};

/// Stores information about an intersection that occured with some instance
/// of geometry in the scene
#[deriving(Copy)]
pub struct Intersection<'a, 'b> {
    /// The hit point 
    pub p: Point,
    /// The shading normal
    pub n: Normal,
    /// The geometry normal
    pub ng: Normal,
    /// The geometry that was hit
    pub geom: &'a (Geometry + 'a),
    /// The instance of geometry that was hit
    pub instance: &'b Instance<'b>,
}

impl<'a, 'b> Intersection<'a, 'b> {
    /// Construct the Intersection from a potential hit stored in a
    /// Option<DifferentialGeometry>. Returns None if `dg` is None
    /// or if the instance member of `dg` is None
    pub fn from_diffgeom(dg: Option<DifferentialGeometry<'a, 'b>>) -> Option<Intersection<'a, 'b>> {
        if let Some(d) = dg {
            if let Some(inst) = d.instance {
                return Some(Intersection { p: d.p, n: d.n, ng: d.ng, geom: d.geom, instance: inst });
            }
        }
        None
    }
}

