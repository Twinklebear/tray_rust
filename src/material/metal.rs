//! Provides a material for modelling metal surfaces of varying roughness
//! using the Torrance Sparrow BRDF and a Blinn microfacet distribution
//! TODO: Add Ashikman-Shirley (spelling?) anisotropic microfacet model
//!
//! # Scene Usage Example
//! The metal material requires a refractive index and absorption coefficient
//! that describe the physical properties of the metal along with a roughness
//! to specify how rough the surface of the metal is.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "rough_silver",
//!         "type": "metal",
//!         "refractive_index": [0.155265, 0.116723, 0.138381],
//!         "absorption_coefficient": [4.82835, 3.12225, 2.14696],
//!         "roughness": 0.3
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, TorranceSparrow};
use bxdf::microfacet::Beckmann;
use bxdf::fresnel::Conductor;
use material::Material;
use texture::Texture;

/// The Metal material describes metals of varying roughness
pub struct Metal {
    eta: Arc<Texture + Send + Sync>,
    k: Arc<Texture + Send + Sync>,
    roughness: Arc<Texture + Send + Sync>,
}

impl Metal {
    /// Create a new metal material specifying the reflectance properties of the metal
    pub fn new(eta: Arc<Texture + Send + Sync>,
               k: Arc<Texture + Send + Sync>,
               roughness: Arc<Texture + Send + Sync>) -> Metal
    {
        Metal { eta: eta.clone(),
                k: k.clone(),
                roughness: roughness.clone()
        }
    }
}

impl Material for Metal {
    fn bsdf<'a, 'b, 'c>(&self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        let eta = self.eta.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let k = self.k.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let roughness = self.roughness.sample_f32(hit.dg.u, hit.dg.v, hit.dg.time);

        let bxdfs = alloc.alloc_slice::<&BxDF>(1);
        let fresnel = alloc <- Conductor::new(&eta, &k);
        let microfacet = alloc <- Beckmann::new(roughness);
        bxdfs[0] = alloc <- TorranceSparrow::new(&Colorf::broadcast(1.0), fresnel, microfacet);
        BSDF::new(bxdfs, 1.0, &hit.dg)
    }
}


