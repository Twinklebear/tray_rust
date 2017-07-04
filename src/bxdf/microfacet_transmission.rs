//! This module provides a microfacet model for transmission (a BTDF), see
//! [Walter et al. 07](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
//! for details.

use std::f32;
use enum_set::EnumSet;

use linalg::{self, Vector};
use film::Colorf;
use bxdf::{self, BxDF, BxDFType};
use bxdf::fresnel::{Dielectric, Fresnel};
use bxdf::microfacet::{MicrofacetDistribution};

/// Struct providing the microfacet BTDF, implemented as described in
/// [Walter et al. 07](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
#[derive(Copy, Clone)]
pub struct MicrofacetTransmission<'a> {
    reflectance: Colorf,
    fresnel: &'a Dielectric,
    /// Microfacet distribution describing the structure of the microfacets of
    /// the material
    microfacet: &'a MicrofacetDistribution,
}

impl<'a> MicrofacetTransmission<'a> {
    /// Create a new transmissive microfacet BRDF
    pub fn new(c: &Colorf, fresnel: &'a Dielectric, microfacet: &'a MicrofacetDistribution)
            -> MicrofacetTransmission<'a> {
        MicrofacetTransmission { reflectance: *c, fresnel: fresnel, microfacet: microfacet }
    }
    /// Convenience method for getting `eta_i` and `eta_t` in the right order for if
    /// we're entering or exiting this material based on the direction of the outgoing
    /// ray.
    fn eta_for_interaction(&self, w_o: &Vector) -> (f32, f32) {
        if bxdf::cos_theta(w_o) > 0.0 {
            (self.fresnel.eta_i, self.fresnel.eta_t)
        } else {
            (self.fresnel.eta_t, self.fresnel.eta_i)
        }
    }
    /// Compute the Jacobian for the change of variables (see [Walter et al 07] section 4.2),
    /// here we compute equation 17 in that section.
    fn jacobian(w_o: &Vector, w_i: &Vector, w_h: &Vector, eta: (f32, f32)) -> f32 {
        let wi_dot_h = linalg::dot(w_i, w_h);
        let wo_dot_h = linalg::dot(w_o, w_h);
        let denom = f32::powf(eta.1 * wi_dot_h + eta.0 * wo_dot_h, 2.0);
        if denom != 0.0 {
            f32::abs(f32::powf(eta.0, 2.0) * f32::abs(wo_dot_h) / denom)
        } else {
            0.0
        }
    }
    fn half_vector(w_o: &Vector, w_i: &Vector, eta: (f32, f32)) -> Vector {
        (-eta.1 * *w_i - eta.0 * *w_o).normalized()
    }
}

impl<'a> BxDF for MicrofacetTransmission<'a> {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Transmission);
        e
    }
    fn eval(&self, w_o: &Vector, w_i: &Vector) -> Colorf {
        if bxdf::same_hemisphere(w_o, w_i) {
            return Colorf::black();
        }
        let cos_to = bxdf::cos_theta(w_o);
        let cos_ti = bxdf::cos_theta(w_i);
        if cos_to == 0.0 || cos_ti == 0.0 {
            return Colorf::black();
        }
        let eta = self.eta_for_interaction(w_o);
        let w_h = MicrofacetTransmission::half_vector(w_o, w_i, eta);
        let d = self.microfacet.normal_distribution(&w_h);
        let f = Colorf::broadcast(1.0) - self.fresnel.fresnel(linalg::dot(w_o, &w_h));
        let g = self.microfacet.shadowing_masking(w_i, w_o, &w_h);
        let wi_dot_h = linalg::dot(w_i, &w_h);
        let jacobian = MicrofacetTransmission::jacobian(w_o, w_i, &w_h, eta);
        self.reflectance * (f32::abs(wi_dot_h) / (f32::abs(w_i.z) * f32::abs(w_o.z)))
            * (f * g * d) * jacobian
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Colorf, Vector, f32) {
        let mut w_h = self.microfacet.sample(w_o, samples);
        if !bxdf::same_hemisphere(w_o, &w_h) {
            w_h = -w_h;
        }
        let eta = self.eta_for_interaction(w_o);
        if let Some(w_i) = linalg::refract(w_o, &w_h, eta.0 / eta.1) {
            if bxdf::same_hemisphere(w_o, &w_i) {
                (Colorf::black(), Vector::broadcast(0.0), 0.0)
            } else {
                (self.eval(w_o, &w_i), w_i, self.pdf(w_o, &w_i))
            }
        } else {
            (Colorf::black(), Vector::broadcast(0.0), 0.0)
        }
    }
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32 {
        if bxdf::same_hemisphere(w_o, w_i) {
            0.0
        } else {
            let eta = self.eta_for_interaction(w_o);
            let w_h = MicrofacetTransmission::half_vector(w_o, w_i, eta);
            self.microfacet.pdf(&w_h) * MicrofacetTransmission::jacobian(w_o, w_i, &w_h, eta)
        }
    }
}


