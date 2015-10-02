//! Defines the light interface implemented by all lights in tray_rust and
//! the OcclusionTester which provides a convenient interface for doing
//! shadow tests for lights

use std::f32;

use linalg::{Point, Vector, Ray};
use film::Colorf;
use scene::Scene;

/// The OcclusionTester provides a simple interface for setting up and executing
/// occlusion queries in the scene
#[derive(Clone, Copy, Debug)]
pub struct OcclusionTester {
    /// The ray (or ray segment) that the occlusion test is performed on
    pub ray: Ray,
}

impl OcclusionTester {
    /// Create an occlusion tester to perform the test between two points
    pub fn test_points(a: &Point, b: &Point, time: f32) -> OcclusionTester {
        OcclusionTester { ray: Ray::segment(a, &(*b - *a), 0.001, 0.999, time) }
    }
    /// Create an occlusion tester to perform the test along the ray starting at `p`
    /// and in direction `d`
    pub fn test_ray(p: &Point, d: &Vector, time: f32) -> OcclusionTester {
        OcclusionTester { ray: Ray::segment(p, d, 0.001, f32::INFINITY, time) }
    }
    /// Perform the occlusion test in the scene
    pub fn occluded(&self, scene: &Scene) -> bool {
        let mut r = self.ray;
        if let Some(_) = scene.intersect(&mut r) {
            true
        } else {
            false
        }
    }
}

/// Trait implemented by all lights in tray_rust. Provides methods for sampling
/// the light and in the future ones for checking if it's a delta light, computing
/// its power and so on.
pub trait Light {
    /// Sample the illumination from the light arriving at the point `p`
    /// Returns the color, incident light direction, pdf and occlusion tester object
    /// `samples` will be used to randomly sample the light.
    fn sample_incident(&self, p: &Point, samples: &(f32, f32), time: f32)
        -> (Colorf, Vector, f32, OcclusionTester);
    /// Determine if the light is described by a delta distribution
    fn delta_light(&self) -> bool;
    /// Compute the PDF for sampling the point with incident direction `w_i`
    fn pdf(&self, p: &Point, w_i: &Vector, time: f32) -> f32;
}

