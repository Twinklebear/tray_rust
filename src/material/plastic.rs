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

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow, Lambertian};
use bxdf::microfacet::Beckmann;
use bxdf::fresnel::Dielectric;
use material::Material;

/// The Plastic material describes plastic materials of varying roughness
pub struct Plastic {
    diffuse: Colorf,
    gloss: Colorf,
    roughness: f32,
}

impl Plastic {
    /// Create a new plastic material specifying the diffuse and glossy colors
    /// along with the roughness of the surface
    pub fn new(diffuse: &Colorf, gloss: &Colorf, roughness: f32) -> Plastic {
        Plastic { diffuse: *diffuse, gloss: *gloss, roughness: roughness }
    }
}

impl Material for Plastic {
    fn bsdf<'a, 'b, 'c>(&self, hit: &Intersection<'a, 'b>, alloc: &'c Allocator) -> BSDF<'c> {
        // TODO: I don't like this counting and junk we have to do to figure out
        // the slice size and then the indices. Is there a better way?
        let mut num_bxdfs = 0;
        if !self.diffuse.is_black() {
            num_bxdfs += 1;
        }
        if !self.gloss.is_black() {
            num_bxdfs += 1;
        }
        let bxdfs = alloc.alloc_slice::<&BxDF>(num_bxdfs);
        
        let mut i = 0;
        if !self.diffuse.is_black() {
            bxdfs[i] = alloc <- Lambertian::new(&self.diffuse);
            i += 1;
        }
        if !self.gloss.is_black() {
            let fresnel = alloc <- Dielectric::new(1.0, 1.5);
            let microfacet = alloc <- Beckmann::new(self.roughness);
            bxdfs[i] = alloc <- TorranceSparrow::new(&self.gloss, fresnel, microfacet);
        }
        BSDF::new(bxdfs, 1.0, &hit.dg)
    }
}

