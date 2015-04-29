//! Defines a Lambertion BRDF that describes perfectly diffuse surfaces.
//! See [Lambertian reflectance](https://en.wikipedia.org/wiki/Lambertian_reflectance)

use std::f32;
use enum_set::EnumSet;

use linalg::Vector;
use film::Colorf;
use bxdf::{BxDF, BxDFType};

/// Lambertian BRDF that implements the Lambertian reflectance model
#[derive(Clone, Copy, Debug)]
pub struct Lambertian {
    /// Color of the diffuse material
    reflectance: Colorf,
}

impl Lambertian {
    /// Create a new Lambertian BRDF with the desired reflective color property
    pub fn new(c: &Colorf) -> Lambertian {
        Lambertian { reflectance: *c }
    }
}

impl BxDF for Lambertian {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Diffuse);
        e.insert(BxDFType::Reflection);
        e
    }
    fn eval(&self, _: &Vector, _: &Vector) -> Colorf {
        self.reflectance * f32::consts::FRAC_1_PI
    }
}

