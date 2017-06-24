//! This module provides a GGX microfacet distribution with a
//! Smith shadowing-masking term. The GGX microfacet distribution
//! is also sometimes referred to as Trowbridge-Reitz.

use std::f32;

use bxdf;
use linalg::{self, Vector};
use bxdf::microfacet::MicrofacetDistribution;

/// GGX microfacet distribution with Smith shadowing-masking. This is the
/// microfacet model described by [Walter et al.](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
#[derive(Copy, Clone)]
pub struct GGX {
    width: f32,
}

impl GGX {
    /// Create a new GGX distribution with the desired width
    pub fn new(w: f32) -> GGX {
        let roughness = f32::max(w, 0.000001);
        GGX { width: roughness }
    }
}

impl MicrofacetDistribution for GGX {
    fn normal_distribution(&self, w_h: &Vector) -> f32 {
        if bxdf::cos_theta(w_h) > 0.0 {
            let width_sqr = f32::powf(self.width, 2.0);
            let denom = f32::consts::PI * f32::powf(bxdf::cos_theta(w_h), 4.0)
                * f32::powf(width_sqr + f32::powf(bxdf::tan_theta(w_h), 2.0), 2.0);
            width_sqr / denom
        } else {
            0.0
        }
    }
    fn sample(&self, _: &Vector, samples: &(f32, f32)) -> Vector {
        let tan_theta_sqr = f32::powf(self.width * f32::sqrt(samples.0) / f32::sqrt(1.0 - samples.0), 2.0);
        let cos_theta = 1.0 / f32::sqrt(1.0 + tan_theta_sqr);
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = 2.0 * f32::consts::PI * samples.1;
        linalg::spherical_dir(sin_theta, cos_theta, phi)
    }
    fn pdf(&self, w_h: &Vector) -> f32 {
        f32::abs(bxdf::cos_theta(w_h)) * self.normal_distribution(w_h)
    }
    fn shadowing_masking(&self, w_i: &Vector, w_o: &Vector, w_h: &Vector) -> f32 {
        self.monodir_shadowing(w_i, w_h) * self.monodir_shadowing(w_o, w_h)
    }
    /// Monodirectional shadowing function from Walter et al., we use the Smith
    /// shadowing-masking which uses the reciprocity of this function.
    /// `w` is the incident/outgoing light direction and `w_h` is the microfacet normal
    fn monodir_shadowing(&self, v: &Vector, w_h: &Vector) -> f32 {
        if linalg::dot(v, w_h) / bxdf::cos_theta(v) > 0.0 {
            2.0 / (1.0 + f32::sqrt(1.0 + f32::powf(self.width * bxdf::tan_theta(v), 2.0)))
        } else {
            0.0
        }
    }
}


