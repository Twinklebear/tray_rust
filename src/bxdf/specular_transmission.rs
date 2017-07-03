//! Defines a BTDF that describes specular transmission

use std::f32;
use enum_set::EnumSet;

use linalg::{self, Vector};
use film::Colorf;
use bxdf::{self, BxDF, BxDFType};
use bxdf::fresnel::{Fresnel, Dielectric};

/// Specular transmission BTDF that implements a specularly transmissive material model
#[derive(Clone, Copy)]
pub struct SpecularTransmission<'a> {
    /// Color of the transmissited light
    transmission: Colorf,
    /// Fresnel term for the tranmission model, only dielectrics make sense here
    fresnel: &'a Dielectric,
}

impl<'a> SpecularTransmission<'a> {
    /// Create a specularly transmissive BTDF with the color and Fresnel term
    pub fn new(c: &Colorf, fresnel: &'a Dielectric) -> SpecularTransmission<'a> {
        SpecularTransmission { transmission: *c, fresnel: fresnel }
    }
}

impl<'a> BxDF for SpecularTransmission<'a> {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Specular);
        e.insert(BxDFType::Transmission);
        e
    }
    /// We'll never exactly hit the specular transmission direction with some pair
    /// so this just returns black. Use `sample` instead
    fn eval(&self, _: &Vector, _: &Vector) -> Colorf { Colorf::broadcast(0.0) }
    /// Sampling the specular BTDF just returns the specular transmission direction
    /// for the light leaving along `w_o`
    fn sample(&self, w_o: &Vector, _: &(f32, f32)) -> (Colorf, Vector, f32) {
        // Select the incident and transmited indices of refraction based on whether
        // we're entering or exiting the material
        let entering = bxdf::cos_theta(w_o) > 0.0;
        let (ei, et, n) =
            if entering {
                (self.fresnel.eta_i, self.fresnel.eta_t, Vector::new(0.0, 0.0, 1.0))
            } else {
                (self.fresnel.eta_t, self.fresnel.eta_i, Vector::new(0.0, 0.0, -1.0))
            };
        if let Some(w_i) = linalg::refract(w_o, &n, ei / et) {
            let f = Colorf::broadcast(1.0) - self.fresnel.fresnel(bxdf::cos_theta(&w_i));
            let c = f * self.transmission / f32::abs(bxdf::cos_theta(&w_i));
            (c, w_i, 1.0)
        } else {
            (Colorf::black(), Vector::broadcast(0.0), 0.0)
        }
    }
}

