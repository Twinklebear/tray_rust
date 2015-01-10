//! Defines a BRDF that describes specular reflection

use std::num::Float;
use collect::enum_set::EnumSet;

use linalg::Vector;
use film::Colorf;
use bxdf;
use bxdf::{BxDF, BxDFType, fresnel};

/// Specular reflection BRDF that implements a specularly reflective material model
pub struct SpecularReflection {
    /// Color of the reflective material
    reflectance: Colorf,
    /// Fresnel term for the reflection model
    fresnel: Box<fresnel::Fresnel + Send + Sync>
}

impl SpecularReflection {
    /// Create a specularly reflective BRDF with the reflective color and Fresnel term
    pub fn new(c: &Colorf, fresnel: Box<fresnel::Fresnel + Send + Sync>) -> SpecularReflection {
        SpecularReflection { reflectance: *c, fresnel: fresnel }
    }
}

impl BxDF for SpecularReflection {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Specular);
        e.insert(BxDFType::Reflection);
        e
    }
    /// We'll never exactly hit the specular reflection direction with some pair
    /// so this just returns black. Use `sample` instead
    fn eval(&self, _: &Vector, _: &Vector) -> Colorf { Colorf::broadcast(0.0) }
    /// Sampling the specular BRDF just returns the specular reflection direction
    /// for the light leaving along `w_o`
    fn sample(&self, w_o: &Vector) -> (Colorf, Vector) {
        let w_i = Vector::new(-w_o.x, w_o.y, w_o.z);
        let c = self.fresnel.fresnel(bxdf::cos_theta(w_o)) * self.reflectance / Float::abs(bxdf::cos_theta(&w_i));
        (c, w_i)
    }
}

