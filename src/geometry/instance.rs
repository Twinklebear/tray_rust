//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them

use std::sync::Arc;
use std::string::ToString;

use geometry::{Intersection, Boundable, BBox, BoundableGeom};
use material::Material;
use linalg;

/// Defines an instance of some geometry with its own transform and material
pub struct Instance {
    /// The geometry that's being instanced. TODO: We must Box for now but this
    /// restriction will be lifted later
    geom: Arc<BoundableGeom + Send + Sync>,
    /// The material being used by this instance.
    pub material: Arc<Material + Send + Sync>,
    /// The transform to world space
    transform: linalg::Transform,
    /// Tag to identify the instance
    pub tag: String,
}

impl Instance {
    /// Create a new instance of some geometry in the scene
    pub fn new(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: linalg::Transform, tag: &str)
               -> Instance {
        Instance { geom: geom, material: material, transform: transform, tag: tag.to_string() }
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
        dg.dp_du = self.transform * dg.dp_du;
        dg.dp_dv = self.transform * dg.dp_dv;
        Some(Intersection::new(dg, self))
    }
}

impl Boundable for Instance {
    fn bounds(&self) -> BBox {
        self.transform * self.geom.bounds()
    }
}

