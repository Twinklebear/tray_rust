//! Defines the Whitted integrator which implements Whitted recursive ray tracing

use std::f32;
use rand::StdRng;

use scene::Scene;
use linalg;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use light::Light;
use sampler::Sampler;

/// The Whitted integrator implementing the Whitted recursive ray tracing algorithm
/// See [Whitted, An improved illumination model for shaded display](http://dl.acm.org/citation.cfm?id=358882)
#[derive(Clone, Copy, Debug)]
pub struct Whitted {
    /// The maximum recursion depth for rays
    max_depth: u32,
}

impl Whitted {
    /// Create a new Whitted integrator with the desired maximum recursion depth for rays
    pub fn new(max_depth: u32) -> Whitted { Whitted { max_depth: max_depth } }
}

impl Integrator for Whitted {
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection, sampler: &mut Sampler,
                    rng: &mut StdRng) -> Colorf {
        let bsdf = hit.instance.material.bsdf(hit);
        let w_o = -ray.d;
        let mut sample_2d = [(0.0, 0.0)];
        sampler.get_samples_2d(&mut sample_2d[..], rng);
        // TODO: When we add support for multiple lights, iterate over all of them
        let (li, w_i, pdf, occlusion) = scene.light.sample_incident(&hit.dg.p, &sample_2d[0]);
        let f = bsdf.eval(&w_o, &w_i, BxDFType::all());
        let mut illum = Colorf::broadcast(0.0);
        if !li.is_black() && !f.is_black() && !occlusion.occluded(scene) {
            illum = f * li * f32::abs(linalg::dot(&w_i, &bsdf.n)) / pdf;
        }
        if ray.depth < self.max_depth {
            illum = illum + self.specular_reflection(scene, ray, &bsdf, sampler, rng);
            illum = illum + self.specular_transmission(scene, ray, &bsdf, sampler, rng);
        }
        illum
    }
}

