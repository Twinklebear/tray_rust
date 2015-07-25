//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them

use std::sync::Arc;

use geometry::{Intersection, Boundable, BBox, BoundableGeom, Receiver, Emitter};
use material::Material;
use linalg::{Transform, Point, Ray};
use film::Colorf;

/// Defines an instance of some geometry with its own transform and material
pub enum Instance {
    Emitter(Emitter),
    Receiver(Receiver),
}

impl Instance {
    /// Create an instance of the geometry in the scene that will only receive light.
    pub fn receiver(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: Transform, tag: &str) -> Instance {
        Instance::Receiver(Receiver::new(geom, material, transform, tag))
    }
    /// Create an instance of the geometry in the scene that will emit and receive light
    /// TODO: area lights
    /*
    pub fn area_light(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: linalg::Transform, tag: &str) -> Instance {
        Instance::Emitter(Emitter::area(geom, material, transform, tag))
    }
    */
    pub fn point_light(pos: Point, emission: Colorf, tag: &str) ->  Instance {
        Instance::Emitter(Emitter::point(pos, emission, tag))
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let hit = match self {
            &Instance::Emitter(ref e) => e.intersect(ray),
            &Instance::Receiver(ref r) => r.intersect(ray),
        };
        match hit {
            Some((dg, mat)) => Some(Intersection::new(dg, self, mat)),
            None => None,
        }
    }
}

impl Boundable for Instance {
    fn bounds(&self) -> BBox {
        match self {
            &Instance::Emitter(ref e) => e.bounds(),
            &Instance::Receiver(ref r) => r.bounds(),
        }
    }
}

