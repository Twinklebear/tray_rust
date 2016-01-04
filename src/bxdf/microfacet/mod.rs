//! Module providing various microfacet distribution functions and trait that's
//! implemented by all provided distributions

use std::f32;

use linalg::{self, Vector};
use bxdf;

pub use self::blinn::Blinn;

pub mod blinn;

/// Trait implemented by all microfacet distributions
/// TODO: This is the normal distribution function of the microfacets
pub trait MicrofacetDistribution {
    /// Compute the probability density that microfacets are
    /// oriented with normal  `w_h` for this distribution
    fn eval(&self, w_h: &Vector) -> f32;
    /// Sample the distribution for some outgoing light direction `w_o`,
    /// returns the incident direction and the PDF for this pair of vectors
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Vector, f32);
    /// Compute the PDF of sampling some pair of incoming and outgoing
    /// directions for light reflecting off the distribution
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32;
}

/// Compute the geometric attenuation term for the distribution for
/// the pair of outgoing and incident light vectors for microfacets
/// with normal `w_h`
/// TODO: This is the masking shadowing function of the microfacets, this is the
/// Cook-Torrance V-cavities masking-shadowing
pub fn geometric_attenuation(w_o: &Vector, w_i: &Vector, w_h: &Vector) -> f32 {
    let n_dot_h = f32::abs(bxdf::cos_theta(w_h));
    let n_dot_o = f32::abs(bxdf::cos_theta(w_o));
    let n_dot_i = f32::abs(bxdf::cos_theta(w_i));
    let o_dot_h = f32::abs(linalg::dot(w_o, w_h));
	f32::min(1.0, f32::min(2.0 * n_dot_h * n_dot_o / o_dot_h, 2.0 * n_dot_h * n_dot_i / o_dot_h))
}

