//! Defines a matte material used to describe diffuse materials
//!
//! # Scene Usage Example
//! The matte material requires a diffuse color and a roughness for the material. A roughness of 0
//! will select a [Lambertian](https://en.wikipedia.org/wiki/Lambertian_reflectance) model
//! while a roughness > 0 will select an
//! [Oren-Nayar](https://en.wikipedia.org/wiki/Oren%E2%80%93Nayar_reflectance_model)
//! reflectance model.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "purple_matte",
//!         "type": "matte",
//!         "diffuse": [1, 0, 1],
//!         "roughness" 0.5
//!     },
//!     ...
//! ]
//! ```

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, Lambertian, OrenNayar};
use material::Material;

/// The Matte material describes diffuse materials with either a Lambertian or
/// Oren-Nayar BRDF. The Lambertian BRDF is used for materials with no roughness
/// while Oren-Nayar is used for those with some roughness.
/// TODO: Currently we create the BSDF when creating the material but later we'd
/// like to change material properties over the surface and should use a memory pool
pub struct Matte {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
}

impl Matte {
    /// Create a new Matte material with the desired diffuse color and roughness
    pub fn new(diffuse: &Colorf, roughness: f32) -> Matte {
        if roughness == 0.0 {
            Matte { bxdfs: vec![Box::new(Lambertian::new(diffuse)) as Box<BxDF + Send + Sync>], }
        } else {
            Matte { bxdfs: vec![Box::new(OrenNayar::new(diffuse, roughness)) as Box<BxDF + Send + Sync>], }
        }
    }
}

impl Material for Matte {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}

