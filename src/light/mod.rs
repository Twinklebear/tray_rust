//! Defines the light interface implemented by all lights in tray_rust and
//! the OcclusionTester which provides a convenient interface for doing
//! shadow tests for lights

use std::f32;

use linalg;
use film::Colorf;
use scene::Scene;

pub use self::point::Point;

pub mod point;

/// The OcclusionTester provides a simple interface for setting up and executing
/// occlusion queries in the scene
#[derive(Copy, Show)]
pub struct OcclusionTester {
    /// The ray (or ray segment) that the occlusion test is performed on
    pub ray: linalg::Ray,
}

impl OcclusionTester {
    /// Create an occlusion tester to perform the test between two points
    pub fn test_points(a: &linalg::Point, b: &linalg::Point) -> OcclusionTester {
        OcclusionTester { ray: linalg::Ray::segment(a, &(*b - *a), 0.001, 0.999) }
    }
    /// Create an occlusion tester to perform the test along the ray starting at `p`
    /// and in direction `d`
    pub fn test_ray(p: &linalg::Point, d: &linalg::Vector) -> OcclusionTester {
        OcclusionTester { ray: linalg::Ray::segment(p, d, 0.001, f32::INFINITY) }
    }
    /// Perform the occlusion test in the scene
    pub fn occluded(&mut self, scene: &Scene) -> bool {
        if let Some(_) = scene.intersect(&mut self.ray) {
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
    /// Returns the color and incident light direction and fills out the
    /// occlusion tester object
    fn sample_incident(&self, p: &linalg::Point, occlusion: &mut OcclusionTester) -> (Colorf, linalg::Vector);
}

