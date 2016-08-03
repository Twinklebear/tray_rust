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
pub struct MicrofacetTransmission {
    reflectance: Colorf,
    fresnel: Dielectric,
    /// Microfacet distribution describing the structure of the microfacets of
    /// the material
    microfacet: Box<MicrofacetDistribution + Send + Sync>,
}

impl MicrofacetTransmission {
    /// Create a new transmissive microfacet BRDF
    pub fn new(c: &Colorf, fresnel: Dielectric,
               microfacet: Box<MicrofacetDistribution + Send + Sync>) -> MicrofacetTransmission {
        MicrofacetTransmission { reflectance: *c, fresnel: fresnel, microfacet: microfacet }
    }
    /// Convenience method for getting `eta_i` and `eta_t` in the right order for if
    /// we're entering or exiting this material based on the direction of the outgoing
    /// ray.
    fn eta_for_interaction(&self, w_o: &Vector) -> (f32, f32) {
        if w_o.z > 0.0 {
            (self.fresnel.eta_i, self.fresnel.eta_t)
        } else {
            (self.fresnel.eta_t, self.fresnel.eta_i)
        }
    }
}

impl BxDF for MicrofacetTransmission {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Transmission);
        e
    }
    fn eval(&self, w_o: &Vector, w_i: &Vector) -> Colorf {
        let cos_to = f32::abs(bxdf::cos_theta(w_o));
        let cos_ti = f32::abs(bxdf::cos_theta(w_i));
        if cos_to == 0.0 || cos_ti == 0.0 {
            return Colorf::new(0.0, 0.0, 0.0)
        }
        let mut w_h = *w_i + *w_o;
        if w_h.x == 0.0 && w_h.y == 0.0 && w_h.z == 0.0 {
            return Colorf::new(0.0, 0.0, 0.0)
        }
        w_h = w_h.normalized();
        let eta = self.eta_for_interaction(w_o);
        let d = self.microfacet.normal_distribution(&w_h);
        let f = Colorf::broadcast(1.0) - self.fresnel.fresnel(linalg::dot(w_i, &w_h));
        let g = self.microfacet.shadowing_masking(w_i, w_o, &w_h);
        let wi_dot_h = linalg::dot(w_i, &w_h);
        let wo_dot_h = linalg::dot(w_o, &w_h);
        self.reflectance
            * (f32::abs(wi_dot_h) * f32::abs(wo_dot_h)) / (f32::abs(w_i.z) * f32::abs(w_o.z))
            * (f32::powf(eta.1, 2.0) * f * g * d) / f32::powf(eta.0 * wi_dot_h + eta.1 * wo_dot_h, 2.0)
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Colorf, Vector, f32) {
        let w_h = match self.microfacet.sample(w_o, samples) {
            h if !bxdf::same_hemisphere(w_o, &h) => -h,
            h => h
        };
        let eta = self.eta_for_interaction(w_o);
        let w_i = match linalg::refract(w_o, &w_h, eta.0, eta.1) {
            Some(v) => v,
            None => return (Colorf::black(), Vector::broadcast(0.0), 0.0),
        };
        if bxdf::same_hemisphere(w_o, &w_i) {
            (Colorf::black(), Vector::broadcast(0.0), 0.0)
        } else {
            // This term is p_o(o) in eq. 38 of Walter et al's 07 paper and is for reflection so
            // we use the Jacobian for reflection, eq. 17
            let wi_dot_h = linalg::dot(&w_i, &w_h);
            let wo_dot_h = linalg::dot(w_o, &w_h);
            let jacobian = eta.1 * f32::abs(linalg::dot(w_o, &w_h))
                / f32::powf(eta.1 * wi_dot_h + eta.0 * wo_dot_h, 2.0);
            let pdf = self.microfacet.pdf(&w_h) * jacobian;
            (self.eval(w_o, &w_i), w_i, pdf)
        }
    }
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32 {
        if bxdf::same_hemisphere(w_o, w_i) {
            0.0
        } else {
            let w_h = *w_o + *w_i;
            if w_h.x == 0.0 && w_h.y == 0.0 && w_h.z == 0.0 {
                0.0
            } else {
                let eta = self.eta_for_interaction(w_o);
                let wi_dot_h = linalg::dot(w_i, &w_h);
                let wo_dot_h = linalg::dot(w_o, &w_h);
                // This term is p_o(o) in eq. 38 of Walter et al's 07 paper and is for reflection so
                // we use the Jacobian for reflection, eq. 17
                let jacobian = eta.1 * f32::abs(linalg::dot(w_o, &w_h))
                    / f32::powf(eta.1 * wi_dot_h + eta.0 * wo_dot_h, 2.0);
                self.microfacet.pdf(&w_h.normalized()) * jacobian
            }
        }
    }
}


