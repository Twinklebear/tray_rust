//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;
use std::path::Path;

use linalg::{Transform, Point, Vector, Ray};
use film::{Camera, Colorf};
use geometry::{Sphere, Plane, Instance, Intersection, BVH, Mesh, Disk};
use material::{Matte, Glass, Metal, Merl};
use integrator::{self, Integrator};

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub camera: Camera,
    pub bvh: BVH<Instance>,
    pub integrator: Box<Integrator + Send + Sync>,
}

impl Scene {
    // Create our (currently) hard-coded scene, passing in the render target
    // dimensions so we can set the projection matrix for the camera
    // TODO: Should take a JSON scene file to parse
    //pub fn new(w: usize, h: usize) -> Scene {}

    pub fn rust_logo_with_friends(w: usize, h: usize) -> Scene {
        let sphere = Arc::new(Sphere::new(1.0));
        let plane = Arc::new(Plane);
        let mut models = Mesh::load_obj(Path::new("./rust-logo.obj"));
        let rust_logo = Arc::new(models.remove("rust_logo").unwrap());

        let mut models = Mesh::load_obj(Path::new("./buddha.obj"));
        let buddha = Arc::new(models.remove("buddha").unwrap());

        let mut models = Mesh::load_obj(Path::new("./dragon.obj"));
        let dragon = Arc::new(models.remove("dragon").unwrap());

        let oxidized_steel = Arc::new(Merl::load_file(Path::new("black-oxidized-steel.binary")));
        let gold_paint = Arc::new(Merl::load_file(Path::new("gold-metallic-paint.binary")));
        let blue_acrylic = Arc::new(Merl::load_file(Path::new("blue-acrylic.binary")));
        let white_wall = Arc::new(Matte::new(&Colorf::new(0.740063, 0.742313, 0.733934), 1.0));
        let red_wall = Arc::new(Matte::new(&Colorf::new(0.366046, 0.0371827, 0.0416385), 1.0));
        let green_wall = Arc::new(Matte::new(&Colorf::new(0.162928, 0.408903, 0.0833759), 1.0));

        let light_color = Colorf::broadcast(125.0) * Colorf::new(0.780131, 0.780409, 0.775833);

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
            Instance::receiver(plane.clone(), green_wall.clone(),
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
            Instance::receiver(rust_logo, oxidized_steel, Transform::translate(&Vector::new(0.0, 6.0, 12.0))
                * Transform::rotate_z(180.0) * Transform::rotate_y(90.0)
                * Transform::scale(&Vector::broadcast(12.0)), "rust_logo"),
            Instance::receiver(buddha, gold_paint, Transform::translate(&Vector::new(-9.0, 0.5, 7.55))
                * Transform::rotate_z(-140.0) * Transform::rotate_x(90.0)
                * Transform::scale(&Vector::broadcast(17.0)), "buddha"),
            Instance::receiver(dragon, blue_acrylic, Transform::translate(&Vector::new(8.25, 3.5, 3.7))
                * Transform::rotate_z(120.0) * Transform::rotate_x(90.0)
                * Transform::scale(&Vector::broadcast(13.0)), "dragon"),
            Instance::area_light(sphere.clone(), white_wall.clone(), light_color,
                Transform::translate(&Vector::new(0.0, -5.0, 23.0)), "light"),
        ];
        Scene {
            camera: Camera::new(Transform::look_at(&Point::new(0.0, -52.0, 12.0),
                &Point::new(0.0, 0.0, 12.0), &Vector::new(0.0, 0.0, 1.0)), 28.0, (w, h)),
            bvh: BVH::new(4, instances),
            integrator: Box::new(integrator::Path::new(4, 8)),
        }
    }
    //// Load the Small PT scene
    pub fn small_pt(w: usize, h: usize) -> Scene {
        let sphere = Arc::new(Sphere::new(1.0));
        let plane = Arc::new(Plane);

        let white_wall = Arc::new(Matte::new(&Colorf::new(1.0, 1.0, 1.0), 1.0));
        let red_wall = Arc::new(Matte::new(&Colorf::new(1.0, 0.2, 0.2), 1.0));
        let blue_wall = Arc::new(Matte::new(&Colorf::new(0.2, 0.2, 1.0), 1.0));
        let metal = Arc::new(Metal::new(&Colorf::new(0.155265, 0.116723, 0.138381),
                                        &Colorf::new(4.82835, 3.12225, 2.14696), 0.1));
        let light_color = Colorf::new(0.780131, 0.780409, 0.775833) * 100.0;

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
            Instance::receiver(sphere.clone(), metal,
                Transform::translate(&Vector::new(-6.0, 8.0, 5.0))
                * Transform::scale(&Vector::broadcast(5.0)), "metal_sphere"),
            // The glass sphere
            Instance::receiver(sphere.clone(),
                Arc::new(Glass::new(&Colorf::broadcast(1.0), &Colorf::broadcast(1.0), 1.52)),
                Transform::translate(&Vector::new(6.0, -2.0, 5.0))
                * Transform::scale(&Vector::broadcast(5.0)), "glass_sphere"),
            // The light
            Instance::area_light(sphere.clone(), white_wall.clone(), light_color,
                Transform::translate(&Vector::new(0.0, 0.0, 22.0)), "light")
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

