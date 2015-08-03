//! A receiver is an instance of geometry that does not emit any light

use std::sync::Arc;
use geometry::{Boundable, BBox, BoundableGeom, DifferentialGeometry};
use material::Material;
use linalg;

/// An instance of geometry in the scene that only receives light
pub struct Receiver {
    /// The geometry that's being instanced.
    geom: Arc<BoundableGeom + Send + Sync>,
    /// The material being used by this instance.
    pub material: Arc<Material + Send + Sync>,
    /// The transform to world space
    transform: linalg::Transform,
    /// Tag to identify the instance
    pub tag: String,
}

impl Receiver {
    /// Create a new instance of some geometry in the scene
    pub fn new(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: linalg::Transform, tag: String) -> Receiver {
        Receiver { geom: geom, material: material, transform: transform, tag: tag }
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut linalg::Ray) -> Option<(DifferentialGeometry, &Material)> {
        let mut local = self.transform.inv_mul_ray(ray);
        let mut dg = match self.geom.intersect(&mut local) {
            Some(dg) => dg,
            None => return None,
        };
        ray.max_t = local.max_t;
        dg.p = self.transform * dg.p;
        dg.n = self.transform * dg.n;
        dg.ng = self.transform * dg.ng;
        dg.dp_du = self.transform * dg.dp_du;
        dg.dp_dv = self.transform * dg.dp_dv;
        Some((dg, &*self.material))
    }
}

impl Boundable for Receiver {
    fn bounds(&self) -> BBox {
        self.transform * self.geom.bounds()
    }
}

