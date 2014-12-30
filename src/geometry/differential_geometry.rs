//! Defines the DifferentialGeometry type which is used to pass information
//! about the hit piece of geometry back from the intersection to the shading

use linalg::{Point, Normal};
use geometry::{Geometry, Instance};

/// Stores information about a hit piece of geometry of some object in the scene
#[deriving(Copy)]
pub struct DifferentialGeometry<'a, 'b> {
    /// The hit point 
    pub p: Point,
    /// The shading normal
    pub n: Normal,
    /// The geometry normal
    pub ng: Normal,
    /// The geometry that was hit
    pub geom: &'a (Geometry + 'a),
    /// The instance of geometry that was hit
    pub instance: Option<&'b Instance<'b>>,
}

impl<'a, 'b> DifferentialGeometry<'a, 'b> {
    /// Initialize the differential geometry with 0 values for all fields
    /// and None for the hit geometry
    pub fn new(p: &Point, n: &Normal, ng: &Normal, geom: &'a (Geometry + 'a),
               instance: Option<&'b Instance<'b>>) -> DifferentialGeometry<'a, 'b> {
        DifferentialGeometry { p: *p, n: *n, ng: *ng, geom: geom, instance: instance }
    }
}

