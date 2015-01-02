//! Defines the DifferentialGeometry type which is used to pass information
//! about the hit piece of geometry back from the intersection to the shading

use linalg::{Point, Normal};
use geometry::Geometry;

/// Stores information about a hit piece of geometry of some object in the scene
#[deriving(Copy)]
pub struct DifferentialGeometry<'a> {
    /// The hit point 
    pub p: Point,
    /// The shading normal
    pub n: Normal,
    /// The geometry normal
    pub ng: Normal,
    /// The geometry that was hit
    pub geom: &'a (Geometry + 'a),
}

impl<'a> DifferentialGeometry<'a> {
    /// Initialize the differential geometry with 0 values for all fields
    /// and None for the hit geometry
    pub fn new(p: &Point, n: &Normal, ng: &Normal, geom: &'a (Geometry + 'a))
               -> DifferentialGeometry<'a> {
        DifferentialGeometry { p: *p, n: *n, ng: *ng, geom: geom }
    }
}

