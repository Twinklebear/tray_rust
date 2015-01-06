//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them

use std::sync::Arc;

use geometry::{Geometry, Intersection};
use material::Material;
use linalg;

/// Defines an instance of some geometry with its own transform and material
pub struct Instance {
    /// The geometry that's being instanced. TODO: We must Box for now but this
    /// restriction will be lifted later
    geom: Arc<Box<Geometry + Send + Sync>>,
    /// The material being used by this instance.
    pub material: Arc<Box<Material + Send + Sync>>,
    /// The transform to world space
    transform: linalg::Transform,
}

impl Instance {
    /// Create a new instance of some geometry in the scene
    pub fn new(geom: Arc<Box<Geometry + Send + Sync>>, material: Arc<Box<Material + Send + Sync>>,
               transform: linalg::Transform)
               -> Instance {
        Instance { geom: geom, material: material, transform: transform }
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut linalg::Ray) -> Option<Intersection> {
        let mut local = self.transform.inv_mul_ray(ray);
        let mut dg = match self.geom.intersect(&mut local) {
            Some(dg) => dg,
            None => return None,
        };
        ray.max_t = local.max_t;
        dg.p = self.transform * dg.p;
        dg.n = self.transform * dg.n;
        dg.ng = self.transform * dg.ng;
        Some(Intersection::new(dg, self))
    }
}

