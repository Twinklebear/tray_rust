//! Defines the Whitted integrator which implements Whitted recursive ray tracing

use std::num::Float;

use scene::Scene;
use linalg;
use linalg::{Point, Ray};
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use light::{Light, OcclusionTester};

/// The Whitted integrator implementing the Whitted recursive ray tracing algorithm
/// See [An improved illumination model for shaded display](http://dl.acm.org/citation.cfm?id=358882)
#[derive(Copy, Show)]
pub struct Whitted {
    /// The maximum recursion depth for rays
    max_depth: u32,
}

impl Whitted {
    /// Create a new Whitted integrator with the desired maximum recursion depth for rays
    pub fn new(max_depth: u32) -> Whitted { Whitted { max_depth: max_depth } }
}

impl Integrator for Whitted {
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection) -> Colorf {
        let bsdf = hit.instance.material.bsdf(hit);
        let w_o = -ray.d;
        // Should we just return this in the tuple as well?
        let mut occlusion = OcclusionTester::test_points(&Point::broadcast(0.0), &Point::broadcast(0.0));
        let (li, w_i) = scene.light.sample_incident(&hit.dg.p, &mut occlusion);
        let c = bsdf.eval(&w_o, &w_i, BxDFType::all());
        // TODO: Need specular reflection and refraction
        if !li.is_black() && !c.is_black() { //&& !occlusion.occluded(scene) {
            // TODO: Divide by pdf once we add that to lights
            c * li * Float::abs(linalg::dot(&w_i, &bsdf.n))
        } else {
            Colorf::broadcast(0.0)
        }
    }
}

