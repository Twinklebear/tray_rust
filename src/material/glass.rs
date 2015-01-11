//! Defines a specular glass material

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection, SpecularTransmission};
use bxdf::fresnel::{Dielectric, Fresnel};
use material::Material;

/// The Glass material describes specularly transmissive and reflective glass material
pub struct Glass {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
    eta: f32,
}

impl Glass {
    /// Create the glass material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    pub fn new(reflect: &Colorf, transmit: &Colorf, eta: f32) -> Glass {
        let mut bxdfs = Vec::new();
        if !reflect.is_black() {
            bxdfs.push(Box::new(SpecularReflection::new(reflect,
                            Box::new(Dielectric::new(1.0, eta)) as Box<Fresnel + Send + Sync>))
                      as Box<BxDF + Send + Sync>);
        }
        if !transmit.is_black() {
            bxdfs.push(Box::new(SpecularTransmission::new(transmit, Dielectric::new(1.0, eta)))
                      as Box<BxDF + Send + Sync>);
        }
        Glass { bxdfs: bxdfs, eta: eta }
    }
}

impl Material for Glass {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, self.eta, &hit.dg)
    }
}


