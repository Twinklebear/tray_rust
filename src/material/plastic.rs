//! A material that models plastic of varying roughness using
//! the Torrance Sparrow BRDF and a Blinn microfacet distribution
//! TODO: Add Ashikman-Shirley (spelling?) anisotropic microfacet model

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow, Lambertian};
use bxdf::microfacet::{MicrofacetDistribution, Blinn};
use bxdf::fresnel::{Fresnel, Dielectric};
use material::Material;

/// The Plastic material describes plastic materials of varying roughness
pub struct Plastic {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
}

impl Plastic {
    /// Create a new plastic material specifying the diffuse and glossy colors
    /// along with the roughness of the surface
    pub fn new(diffuse: &Colorf, gloss: &Colorf, roughness: f32) -> Plastic {
        let mut bxdfs = Vec::new();
        if !diffuse.is_black() {
            bxdfs.push(Box::new(Lambertian::new(diffuse)) as Box<BxDF + Send + Sync>);
        }
        if !gloss.is_black() {
            let fresnel = Box::new(Dielectric::new(1.5, 1.0)) as Box<Fresnel + Send + Sync>;
            let microfacet = Box::new(Blinn::new(1.0 / roughness)) as Box<MicrofacetDistribution + Send + Sync>;
            bxdfs.push(Box::new(TorranceSparrow::new(gloss, fresnel, microfacet)) as Box<BxDF + Send + Sync>);
        }
        Plastic { bxdfs: bxdfs }
    }
}

impl Material for Plastic {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}

