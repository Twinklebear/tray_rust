//! Provides an adaptive sampler which will start sampling at
//! one rate and then take more samples if it determines more
//! are necessary for the pixel. The samples generated are the
//! same as those from the Low Discrepancy sampler but the
//! number of samples taken per pixel will vary.

use std::u32;
use std::f32;
use rand::{Rng, StdRng};
use rand::distributions::{IndependentSample, Range};

use sampler::{Sampler, Region, ld};
use film::ImageSample;

/// Adaptive sampler that makes use of the (0, 2) sequence to generate
/// well distributed samples and takes `min_spp` to `max_spp` samples per pixel
pub struct Adaptive {
    region: Region,
    /// Number of samples to take per pixel
    min_spp: usize,
    max_spp: usize,
    /// How many samples we've taken for this pixel so far
    samples_taken: usize,
    scramble_range: Range<u32>,
}

impl Adaptive {
    /// Create a low discrepancy sampler to sample the image in `dim.0 * dim.1` sized blocks
    pub fn new(dim: (u32, u32), mut min_spp: usize, mut max_spp: usize) -> Adaptive {
        if !min_spp.is_power_of_two() {
            min_spp = min_spp.next_power_of_two();
            print!("Warning: Adaptive sampler requires power of two samples per pixel, ");
            println!("rounding min_spp up to {}", min_spp);
        }
        if !max_spp.is_power_of_two() {
            max_spp = max_spp.next_power_of_two();
            print!("Warning: Adaptive sampler requires power of two samples per pixel, ");
            println!("rounding max_spp up to {}", max_spp);
        }
        Adaptive { region: Region::new((0, 0), dim), min_spp: min_spp, max_spp: max_spp,
                   samples_taken: 0, scramble_range: Range::new(0, u32::MAX) }
    }
    /// Determine if more samples need to be taken for the pixel currently sampled with the
    /// set of samples passed. This is done by simply looking at the contrast difference
    /// between the samples. TODO: What are some better strategies for estimating
    /// if we need more samples?
    fn needs_supersampling(&self, samples: &[ImageSample]) -> bool {
        let max_contrast = 0.5;
        let avg_lum = samples.iter().fold(0.0, |ac, s| ac + s.color.luminance()) / samples.len() as f32;
        for s in samples.iter() {
            if f32::abs(s.color.luminance() - avg_lum) / avg_lum > max_contrast {
                return true;
            }
        }
        return false;
    }
}

impl Sampler for Adaptive {
    fn get_samples(&mut self, samples: &mut Vec<(f32, f32)>, rng: &mut StdRng) {
        samples.clear();
        if !self.has_samples() {
            return;
        }

        self.samples_taken += self.min_spp;
        if samples.len() < self.samples_taken {
            samples.resize(self.samples_taken, (0.0, 0.0));
        }
        self.get_samples_2d(&mut samples[..], rng);
        for s in samples.iter_mut() {
            s.0 += self.region.current.0 as f32;
            s.1 += self.region.current.1 as f32;
        }
    }
    fn get_samples_2d(&mut self, samples: &mut [(f32, f32)], rng: &mut StdRng) {
        let scramble = (self.scramble_range.ind_sample(rng),
                        self.scramble_range.ind_sample(rng));
        ld::sample_2d(samples, scramble);
        rng.shuffle(samples);
    }
    fn get_samples_1d(&mut self, samples: &mut [f32], rng: &mut StdRng) {
        let scramble = self.scramble_range.ind_sample(rng);
        ld::sample_1d(samples, scramble);
        rng.shuffle(samples);
    }
    fn max_spp(&self) -> usize { self.max_spp }
    fn has_samples(&self) -> bool { self.region.current.1 != self.region.end.1 }
    fn dimensions(&self) -> (u32, u32) { self.region.dim }
    fn select_block(&mut self, start: (u32, u32)) {
        self.region.select_region(start);
    }
    fn get_region(&self) -> &Region {
        &self.region
    }
    fn report_results(&mut self, samples: &[ImageSample]) -> bool {
        // If we've hit taken the max samples per pixel or don't need to super sample
        // this pixel advance to the next one
        if self.samples_taken >= self.max_spp || !self.needs_supersampling(samples) {
            self.samples_taken = 0;
            self.region.current.0 += 1;
            if self.region.current.0 == self.region.end.0 {
                self.region.current.0 = self.region.start.0;
                self.region.current.1 += 1;
            }
            true
        } else {
            false
        }
    }
}

