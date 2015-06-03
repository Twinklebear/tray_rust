//! This module provides the Blinn Microfacet Distribution
//! TODO: Wikipedia link or something?

use std::f32;

use bxdf;
use linalg::{self, Vector};
use bxdf::microfacet::MicrofacetDistribution;

/// Struct providing the Blinn microfacet distribution
pub struct Blinn {
    exponent: f32,
}

impl Blinn {
    /// Create the Blinn microfacet distribution with some exponent
    pub fn new(e: f32) -> Blinn {
        Blinn { exponent: e }
    }
}

impl MicrofacetDistribution for Blinn {
    fn eval(&self, w_h: &Vector) -> f32 {
        (self.exponent + 2.0) * f32::consts::FRAC_2_PI
            * f32::powf(f32::abs(bxdf::cos_theta(w_h)), self.exponent)
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Vector, f32) {
        // Sample a direction on the hemisphere for the half-vector
        let cos_theta = f32::powf(samples.0, 1.0 / (self.exponent + 1.0));
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = f32::consts::PI_2 * samples.1;
        let mut w_h = linalg::spherical_dir(sin_theta, cos_theta, phi);
        if !bxdf::same_hemisphere(w_o, &w_h) {
            w_h = -w_h;
        }
        // The sampled incident direction is the outgoing direction reflected about the half-vector
        let w_i = -*w_o + 2.0 * linalg::dot(w_o, &w_h) * w_h;
        let d = linalg::dot(w_o, &w_h);
        if d <= 0.0 {
            (w_i, 0.0)
        } else {
            let pdf_val = ((self.exponent + 1.0) * f32::powf(cos_theta, self.exponent))
                        / (f32::consts::PI_2 * 4.0 * d);
            (w_i, pdf_val)
        }
    }
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32 {
        let w_h = (*w_o + *w_i).normalized();
        let cos_theta = f32::abs(bxdf::cos_theta(&w_h));
        let d = linalg::dot(w_o, &w_h);
        if d <= 0.0 {
            0.0
        } else {
            ((self.exponent + 1.0) * f32::powf(cos_theta, self.exponent))
                / (f32::consts::PI_2 * 4.0 * d)
        }
    }
}

