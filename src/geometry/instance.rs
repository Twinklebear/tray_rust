//! Defines an instance of some piece of geometry in the scene, instances
//! can re-use loaded geometry but apply different transformations and materials
//! to them
//!
//! # Scene Usage Example
//! An instance is an instantion of geometry in your scene. The instance needs to know
//! its type (emitter/receiver), its name along with the geometry to use, material to apply
//! and transformation to place the object in the scene. The instances are specified in the
//! objects list in the scene file.
//!
//! The transform for the object is specified in the order in which the transformations should
//! be applied. For information on emitters see the emitter documentation.
//!
//! ```json
//! "objects": [
//!     {
//!         "name": "back_wall",
//!         "type": "receiver",
//!         "material": "white_wall",
//!         "geometry": {
//!             "type": "plane"
//!         },
//!         "transform": [
//!             {
//!                 "type": "scaling",
//!                 "scaling": 15.0
//!             },
//!             {
//!                 "type": "translate",
//!                 "translation": [0.0, 1.0, 20.]
//!             }
//!         ]
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;

use geometry::{Intersection, Boundable, BBox, BoundableGeom, Receiver, Emitter,
               SampleableGeom};
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
               transform: Transform, tag: String) -> Instance {
        Instance::Receiver(Receiver::new(geom, material, transform, tag))
    }
    /// Create an instance of the geometry in the scene that will emit and receive light
    pub fn area_light(geom: Arc<SampleableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               emission: Colorf, transform: Transform, tag: String) -> Instance {
        Instance::Emitter(Emitter::area(geom, material, emission, transform, tag))
    }
    pub fn point_light(pos: Point, emission: Colorf, tag: String) ->  Instance {
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
    /// Get the tag for this instance
    pub fn tag(&self) -> &str {
        match self {
            &Instance::Emitter(ref e) => &e.tag[..],
            &Instance::Receiver(ref r) => &r.tag[..],
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

