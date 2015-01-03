//! Provides the Sampler trait which is implemented by the various samplers
//! to provide stratified, low-discrepancy, adaptive sampling methods and so
//! on through a simple trait interface

pub use self::uniform::Uniform;

pub mod morton;
pub mod uniform;

/// Provides the interface for all samplers to implement. Defines functions for
/// getting samples from the sampler and checking the sampler has finished sampling
/// the region
pub trait Sampler {
    /// Fill the vector with 2D pixel coordinate samples for a single pixel
    /// in the region being sampled. If the sampler doesn't have any more samples
    /// for the region the vector will be empty
    fn get_samples(&mut self, samples: &mut Vec<(f32, f32)>);
    /// Get the max number of samples this sampler will take per pixel
    fn max_spp(&self) -> uint;
    /// Check if the sampler has more samples for the region being sampled
    fn has_samples(&self) -> bool;
    /// Get the dimensions of the region being sampled in pixels
    fn dimensions(&self) -> (u32, u32);
    /// Move to a new block of the image to sample with this sampler by specifying
    /// the starting `(x, y)` block index for the new block. The block starting
    /// position will be calculated as `dimensions * start`
    fn select_block(&mut self, start: &(u32, u32));
}

