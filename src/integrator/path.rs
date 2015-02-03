//! Defines the Path integrator which implements path tracing with
//! explicit light sampling

use std::num::Float;
use std::rand::{Rng, StdRng};

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
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection, rng: &mut StdRng) -> Colorf {
        let bsdf = hit.instance.material.bsdf(hit);
        let w_o = -ray.d;
        let junk_samples = [0.0; 3];
        let (li, w_i, pdf, occlusion) = scene.light.sample_incident(&hit.dg.p, &junk_samples[]);
        let f = bsdf.eval(&w_o, &w_i, BxDFType::all());
        let mut illum = Colorf::broadcast(0.0);
        if !li.is_black() && !f.is_black() && !occlusion.occluded(scene) {
            illum = f * li * Float::abs(linalg::dot(&w_i, &bsdf.n)) / pdf;
        }
        if ray.depth < self.max_depth {
            illum = illum + self.specular_reflection(scene, ray, &bsdf, rng);
            illum = illum + self.specular_transmission(scene, ray, &bsdf, rng);
        }
        illum
    }
}

