//! Defines a specular metal material
//!
//! # Scene Usage Example
//! The specular metal material requires a refractive index and absorption coefficient
//! that describe the physical properties of the metal.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "specular_silver",
//!         "type": "specular_metal",
//!         "refractive_index": [0.155265, 0.116723, 0.138381],
//!         "absorption_coefficient": [4.82835, 3.12225, 2.14696]
//!     },
//!     ...
//! ]
//! ```


use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection};
use bxdf::fresnel::{Fresnel, Conductor};
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
                                         Box::new(Conductor::new(eta, k)) as Box<Fresnel + Send + Sync>))
                                    as Box<BxDF + Send + Sync>] }
    }
}

impl Material for SpecularMetal {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}

