//! The filter module provides reconstruction filters to be used
//! when writing samples to the render target

pub use self::gaussian::Gaussian;

pub mod gaussian;

pub trait Filter {
    /// Compute the weight of this filter at some point (x, y) relative
    /// to the center of the filter
    fn weight(&self, x: f32, y: f32) -> f32;
}

