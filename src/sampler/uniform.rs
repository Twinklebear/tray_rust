///! Provides the simplest and worst sampling method, the Uniform sampler takes
///! a single sample at the center of each pixel in its region

use sampler::Sampler;

/// Uniform sampler that takes one sample per pixel at the center of each pixel
#[deriving(Copy, Show)]
pub struct Uniform {
    /// Current coordinates of the pixel to sample (x, y)
    current: (u32, u32),
    /// Coordinates of the start of region being sampled (x, y)
    start: (u32, u32),
    /// Coordinates of the end of the region being sampled (x, y)
    end: (u32, u32),
    /// Dimensions of the region being sampled
    dimensions: (u32, u32),
}

impl Uniform {
    /// Create a uniform sampler to sample the image in `dimension.0 * dimension.1`
    /// sized blocks, selected in Morton order via `select_block`
    pub fn new(dimensions: (u32, u32)) -> Uniform {
        Uniform { current: (0, 0), start: (0, 0),
                  end: dimensions, dimensions: dimensions }
    }
}

impl Sampler for Uniform {
    fn get_samples(&mut self, samples: &mut Vec<(f32, f32)>) {
        samples.clear();
        if !self.has_samples() {
            return;
        }
        samples.push((self.current.0 as f32 + 0.5, self.current.1 as f32 + 0.5));
        self.current.0 += 1;
        if self.current.0 == self.end.0 {
            self.current.0 = self.start.0;
            self.current.1 += 1;
        }
    }
    fn max_spp(&self) -> uint { 1 }
    fn has_samples(&self) -> bool { self.current.1 != self.end.1 }
    fn dimensions(&self) -> (u32, u32) { self.dimensions }
    fn select_block(&mut self, start: &(u32, u32)) {
        self.start.0 = start.0 * self.dimensions.0;
        self.start.1 = start.1 * self.dimensions.1;
        self.end.0 = self.start.0 + self.dimensions.0;
        self.end.1 = self.start.1 + self.dimensions.1;
        self.current.0 = self.start.0;
        self.current.1 = self.start.1;
    }
}

