//! Defines a specular metal material

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection, fresnel};
use material::Material;

/// The Specular Metal material describes specularly reflective metals using their
/// refractive index and absorption coefficient
pub struct SpecularMetal {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
}

impl SpecularMetal {
    /// Create a new specular metal with the desired metal properties.
    /// `eta`: refractive index of the metal
    /// `k`: absorption coefficient of the metal
    pub fn new(eta: &Colorf, k: &Colorf) -> SpecularMetal {
        SpecularMetal { bxdfs: vec![Box::new(SpecularReflection::new(&Colorf::broadcast(1.0),
                                         Box::new(fresnel::Conductor::new(eta, k)) as Box<fresnel::Fresnel + Send + Sync>))
                                    as Box<BxDF + Send + Sync>] }
    }
}

impl Material for SpecularMetal {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}

