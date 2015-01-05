//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;

use linalg::{Transform, Point, Vector};
use film::{Camera};
use geometry::{Sphere, Instance};
use geometry::Geometry;
use integrator::{HitMarker, Integrator};

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub camera: Arc<Camera>,
    pub instance: Arc<Instance>,
    pub integrator: Arc<Box<Integrator + Send + Sync>>,
    sphere: Arc<Box<Geometry + Send + Sync>>,
}

impl Scene {
    /// Create our (currently) hard-coded scene, passing in the render target
    /// dimensions so we can set the projection matrix for the camera
    pub fn new(w: uint, h: uint) -> Scene {
        let sphere = Arc::new(box Sphere::new(1.5)
                              as Box<Geometry + Send + Sync>);
        Scene {
            camera: Arc::new(Camera::new(Transform::look_at(&Point::new(0.0, 0.0, -10.0),
                &Point::new(0.0, 0.0, 0.0), &Vector::new(0.0, 1.0, 0.0)), 40.0, (w, h))),
            instance: Arc::new(Instance::new(sphere.clone(),
                Transform::translate(&Vector::new(0.0, 2.0, 0.0)))),
            integrator: Arc::new(box HitMarker as Box<Integrator + Send + Sync>),
            sphere: sphere.clone(),
        }
    }
}

