//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them

use geometry::{Geometry, DifferentialGeometry};
use linalg;

/// Defines an instance of some geometry with its own transform and material
pub struct Instance<'a> {
    /// The geometry that's being instanced
    geom: &'a (Geometry + 'a),
    /// The transform to world space
    transform: linalg::Transform,
    /// The inverse transform, to object space
    inv_transform: linalg::Transform,
}

impl<'a> Geometry for Instance<'a> {
    fn intersect(&self, ray: &mut linalg::Ray, dg: &mut DifferentialGeometry) -> bool {
        let mut local = self.inv_transform * *ray;
        if self.geom.intersect(&mut local, dg) {
            ray.max_t = local.max_t;
            dg.p = self.transform * dg.p;
            dg.n = self.transform * dg.n;
            dg.ng = self.transform * dg.ng;
            true
        } else {
            false
        }
    }
}


