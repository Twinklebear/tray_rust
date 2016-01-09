//! A material that models plastic of varying roughness using
//! the Torrance Sparrow BRDF and a Blinn microfacet distribution
//! TODO: Add Ashikman-Shirley (spelling?) anisotropic microfacet model
//!
//! # Scene Usage Example
//! The plastic material requires a diffuse and glossy color. The diffuse color
//! is used by a Lambertian model and the gloss color is used by a Torrance-Sparrow
//! microfacet model with a Blinn microfacet distribution. The roughness will specify
//! how reflective the gloss color is while the diffuse color provides a uniform base color
//! for the object.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "red_plastic",
//!         "type": "plastic",
//!         "diffuse": [0.8, 0, 0],
//!         "gloss": [1, 1, 1],
//!         "roughness": 0.05
//!     },
//!     ...
//! ]
//! ```

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow, Lambertian};
use bxdf::microfacet::{MicrofacetDistribution, Beckmann};
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
            let fresnel = Box::new(Dielectric::new(1.0, 1.5)) as Box<Fresnel + Send + Sync>;
            let microfacet = Box::new(Beckmann::new(roughness)) as Box<MicrofacetDistribution + Send + Sync>;
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

