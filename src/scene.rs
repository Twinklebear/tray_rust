//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;
use std::path::Path;

use linalg::{Transform, Point, Vector, Ray};
use film::{Camera, Colorf};
use geometry::{Sphere, Plane, Instance, Intersection, BVH};
use material::{Matte, Glass, Metal};
use integrator::{self, Integrator};

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub camera: Camera,
    pub bvh: BVH<Instance>,
    pub integrator: Box<Integrator + Send + Sync>,
}

impl Scene {
    /// Create our (currently) hard-coded scene, passing in the render target
    /// dimensions so we can set the projection matrix for the camera
    pub fn new(w: usize, h: usize) -> Scene {
        let sphere = Arc::new(Sphere::new(1.0));
        let plane = Arc::new(Plane);

        let white_wall = Arc::new(Matte::new(&Colorf::new(1.0, 1.0, 1.0), 1.0));
        let red_wall = Arc::new(Matte::new(&Colorf::new(1.0, 0.2, 0.2), 1.0));
        let blue_wall = Arc::new(Matte::new(&Colorf::new(0.2, 0.2, 1.0), 1.0));
        let metal = Arc::new(Metal::new(&Colorf::new(0.155265, 0.116723, 0.138381),
                                        &Colorf::new(4.82835, 3.12225, 2.14696), 0.1));
        let light_color = Colorf::broadcast(100.0) * Colorf::new(0.780131, 0.780409, 0.775833);

        let instances = vec![
            // The back wall
            Instance::receiver(plane.clone(), white_wall.clone(),
                Transform::translate(&Vector::new(0.0, 20.0, 12.0))
                * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(90.0),
                "back_wall"),
            // The left wall
            Instance::receiver(plane.clone(), red_wall.clone(),
                Transform::translate(&Vector::new(-15.0, 0.0, 12.0))
                * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(90.0),
                "left_wall"),
            // The right wall
            Instance::receiver(plane.clone(), blue_wall.clone(),
                Transform::translate(&Vector::new(15.0, 0.0, 12.0))
                * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(-90.0),
                "right_wall"),
            // The top wall
            Instance::receiver(plane.clone(), white_wall.clone(),
                Transform::translate(&Vector::new(0.0, 0.0, 24.0))
                * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(180.0),
                "top_wall"),
            // The bottom wall
            Instance::receiver(plane.clone(), white_wall.clone(),
                Transform::translate(&Vector::new(0.0, 0.0, 0.0))
                * Transform::scale(&Vector::broadcast(32.0)), "bottom_wall"),
            // The reflective sphere
            Instance::receiver(sphere.clone(), metal, Transform::translate(&Vector::new(-6.0, 8.0, 5.0))
                * Transform::scale(&Vector::broadcast(5.0)), "metal_sphere"),
            // The glass sphere
            Instance::receiver(sphere.clone(),
                Arc::new(Glass::new(&Colorf::broadcast(1.0), &Colorf::broadcast(1.0), 1.52)),
                Transform::translate(&Vector::new(6.0, -2.0, 5.0))
                * Transform::scale(&Vector::broadcast(5.0)), "glass_sphere"),
            // The light
            Instance::area_light(sphere.clone(), white_wall.clone(), light_color,
                Transform::translate(&Vector::new(0.0, 0.0, 22.0)), "light"),
            //Instance::point_light(Point::new(0.0, 0.0, 22.0), light_color * 2, "light"),
            //Instance::point_light(Point::new(10.0, 0.0, 12.0), light_color / 2.0, "light2"),
        ];
        Scene {
            camera: Camera::new(Transform::look_at(&Point::new(0.0, -60.0, 12.0),
                &Point::new(0.0, 0.0, 12.0), &Vector::new(0.0, 0.0, 1.0)), 30.0, (w, h)),
            bvh: BVH::new(4, instances),
            integrator: Box::new(integrator::Path::new(4, 8)),
        }
    }
    /// Test the ray for intersections against the objects in the scene.
    /// Returns Some(Intersection) if an intersection was found and None if not.
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
}

