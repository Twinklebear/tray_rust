//! Provides a material for modelling metal surfaces of varying roughness
//! using the Torrance Sparrow BRDF and a Blinn microfacet distribution
//! TODO: Add Ashikman-Shirley (spelling?) anisotropic microfacet model

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow};
use bxdf::microfacet::{MicrofacetDistribution, Blinn};
use bxdf::fresnel::{Fresnel, Conductor};
use material::Material;

/// The Metal material describes metals of varying roughness
pub struct Metal {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
}

impl Metal {
    /// Create a new metal material specifying the reflectance properties of the metal
    pub fn new(eta: &Colorf, k: &Colorf, roughness: f32) -> Metal {
        let fresnel = Box::new(Conductor::new(eta, k)) as Box<Fresnel + Send + Sync>;
        let microfacet = Box::new(Blinn::new(1.0 / roughness)) as Box<MicrofacetDistribution + Send + Sync>;
        Metal { bxdfs: vec![Box::new(TorranceSparrow::new(&Colorf::broadcast(1.0), fresnel, microfacet))
                            as Box<BxDF + Send + Sync>] }
    }
}

impl Material for Metal {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, 1.0, &hit.dg)
    }
}


