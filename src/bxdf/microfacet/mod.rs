//! Module providing various microfacet distribution functions and trait that's
//! implemented by all provided distributions

use linalg::Vector;

pub use self::blinn::Blinn;
pub use self::beckmann::Beckmann;

pub mod blinn;
pub mod beckmann;

/// Trait implemented by all microfacet distributions
pub trait MicrofacetDistribution {
    /// Compute the probability that microfacets are
    /// oriented with normal  `w_h` in this distribution
    fn normal_distribution(&self, w_h: &Vector) -> f32;
    /// Sample the distribution for some outgoing light direction `w_o`.
    /// returns the incident direction and the PDF for this pair of vectors
    /// TODO: This should not return the reflected direction, it should return the sampled
    /// microfacet normal! It should also not return the PDF
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Vector, f32);
    /// Compute the PDF of sampling some pair of incoming and outgoing
    /// directions for light reflecting/refracting off the microfacets
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32;
    /// Compute the shadowing masking function for the incident and outgoing
    /// directions `w_i` and `w_o` for microfacets with normal `w_h`.
    /// Returns what fraction of the microfacets with the normal are visible
    /// in both directions.
    fn shadowing_masking(&self, w_o: &Vector, w_i: &Vector, w_h: &Vector) -> f32;
}

