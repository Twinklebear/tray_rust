//! This module provides the Blinn Microfacet Distribution
//! TODO: Wikipedia link or something?
//! TODO: This is sort of a Frankenstein Blinn model, the monodir shadowing
//! and masking_shadowing terms don't really fit each other

use std::f32;

use bxdf;
use linalg::{self, Vector};
use bxdf::microfacet::MicrofacetDistribution;

/// Struct providing the normalized Blinn microfacet distribution with
/// a Cook-Torrance V-cavity shadowing-masking term
pub struct Blinn {
    exponent: f32,
}

impl Blinn {
    /// Create the Blinn microfacet distribution with some exponent
    pub fn new(e: f32) -> Blinn {
        if e > 10000.0 || f32::is_nan(e) {
            Blinn { exponent: 10000.0 }
        } else {
            Blinn { exponent: e }
        }
    }
}

impl MicrofacetDistribution for Blinn {
    fn normal_distribution(&self, w_h: &Vector) -> f32 {
        (self.exponent + 2.0) * (1.0 / (f32::consts::PI * 2.0))
            * f32::powf(f32::abs(bxdf::cos_theta(w_h)), self.exponent)
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> Vector {
        // Sample a direction on the hemisphere for the half-vector
        let cos_theta = f32::powf(samples.0, 1.0 / (self.exponent + 1.0));
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = f32::consts::PI * 2.0 * samples.1;
        let mut w_h = linalg::spherical_dir(sin_theta, cos_theta, phi);
        if !bxdf::same_hemisphere(w_o, &w_h) {
            -w_h
        } else {
            w_h
        }
    }
    fn pdf(&self, w_h: &Vector) -> f32 {
        f32::abs(bxdf::cos_theta(w_h)) * self.normal_distribution(w_h)
    }
    /// Cook-Torrance V-cavities shadowing-masking
    fn shadowing_masking(&self, w_o: &Vector, w_i: &Vector, w_h: &Vector) -> f32 {
        let n_dot_h = f32::abs(bxdf::cos_theta(w_h));
        let n_dot_o = f32::abs(bxdf::cos_theta(w_o));
        let n_dot_i = f32::abs(bxdf::cos_theta(w_i));
        let o_dot_h = f32::abs(linalg::dot(w_o, w_h));
        f32::min(1.0, f32::min(2.0 * n_dot_h * n_dot_o / o_dot_h, 2.0 * n_dot_h * n_dot_i / o_dot_h))
    }
    /// Monodirectional shadowing function from Walter et al., we use the Smith
    /// shadowing-masking which uses the reciprocity of this function.
    /// `w` is the incident/outgoing light direction and `w_h` is the microfacet normal
    fn monodir_shadowing(&self, v: &Vector, w_h: &Vector) -> f32 {
        if linalg::dot(v, w_h) / bxdf::cos_theta(v) > 0.0 {
            let a = 1.0 / (1.0 / self.exponent * bxdf::tan_theta(v));
            if a < 1.6 {
                let a_sqr = f32::powf(a, 2.0);
                (3.535 * a + 2.181 * a_sqr) / (1.0 + 2.276 * a + 2.577 * a_sqr)
            } else {
                1.0
            }
        } else {
            0.0
        }
    }
}

