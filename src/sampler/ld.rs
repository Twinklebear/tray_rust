//! Provides a high quality sampling scheme based on (0, 2)-sequences
//! See sec. 7.4.3 of Physically Based Rendering

use std::num::UnsignedInt;
use std::rand;
use std::rand::Rng;
use std::rand::distributions::{IndependentSample, Range};
use std::u32;
use std::f32;
use std::num::Float;

use sampler::{Sampler, Region};

/// Low discrepancy sampler that makes use of the (0, 2) sequence to generate
/// well distributed samples
pub struct LowDiscrepancy {
    region: Region,
    /// Number of samples to take per pixel
    spp: usize,
    scramble_range: Range<u32>,
}

impl LowDiscrepancy {
    /// Create a low discrepancy sampler to sample the image in `dim.0 * dim.1` sized blocks
    pub fn new(dim: (u32, u32), mut spp: usize) -> LowDiscrepancy {
        if !spp.is_power_of_two() {
            spp = spp.next_power_of_two();
            print!("Warning: LowDiscrepancy sampler requires power of two samples per pixel, ");
            println!("rounding up to {}", spp);
        }
        LowDiscrepancy { region: Region::new((0, 0), dim), spp: spp,
                         scramble_range: Range::new(0, u32::MAX) }
    }
}

impl Sampler for LowDiscrepancy {
    fn get_samples<R: Rng>(&mut self, samples: &mut Vec<(f32, f32)>, rng: &mut R) {
        samples.clear();
        if !self.has_samples() {
            return;
        }
        if samples.len() < self.spp {
            samples.resize(self.spp, (0.0, 0.0));
        }
        self.get_samples_2d(&mut samples[], rng);
        for s in samples.iter_mut() {
            s.0 += self.region.current.0 as f32;
            s.1 += self.region.current.1 as f32;
        }

        self.region.current.0 += 1;
        if self.region.current.0 == self.region.end.0 {
            self.region.current.0 = self.region.start.0;
            self.region.current.1 += 1;
        }
    }
    fn get_samples_2d<R: Rng>(&mut self, samples: &mut [(f32, f32)], rng: &mut R){
        let scramble = (self.scramble_range.ind_sample(rng),
                        self.scramble_range.ind_sample(rng));
        sample_2d(samples, scramble);
        rng.shuffle(samples);
    }
    fn get_samples_1d<R: Rng>(&mut self, samples: &mut [f32], rng: &mut R){
        let scramble = self.scramble_range.ind_sample(rng);
        sample_1d(samples, scramble);
        rng.shuffle(samples);
    }
    fn max_spp(&self) -> usize { self.spp }
    fn has_samples(&self) -> bool { self.region.current.1 != self.region.end.1 }
    fn dimensions(&self) -> (u32, u32) { self.region.dim }
    fn select_block(&mut self, start: (u32, u32)) {
        self.region.select_region(start);
    }
}

/// Generate a 2D pattern of low discrepancy samples to fill the slice
/// sample values will be normalized between [0, 1]
pub fn sample_2d(samples: &mut [(f32, f32)], scramble: (u32, u32)) {
    for s in samples.iter_mut().enumerate() {
        *s.1 = sample_02(s.0 as u32, scramble);
    }
}
/// Generate a 1D pattern of low discrepancy samples to fill the slice
/// sample values will be normalized between [0, 1]
pub fn sample_1d(samples: &mut [f32], scramble: u32) {
    for s in samples.iter_mut().enumerate() {
        *s.1 = van_der_corput(s.0 as u32, scramble);
    }
}
/// Generate a sample from a scrambled (0, 2) sequence
pub fn sample_02(n: u32, scramble: (u32, u32)) -> (f32, f32) {
    (van_der_corput(n, scramble.0), sobol(n, scramble.1))
}
/// Generate a scrambled Van der Corput sequence value
/// as described by Kollig & Keller (2002) and in PBR
/// method is specialized for base 2
pub fn van_der_corput(mut n: u32, scramble: u32) -> f32 {
	n = (n << 16) | (n >> 16);
	n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
	n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
	n = ((n & 0x33333333) << 2) | ((n & 0xcccccccc) >> 2);
	n = ((n & 0x55555555) << 1) | ((n & 0xaaaaaaaa) >> 1);
	n ^= scramble;
	Float::min(((n >> 8) & 0xffffff) as f32 / ((1 << 24) as f32), 1.0 - f32::EPSILON)
}
/// Generate a scrambled Sobol' sequence value
/// as described by Kollig & Keller (2002) and in PBR
/// method is specialized for base 2
pub fn sobol(mut n: u32, mut scramble: u32) -> f32 {
    let mut i = 1 << 31;
    while n != 0 {
        if n & 0x1 != 0 {
            scramble ^= i;
        }
        n >>= 1;
        i ^= i >> 1;
    }
    Float::min(((scramble >> 8) & 0xffffff) as f32 / ((1 << 24) as f32), 1.0 - f32::EPSILON)
}

