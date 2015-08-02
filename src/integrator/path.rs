//! Defines the Path integrator which implements path tracing with
//! explicit light sampling

use std::f32;
use rand::StdRng;

use scene::Scene;
use linalg::{self, Ray};
use geometry::{Intersection, Emitter, Instance};
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use sampler::{Sampler, Sample};

/// The path integrator implementing Path tracing with explicit light sampling
/// See [Kajiya, The Rendering Equation](http://dl.acm.org/citation.cfm?id=15902)
#[derive(Clone, Copy, Debug)]
pub struct Path {
    min_depth: usize,
    max_depth: usize,
}

impl Path {
    /// Create a new path integrator with the min and max length desired for paths
    pub fn new(min_depth: u32, max_depth: u32) -> Path {
        Path { min_depth: min_depth as usize, max_depth: max_depth as usize }
    }
}

impl Integrator for Path {
    fn illumination(&self, scene: &Scene, light_list: &Vec<&Emitter>, r: &Ray,
                    hit: &Intersection, sampler: &mut Sampler, rng: &mut StdRng) -> Colorf {
        // TODO: We really need the memory pool now
        let num_samples = self.max_depth as usize + 1;
        let mut l_samples = vec![(0.0, 0.0); num_samples];
        let mut l_samples_comp = vec![0.0; num_samples];
        let mut bsdf_samples = vec![(0.0, 0.0); num_samples];
        let mut bsdf_samples_comp = vec![0.0; num_samples];
        let mut path_samples = vec![(0.0, 0.0); num_samples];
        let mut path_samples_comp = vec![0.0; num_samples];
        sampler.get_samples_2d(&mut l_samples[..], rng);
        sampler.get_samples_2d(&mut bsdf_samples[..], rng);
        sampler.get_samples_2d(&mut path_samples[..], rng);
        sampler.get_samples_1d(&mut l_samples_comp[..], rng);
        sampler.get_samples_1d(&mut bsdf_samples_comp[..], rng);
        sampler.get_samples_1d(&mut path_samples_comp[..], rng);

        let mut illum = Colorf::black();
        let mut path_throughput = Colorf::broadcast(1.0);
        // Track if the previous bounce was a specular one
        let mut specular_bounce = false;
        let mut current_hit = *hit;
        let mut ray = *r;
        let mut bounce = 0;
        loop {
            if bounce == 0 || specular_bounce {
                if let &Instance::Emitter(ref e) = current_hit.instance {
                    let w = -ray.d;
                    illum = illum + path_throughput * e.radiance(&w, &hit.dg.p, &hit.dg.ng);
                }
            }
            let bsdf = current_hit.material.bsdf(&current_hit);
            let w_o = -ray.d;
            let light_sample = Sample::new(&l_samples[bounce], l_samples_comp[bounce]);
            let bsdf_sample = Sample::new(&bsdf_samples[bounce], bsdf_samples_comp[bounce]);
            let li = self.sample_one_light(scene, light_list, &w_o, &current_hit.dg.p, &bsdf,
                                           &light_sample, &bsdf_sample);
            illum = illum + path_throughput * li;

            // Determine the next direction to take the path by sampling the BSDF
            let path_sample = Sample::new(&path_samples[bounce], path_samples_comp[bounce]);
            let (f, w_i, pdf, sampled_type) = bsdf.sample(&w_o, BxDFType::all(), &path_sample);
            if f.is_black() || pdf == 0.0 {
                break;
            }
            specular_bounce = sampled_type.contains(&BxDFType::Specular);
            path_throughput = path_throughput * f * f32::abs(linalg::dot(&w_i, &bsdf.n)) / pdf;

            // Check if we're beyond the min depth at which point we start trying to
            // terminate rays using Russian Roulette
            // TODO: Am I re-weighting properly? The Russian roulette results don't look quite as
            // nice, eg. damping light in transparent objects and such.
            /*
            if bounce > self.min_depth {
                let cont_prob = f32::max(0.5, path_throughput.luminance());
                if rng.next_f32() > cont_prob {
                    break;
                }
                // Re-weight the sum terms accordingly with the Russian roulette weight
                path_throughput = path_throughput / cont_prob;
            }
            */
            if bounce == self.max_depth {
                break;
            }

            ray = ray.child(&bsdf.p, &w_i.normalized());
            ray.min_t = 0.001;
            // Find the next vertex on the path
            match scene.intersect(&mut ray) {
                Some(h) => current_hit = h,
                None => break,
            }
            bounce += 1;
        }
        illum
    }
}

