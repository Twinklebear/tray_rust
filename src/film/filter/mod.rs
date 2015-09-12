//! The filter module provides reconstruction filters to be used
//! when writing samples to the render target. The filter width and
//! height refer to how many pixels the filter covers, where a single
//! pixel is 0.5x0.5

pub use self::gaussian::Gaussian;
pub use self::mitchell_netravali::MitchellNetravali;

pub mod gaussian;
pub mod mitchell_netravali;

/// Trait implemented by all reconstructon filters. Provides methods for getting
/// the width/height and computing the weight at some point relative to the filter
/// center.
pub trait Filter {
    /// Compute the weight of this filter at some point (x, y) relative
    /// to the center of the filter
    fn weight(&self, x: f32, y: f32) -> f32;
    /// Return the width of the filter
    fn width(&self) -> f32;
    /// Return the inverse width of the filter
    fn inv_width(&self) -> f32;
    /// Return the height of the filter
    fn height(&self) -> f32;
    /// Return the inverse height of the filter
    fn inv_height(&self) -> f32;
}

