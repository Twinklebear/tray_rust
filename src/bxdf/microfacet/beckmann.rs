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
        Beckmann { width: w }
    }
    /// Monodirectional shadowing function from Walter et al., we use the Smith
    /// shadowing-masking which uses the reciprocity of this function.
    /// `w` is the incident/outgoing light direction and `w_h` is the microfacet normal
    fn monodir_shadowing(&self, w: &Vector, w_h: &Vector) -> f32 {
        if linalg::dot(w, w_h) / w.z > 0.0 {
            let a = 1.0 / (self.width * bxdf::tan_theta(w));
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


// In walter et al. chi^+(a) = 1 if a > 0 and 0 if a <= 0
// bold n is the macrosurface normal. In shading space this is +z
// theta is angle between the dir and the macrosurface normal (a.k.a +z)
// phi is angle between the dir and a vector perpindicular to the macrosurface normal, like tangent
// or bitangent

impl MicrofacetDistribution for Beckmann {
    fn normal_distribution(&self, w_h: &Vector) -> f32 {
        if w_h.z > 0.0 {
            let e = f32::exp(-f32::powf(bxdf::tan_theta(w_h) / self.width, 2.0));
            e / (f32::consts::PI * f32::powf(self.width, 2.0) * f32::powf(bxdf::cos_theta(w_h), 4.0))
        } else {
            0.0
        }
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Vector, f32) {
        let theta = f32::atan(f32::sqrt(-f32::powf(self.width, 2.0) * f32::ln(1.0 - samples.0)));
        let phi = 2.0 * f32::consts::PI * samples.1;
        let w_h = linalg::spherical_dir(f32::sin(theta), f32::cos(theta), phi);
        // The sampled incident direction is the outgoing direction perfectly reflected about the half-vector
        let w_i = 2.0 * linalg::dot(w_o, &w_h) * w_h - *w_o;
        let w = f32::abs(linalg::dot(w_o, &w_h)) * self.shadowing_masking(w_o, &w_i, &w_h) / (w_o.z * w_h.z);
        (w_i, w)
    }
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32 {
        let w_h = (*w_o + *w_i).normalized();
        f32::abs(linalg::dot(w_o, &w_h)) * self.shadowing_masking(w_o, &w_i, &w_h) / (w_o.z * w_h.z)
    }
    fn shadowing_masking(&self, w_o: &Vector, w_i: &Vector, w_h: &Vector) -> f32 {
        self.monodir_shadowing(w_o, w_h) * self.monodir_shadowing(w_i, w_h)
    }
}

