//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;

use linalg::{Transform, Point, Vector, Ray};
use film::{Camera, Colorf};
use geometry::{Sphere, Instance};
use material::{Matte, Material};
use geometry::{Geometry, Intersection};
use integrator::{Whitted, Integrator};
use light;
use light::Light;

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub camera: Arc<Camera>,
    /// TODO: Only one instance for now
    pub instances: Arc<Vec<Instance>>,
    pub integrator: Arc<Box<Integrator + Send + Sync>>,
    /// TODO: Only one light for now
    pub light: Arc<Box<Light + Send + Sync>>,
    sphere: Arc<Box<Geometry + Send + Sync>>,
}

impl Scene {
    /// Create our (currently) hard-coded scene, passing in the render target
    /// dimensions so we can set the projection matrix for the camera
    pub fn new(w: uint, h: uint) -> Scene {
        let sphere = Arc::new(box Sphere::new(1.0) as Box<Geometry + Send + Sync>);
        let instances = vec![Instance::new(sphere.clone(),
            Arc::new(box Matte::new(&Colorf::new(1.0, 0.0, 0.0), 0.3)
                     as Box<Material + Send + Sync>), Transform::scale(&Vector::broadcast(1.5))),
            Instance::new(sphere.clone(),
            Arc::new(box Matte::new(&Colorf::new(0.0, 0.0, 1.0), 0.8)
                     as Box<Material + Send + Sync>), Transform::translate(&Vector::new(2.0, -2.0, -1.0)))];
        Scene {
            camera: Arc::new(Camera::new(Transform::look_at(&Point::new(0.0, 0.0, -10.0),
                &Point::new(0.0, 0.0, 0.0), &Vector::new(0.0, 1.0, 0.0)), 40.0, (w, h))),
            instances: Arc::new(instances),
            integrator: Arc::new(box Whitted::new(8) as Box<Integrator + Send + Sync>),
            light: Arc::new(box light::Point::new(&Point::new(0.0, 1.5, -4.0), &Colorf::broadcast(30.0))
                            as Box<Light + Send + Sync>),
            sphere: sphere.clone(),
        }
    }
    /// Test the ray for intersections against the objects in the scene.
    /// Returns Some(Intersection) if an intersection was found and None if not.
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        // We can always return the next hit found since the ray's max_t value is updated
        // after an intersection is found. Thus if we find another hit we know that one
        // occured before any previous ones. TODO: Replace with a BVH
        self.instances.iter().fold(None, |p, ref i|
                                    match i.intersect(ray) {
                                        Some(h) => Some(h),
                                        None => p,
                                    })
    }
}

