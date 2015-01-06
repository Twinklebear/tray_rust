//! Defines a matte material used to describe diffuse materials

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, Lambertian};
use material::Material;

/// The Matte material describes diffuse materials with either a Lambertian or
/// Oren-Nayar BRDF. The Lambertian BRDF is used for materials with no roughness
/// while Oren-Nayar is used for those with some roughness.
/// TODO: Currently we create the BSDF when creating the material but later we'd
/// like to change material properties over the surface and should use a memory pool
/// TODO: Oren-Nayar BxDF
pub struct Matte {
    bxdfs: Vec<Box<BxDF + 'static>>,
}

impl Matte {
    /// Create a new Matte material with the desired diffuse color and roughness
    pub fn new(diffuse: &Colorf, roughess: f32) -> Matte {
        Matte { bxdfs: vec![box Lambertian::new(diffuse) as Box<BxDF>], }
    }
}

impl Material for Matte {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}

