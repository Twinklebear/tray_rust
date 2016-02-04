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
//!                 "translation": [0.0, 1.0, 20]
//!             }
//!         ]
//!     },
//!     ...
//! ]
//! ```
//!
//! # Object Group Example
//! You can also specify groups of objects to have the same transformation applied to all of them.
//! This is done with a 'group' type object followed by a list of objects in the group. For a full
//! example see `scenes/cornell_box.json`.
//!
//! ```json
//! "objects": [
//!     {
//!         "name": "my_group",
//!         "type": "group",
//!         "transform": [
//!             {
//!                 "type": "translate",
//!                 "translation": [0.0, 1.0, 20]
//!             }
//!         ],
//!         "objects": [
//!             ...
//!         ]
//!     },
//!     ...
//! ]
//! ```
//!

use std::sync::Arc;

use geometry::{Intersection, Boundable, BBox, BoundableGeom, Receiver, Emitter,
               SampleableGeom};
use material::Material;
use linalg::{Ray, AnimatedTransform};
use film::AnimatedColor;

/// Defines an instance of some geometry with its own transform and material
pub enum Instance {
    Emitter(Emitter),
    Receiver(Receiver),
}

impl Instance {
    /// Create an instance of the geometry in the scene that will only receive light.
    pub fn receiver(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: AnimatedTransform, tag: String) -> Instance {
        Instance::Receiver(Receiver::new(geom, material, transform, tag))
    }
    /// Create an instance of the geometry in the scene that will emit and receive light
    pub fn area_light(geom: Arc<SampleableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               emission: AnimatedColor, transform: AnimatedTransform, tag: String) -> Instance {
        Instance::Emitter(Emitter::area(geom, material, emission, transform, tag))
    }
    /// Create a point light at the origin that is transformed by `transform` to its location
    /// in the world
    pub fn point_light(transform: AnimatedTransform, emission: AnimatedColor, tag: String) ->  Instance {
        Instance::Emitter(Emitter::point(transform, emission, tag))
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let hit = match *self {
            Instance::Emitter(ref e) => e.intersect(ray),
            Instance::Receiver(ref r) => r.intersect(ray),
        };
        match hit {
            Some((dg, mat)) => Some(Intersection::new(dg, self, mat)),
            None => None,
        }
    }
    /// Get the tag for this instance
    pub fn tag(&self) -> &str {
        match *self {
            Instance::Emitter(ref e) => &e.tag[..],
            Instance::Receiver(ref r) => &r.tag[..],
        }
    }
    /// Get the transform for this instance
    pub fn get_transform(&self) -> &AnimatedTransform {
        match *self {
            Instance::Emitter(ref e) => e.get_transform(),
            Instance::Receiver(ref r) => r.get_transform()
        }
    }
    /// Set the transform for this instance
    pub fn set_transform(&mut self, transform: AnimatedTransform) {
        match *self {
            Instance::Emitter(ref mut e) => e.set_transform(transform),
            Instance::Receiver(ref mut r) => r.set_transform(transform)
        }
    }
}

impl Boundable for Instance {
    fn bounds(&self, start: f32, end: f32) -> BBox {
        match self {
            &Instance::Emitter(ref e) => e.bounds(start, end),
            &Instance::Receiver(ref r) => r.bounds(start, end),
        }
    }
}

