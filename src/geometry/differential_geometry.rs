//! Defines the DifferentialGeometry type which is used to pass information
//! about the hit piece of geometry back from the intersection to the shading

use linalg;
use geometry;

/// Stores information about a hit piece of geometry of some object in the scene
#[deriving(Copy)]
pub struct DifferentialGeometry<'a> {
    /// The hit point 
    pub p: linalg::Point,
    /// The shading normal
    pub n: linalg::Normal,
    /// The geometry normal
    pub ng: linalg::Normal,
    /// The geometry that was hit
    pub geom: &'a (geometry::Geometry + 'a),
}
    

