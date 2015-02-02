//! Defines the Path integrator which implements path tracing with
//! explicit light sampling

use std::num::Float;

use scene::Scene;
use linalg;
use linalg::{Point, Ray};
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use light::{Light, OcclusionTester};

/// The path integrator implementing Path tracing with explicit light sampling
/// See [Kajiya, The Rendering Equation](http://dl.acm.org/citation.cfm?id=15902)
#[derive(Copy, Debug)]
pub struct Path {
    min_depth: u32,
    max_depth: u32,
}

impl Path {
    /// Create a new path integrator with the min and max length desired for paths
    pub fn new(min_depth: u32, max_depth: u32) -> Path {
        Path { min_depth: min_depth, max_depth: max_depth }
    }
}

impl Integrator for Path {
    // TODO Stop pretending to be whitted! :)
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection) -> Colorf {
        let bsdf = hit.instance.material.bsdf(hit);
        let w_o = -ray.d;
        // Should we just return this in the tuple as well?
        // TODO: When we add support for multiple lights, iterate over all of them
        let mut occlusion = OcclusionTester::test_points(&Point::origin(), &Point::origin());
        let (li, w_i) = scene.light.sample_incident(&hit.dg.p, &mut occlusion);
        let f = bsdf.eval(&w_o, &w_i, BxDFType::all());
        let mut illum = Colorf::broadcast(0.0);
        if !li.is_black() && !f.is_black() && !occlusion.occluded(scene) {
            // TODO: Divide by pdf once we add that to lights
            illum = f * li * Float::abs(linalg::dot(&w_i, &bsdf.n));
        }
        if ray.depth < self.max_depth {
            illum = illum + self.specular_reflection(scene, ray, &bsdf);
            illum = illum + self.specular_transmission(scene, ray, &bsdf);
        }
        illum
    }
}

