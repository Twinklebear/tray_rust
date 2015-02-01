//! Provides the simplest and worst sampling method, the Uniform sampler takes
//! a single sample at the center of each pixel in its region

use std::rand::Rng;

use sampler::{Sampler, Region};

/// Uniform sampler that takes one sample per pixel at the center of each pixel
#[derive(Copy, Debug)]
pub struct Uniform {
    region: Region,
}

impl Uniform {
    /// Create a uniform sampler to sample the image in `dim.0 * dim.1` sized blocks
    pub fn new(dim: (u32, u32)) -> Uniform {
        Uniform { region: Region::new((0, 0), dim) }
    }
}

impl Sampler for Uniform {
    fn get_samples<R: Rng>(&mut self, samples: &mut Vec<(f32, f32)>, rng: &mut R) {
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
    fn max_spp(&self) -> usize { 1 }
    fn has_samples(&self) -> bool { self.region.current.1 != self.region.end.1 }
    fn dimensions(&self) -> (u32, u32) { self.region.dim }
    fn select_block(&mut self, start: (u32, u32)) {
        self.region.select_region(start);
    }
}

