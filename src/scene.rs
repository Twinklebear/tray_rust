//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;

use linalg::{Transform, Point, Vector, Ray};
use film::{Camera, Colorf};
use geometry::{Sphere, Plane, Instance};
use material::{Matte, SpecularMetal, Glass, Material};
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
    pub fn new(w: usize, h: usize) -> Scene {
        let sphere = Arc::new(Box::new(Sphere::new(1.0)) as Box<Geometry + Send + Sync>);
        let plane = Arc::new(Box::new(Plane) as Box<Geometry + Send + Sync>);
        let white_wall = Arc::new(Box::new(Matte::new(&Colorf::new(1.0, 1.0, 1.0), 1.0)) as Box<Material + Send + Sync>);
        let red_wall = Arc::new(Box::new(Matte::new(&Colorf::new(1.0, 0.2, 0.2), 1.0)) as Box<Material + Send + Sync>);
        let blue_wall = Arc::new(Box::new(Matte::new(&Colorf::new(0.2, 0.2, 1.0), 1.0)) as Box<Material + Send + Sync>);
        let instances = vec![
            // The reflective sphere
            Instance::new(sphere.clone(),
            Arc::new(Box::new(SpecularMetal::new(&Colorf::new(0.155, 0.116, 0.138),
                                                 &Colorf::new(4.828, 3.122, 2.146)))
                     as Box<Material + Send + Sync>), Transform::translate(&Vector::new(-6.0, 8.0, 5.0))
                    * Transform::scale(&Vector::broadcast(5.0))),
            // The glass sphere
            Instance::new(sphere.clone(),
            Arc::new(Box::new(Glass::new(&Colorf::broadcast(1.0), &Colorf::broadcast(1.0), 1.52))
                     as Box<Material + Send + Sync>), Transform::translate(&Vector::new(6.0, -2.0, 5.0))
                    * Transform::scale(&Vector::broadcast(5.0))),
            // The back wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 20.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(90.0)),
            // The left wall
            Instance::new(plane.clone(), red_wall.clone(), Transform::translate(&Vector::new(-15.0, 0.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(90.0)),
            // The right wall
            Instance::new(plane.clone(), blue_wall.clone(), Transform::translate(&Vector::new(15.0, 0.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(-90.0)),
            // The top wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 0.0, 24.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(180.0)),
            // The bottom wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 0.0, 0.0))
                          * Transform::scale(&Vector::broadcast(32.0)))
        ];
        Scene {
            camera: Arc::new(Camera::new(Transform::look_at(&Point::new(0.0, -60.0, 12.0),
                &Point::new(0.0, 0.0, 12.0), &Vector::new(0.0, 0.0, 1.0)), 30.0, (w, h))),
            instances: Arc::new(instances),
            integrator: Arc::new(Box::new(Whitted::new(8)) as Box<Integrator + Send + Sync>),
            light: Arc::new(Box::new(light::Point::new(&Point::new(0.0, 0.0, 22.0), &Colorf::broadcast(150.0)))
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

