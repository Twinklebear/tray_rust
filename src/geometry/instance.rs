//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them

use geometry::{Geometry, DifferentialGeometry};
use linalg;

/// Defines an instance of some geometry with its own transform and material
pub struct Instance<'a> {
    /// The geometry that's being instanced
    geom: &'a (Geometry + 'a),
    /// TODO Don't store both transform and inverse, just use the transform
    /// backwards when coming back up
    /// The transform to world space
    transform: linalg::Transform,
}

impl<'a> Instance<'a> {
    /// Create a new instance of some geometry in the scene
    pub fn new(geom: &'a (Geometry + 'a), transform: linalg::Transform) -> Instance<'a> {
        Instance { geom: geom, transform: transform }
    }
}

impl<'a> Geometry for Instance<'a> {
    fn intersect(&self, ray: &mut linalg::Ray) -> Option<DifferentialGeometry> {
        let mut local = self.transform.inv_mul_ray(ray);
        let mut dg = match self.geom.intersect(&mut local) {
            Some(dg) => dg,
            None => return None,
        };
        ray.max_t = local.max_t;
        dg.p = self.transform * dg.p;
        dg.n = self.transform * dg.n;
        dg.ng = self.transform * dg.ng;
        dg.instance = Some(self);
        Some(dg)
    }
}


