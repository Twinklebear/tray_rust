//! Defines the Whitted integrator which implements Whitted recursive ray tracing
//! See [Whitted, An improved illumination model for shaded display](http://dl.acm.org/citation.cfm?id=358882)
//!
//! # Scene Usage Example
//! The Whitted integrator just needs a maximum ray depth to terminate specular reflection
//! and transmission rays.
//!
//! ```json
//! "integrator": {
//!     "type": "whitted",
//!     "max_depth": 8
//! }
//! ```

use std::f32;
use rand::StdRng;

use scene::Scene;
use linalg::{self, Ray};
use geometry::{Intersection, Emitter, Instance};
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use light::Light;
use sampler::Sampler;

/// The Whitted integrator implementing the Whitted recursive ray tracing algorithm
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
    fn illumination(&self, scene: &Scene, light_list: &Vec<&Emitter>, ray: &Ray,
                    hit: &Intersection, sampler: &mut Sampler, rng: &mut StdRng) -> Colorf {
        let bsdf = hit.material.bsdf(hit);
        let w_o = -ray.d;
        let mut sample_2d = [(0.0, 0.0)];
        sampler.get_samples_2d(&mut sample_2d[..], rng);
        let mut illum = Colorf::broadcast(0.0);
        if ray.depth == 0 {
            if let &Instance::Emitter(ref e) = hit.instance {
                let w = -ray.d;
                illum = illum + e.radiance(&w, &hit.dg.p, &hit.dg.ng);
            }
        }

        for light in light_list {
            let (li, w_i, pdf, occlusion) = light.sample_incident(&hit.dg.p, &sample_2d[0], ray.time);
            let f = bsdf.eval(&w_o, &w_i, BxDFType::all());
            if !li.is_black() && !f.is_black() && !occlusion.occluded(scene, ray.time) {
                illum = illum + f * li * f32::abs(linalg::dot(&w_i, &bsdf.n)) / pdf;
            }
        }
        if ray.depth < self.max_depth {
            illum = illum + self.specular_reflection(scene, light_list, ray, &bsdf, sampler, rng);
            illum = illum + self.specular_transmission(scene, light_list, ray, &bsdf, sampler, rng);
        }
        illum
    }
}

