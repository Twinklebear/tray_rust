//! Defines the Intersection type which stores information about
//! a full intersection, eg. hit info about the geometry and instance
//! that was intersected

use geometry::{Instance, DifferentialGeometry};

/// Stores information about an intersection that occured with some instance
/// of geometry in the scene
#[derive(Copy)]
pub struct Intersection<'a, 'b> {
    /// The differential geometry holding information about the piece of geometry
    /// that was hit
    pub dg: DifferentialGeometry<'a>,
    /// The instance of geometry that was hit
    pub instance: &'b Instance,
}

impl<'a, 'b> Intersection<'a, 'b> {
    /// Construct the Intersection from a potential hit stored in a
    /// Option<DifferentialGeometry>. Returns None if `dg` is None
    /// or if the instance member of `dg` is None
    pub fn new(dg: DifferentialGeometry<'a>, inst: &'b Instance) -> Intersection<'a, 'b> {
        Intersection { dg: dg, instance: inst }
    }
}

