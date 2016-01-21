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
    /// oriented with normal `w_h` in this distribution
    fn normal_distribution(&self, w_h: &Vector) -> f32;
    /// Sample the distribution for some outgoing light direction `w_o`.
    /// returns the sampled microfacet normal
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> Vector;
    /// Compute the probability of sampling a certain microfacet normal
    /// from the distribution, `w_h`
    fn pdf(&self, w_h: &Vector) -> f32;
    /// Compute the shadowing masking function for the incident and outgoing
    /// directions `w_i` and `w_o` for microfacets with normal `w_h`.
    /// Returns what fraction of the microfacets with the normal are visible
    /// in both directions.
    fn shadowing_masking(&self, w_i: &Vector, w_o: &Vector, w_h: &Vector) -> f32;
    /// Return the monodirectional shadowing function, G_1
    /// `v` is the reflected/incident direction, `w_h` is the microfacet normal
    fn monodir_shadowing(&self, v: &Vector, w_h: &Vector) -> f32;
}

