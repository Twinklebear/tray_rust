//! This module provides a Beckmann microfacet distribution with a
//! Smith shadowing-masking term

use std::f32;

use bxdf;
use linalg::{self, Vector};
use bxdf::microfacet::MicrofacetDistribution;

/// Beckmann microfacet distribution with Smith shadowing-masking. This is the
/// microfacet model described by [Walter et al.](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
pub struct Beckmann {
    width: f32,
}

impl Beckmann {
    /// Create a new Beckmann distribution with the desired width
    pub fn new(w: f32) -> Beckmann {
        let roughness = f32::max(w, 0.000001);
        Beckmann { width: roughness }
    }
}

impl MicrofacetDistribution for Beckmann {
    fn normal_distribution(&self, w_h: &Vector) -> f32 {
        if bxdf::cos_theta(w_h) > 0.0 {
            let e = f32::exp(-f32::powf(bxdf::tan_theta(w_h) / self.width, 2.0));
            e / (f32::consts::PI * f32::powf(self.width, 2.0) * f32::powf(bxdf::cos_theta(w_h), 4.0))
        } else {
            0.0
        }
    }
    fn sample(&self, _: &Vector, samples: &(f32, f32)) -> Vector {
        let log_sample = match f32::ln(1.0 - samples.0) {
            x if f32::is_infinite(x) => 0.0,
            x => x,
        };
        let tan_theta_sqr = -f32::powf(self.width, 2.0) * log_sample;
        let phi = 2.0 * f32::consts::PI * samples.1;
        let cos_theta = 1.0 / f32::sqrt(1.0 + tan_theta_sqr);
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
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
            let a = 1.0 / (self.width * bxdf::tan_theta(v));
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

