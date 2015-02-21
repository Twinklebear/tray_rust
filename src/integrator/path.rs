//! Defines the Path integrator which implements path tracing with
//! explicit light sampling

use std::vec::Vec;
use std::iter;
use std::num::Float;
use rand::{Rng, StdRng};

use scene::Scene;
use linalg;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;
use sampler::{Sampler, Sample};

/// The path integrator implementing Path tracing with explicit light sampling
/// See [Kajiya, The Rendering Equation](http://dl.acm.org/citation.cfm?id=15902)
#[derive(Copy, Debug)]
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
    fn illumination(&self, scene: &Scene, r: &Ray, hit: &Intersection, sampler: &mut Sampler,
                    rng: &mut StdRng) -> Colorf {
        // TODO: We really need the memory pool now
        let mut l_samples: Vec<_> = iter::repeat((0.0, 0.0)).take(self.max_depth as usize + 1us).collect();
        let mut l_samples_comp: Vec<_> = iter::repeat(0.0).take(self.max_depth as usize + 1us).collect();
        let mut bsdf_samples: Vec<_> = iter::repeat((0.0, 0.0)).take(self.max_depth as usize + 1us).collect();
        let mut bsdf_samples_comp: Vec<_> = iter::repeat(0.0).take(self.max_depth as usize + 1us).collect();
        let mut path_samples: Vec<_> = iter::repeat((0.0, 0.0)).take(self.max_depth as usize + 1us).collect();
        let mut path_samples_comp: Vec<_> = iter::repeat(0.0).take(self.max_depth as usize + 1us).collect();
        sampler.get_samples_2d(&mut l_samples[], rng);
        sampler.get_samples_2d(&mut bsdf_samples[], rng);
        sampler.get_samples_2d(&mut path_samples[], rng);
        sampler.get_samples_1d(&mut l_samples_comp[], rng);
        sampler.get_samples_1d(&mut bsdf_samples_comp[], rng);
        sampler.get_samples_1d(&mut path_samples_comp[], rng);

        let mut illum = Colorf::black();
        let mut path_throughput = Colorf::broadcast(1.0);
        // Track if the previous bounce was a specular one
        let mut specular_bounce = false;
        let mut current_hit = *hit;
        let mut ray = *r;
        let mut bounce = 0us;
        loop {
            // TODO: Sample emissive objects on first bounce and specular bounces
            // when emissive objects are added
            /*
            if bounce == 0 || specular_bounce {
            }
            */
            let bsdf = current_hit.instance.material.bsdf(&current_hit);
            let w_o = -ray.d;
            let light_sample = Sample::new(&l_samples[bounce], l_samples_comp[bounce]);
            let bsdf_sample = Sample::new(&bsdf_samples[bounce], bsdf_samples_comp[bounce]);
            let li = self.sample_one_light(scene, &w_o, &bsdf, &light_sample, &bsdf_sample);
            illum = illum + path_throughput * li;

            // Determine the next direction to take the path by sampling the BSDF
            let path_sample = Sample::new(&path_samples[bounce], path_samples_comp[bounce]);
            let (f, w_i, pdf, sampled_type) = bsdf.sample(&w_o, BxDFType::all(), &path_sample);
            if f.is_black() || pdf == 0.0 {
                break;
            }
            specular_bounce = sampled_type.contains(&BxDFType::Specular);
            path_throughput = path_throughput * f * Float::abs(linalg::dot(&w_i, &bsdf.n)) / pdf;

            // Check if we're beyond the min depth at which point we start trying to
            // terminate rays using Russian Roulette
            if bounce > self.min_depth {
                let cont_prob = Float::min(0.5, path_throughput.luminance());
                if rng.next_f32() > cont_prob {
                    break;
                }
                // Re-weight the sum terms accordingly with the Russian roulette weight
                path_throughput = path_throughput / cont_prob;
            }
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

