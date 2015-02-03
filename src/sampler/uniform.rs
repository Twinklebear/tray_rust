//! Provides the simplest and worst sampling method, the Uniform sampler takes
//! a single sample at the center of each pixel in its region

use std::rand::{Rng, StdRng};
use std::rand::distributions::{Range, IndependentSample};

use sampler::{Sampler, Region};

/// Uniform sampler that takes one sample per pixel at the center of each pixel
pub struct Uniform {
    region: Region,
    float_range: Range<f32>,
}

impl Uniform {
    /// Create a uniform sampler to sample the image in `dim.0 * dim.1` sized blocks
    pub fn new(dim: (u32, u32)) -> Uniform {
        Uniform { region: Region::new((0, 0), dim), float_range: Range::new(0.0, 1.0) }
    }
}

impl Sampler for Uniform {
    fn get_samples(&mut self, samples: &mut Vec<(f32, f32)>, rng: &mut StdRng) {
        samples.clear();
        if !self.has_samples() {
            return;
        }
        samples.push((self.region.current.0 as f32 + 0.5, self.region.current.1 as f32 + 0.5));
        self.region.current.0 += 1;
        if self.region.current.0 == self.region.end.0 {
            self.region.current.0 = self.region.start.0;
            self.region.current.1 += 1;
        }
    }
    fn get_samples_2d(&mut self, samples: &mut [(f32, f32)], rng: &mut StdRng) {
        for s in samples.iter_mut() {
            s.0 = self.float_range.ind_sample(rng);
            s.1 = self.float_range.ind_sample(rng);
        }
    }
    fn get_samples_1d(&mut self, samples: &mut [f32], rng: &mut StdRng) {
        for s in samples.iter_mut() {
            *s = self.float_range.ind_sample(rng);
        }
    }
    fn max_spp(&self) -> usize { 1 }
    fn has_samples(&self) -> bool { self.region.current.1 != self.region.end.1 }
    fn dimensions(&self) -> (u32, u32) { self.region.dim }
    fn select_block(&mut self, start: (u32, u32)) {
        self.region.select_region(start);
    }
}

