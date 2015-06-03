//! This module provides the Torrance Sparrow microfacet BRDF
//! TODO: Wikipedia link?

use std::f32;
use enum_set::EnumSet;

use linalg::{self, Vector};
use film::Colorf;
use bxdf::{self, BxDF, BxDFType};
use bxdf::fresnel::Fresnel;
use bxdf::microfacet::{self, MicrofacetDistribution};

/// Struct providing the Torrance Sparrow BRDF
pub struct TorranceSparrow {
    reflectance: Colorf,
    fresnel: Box<Fresnel + Send + Sync>,
    /// Microfacet distribution describing the structure of the microfacets of
    /// the material
    microfacet: Box<MicrofacetDistribution + Send + Sync>,
}

impl TorranceSparrow {
    /// Create a new Torrance Sparrow microfacet BRDF
    pub fn new(c: &Colorf, fresnel: Box<Fresnel + Send + Sync>,
               microfacet: Box<MicrofacetDistribution + Send + Sync>) -> TorranceSparrow {
        TorranceSparrow { reflectance: *c, fresnel: fresnel, microfacet: microfacet }
    }
}

impl BxDF for TorranceSparrow {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Reflection);
        e
    }
    fn eval(&self, w_o: &Vector, w_i: &Vector) -> Colorf {
        let cos_to = f32::abs(bxdf::cos_theta(w_o));
        let cos_ti = f32::abs(bxdf::cos_theta(w_i));
        if cos_to == 0.0 || cos_ti == 0.0 {
            return Colorf::black()
        }
        let w_h = *w_i + *w_o;
        if w_h.length_sqr() == 0.0 {
            return Colorf::black();
        }
        let w_h = w_h.normalized();
        let cos_th = linalg::dot(w_i, &w_h);
        self.reflectance * self.microfacet.eval(&w_h) * microfacet::geometric_attenuation(w_o, w_i, &w_h)
            * self.fresnel.fresnel(cos_th) / (4.0 * cos_ti * cos_to)
    }
    fn sample(&self, w_o: &Vector, samples: &(f32, f32)) -> (Colorf, Vector, f32) {
        let (w_i, pdf) = self.microfacet.sample(w_o, samples);
        if !bxdf::same_hemisphere(w_o, &w_i) {
            (Colorf::black(), Vector::broadcast(0.0), 0.0)
        } else {
            (self.eval(w_o, &w_i), w_i, pdf)
        }
    }
    fn pdf(&self, w_o: &Vector, w_i: &Vector) -> f32 {
        if !bxdf::same_hemisphere(w_o, w_i) {
            0.0
        } else {
            self.microfacet.pdf(w_o, w_i)
        }
    }
}

