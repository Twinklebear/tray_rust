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

use std::sync::Arc;

use light_arena::Allocator;

use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow, Lambertian};
use bxdf::microfacet::Beckmann;
use bxdf::fresnel::Dielectric;
use material::Material;
use texture::Texture;

/// The Plastic material describes plastic materials of varying roughness
pub struct Plastic {
    diffuse: Arc<Texture + Send + Sync>,
    gloss: Arc<Texture + Send + Sync>,
    roughness: Arc<Texture + Send + Sync>,
}

impl Plastic {
    /// Create a new plastic material specifying the diffuse and glossy colors
    /// along with the roughness of the surface
    pub fn new(diffuse: Arc<Texture + Send + Sync>,
               gloss: Arc<Texture + Send + Sync>,
               roughness: Arc<Texture + Send + Sync>) -> Plastic
    {
        Plastic {
            diffuse: diffuse.clone(),
            gloss: gloss.clone(),
            roughness: roughness.clone()
        }
    }
}

impl Material for Plastic {
    fn bsdf<'a, 'b, 'c>(&self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c
    {
        let diffuse = self.diffuse.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let gloss = self.gloss.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let roughness = self.roughness.sample_f32(hit.dg.u, hit.dg.v, hit.dg.time);

        // TODO: I don't like this counting and junk we have to do to figure out
        // the slice size and then the indices. Is there a better way?
        let mut num_bxdfs = 0;
        if !diffuse.is_black() {
            num_bxdfs += 1;
        }
        if !gloss.is_black() {
            num_bxdfs += 1;
        }
        let bxdfs = alloc.alloc_slice::<&BxDF>(num_bxdfs);

        let mut i = 0;
        if !diffuse.is_black() {
            bxdfs[i] = alloc.alloc(Lambertian::new(&diffuse));
            i += 1;
        }
        if !gloss.is_black() {
            let fresnel = alloc.alloc(Dielectric::new(1.0, 1.5));
            let microfacet = alloc.alloc(Beckmann::new(roughness));
            bxdfs[i] = alloc.alloc(TorranceSparrow::new(&gloss, fresnel, microfacet));
        }
        BSDF::new(bxdfs, 1.0, &hit.dg)
    }
}

