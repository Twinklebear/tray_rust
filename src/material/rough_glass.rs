//! Defines a rough glass material
//!
//! # Scene Usage Example
//! The rough glass material describes a thin glass surface material,
//! not a solid block of glass (there is no absorption of light). The glass requires
//! a reflective and emissive color along with a refrective index, eta and roughness.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "foggy_glass",
//!         "type": "rough_glass",
//!         "reflect": [1, 1, 1],
//!         "transmit": [1, 1, 1],
//!         "eta": 1.52,
//!         "roughness": 0.5,
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, MicrofacetTransmission, TorranceSparrow};
use bxdf::microfacet::Beckmann;
use bxdf::fresnel::Dielectric;
use material::Material;
use texture::Texture;

/// The `RoughGlass` material describes specularly transmissive and reflective glass material
pub struct RoughGlass {
    reflect: Arc<Texture<Colorf> + Send + Sync>,
    transmit: Arc<Texture<Colorf> + Send + Sync>,
    eta: Arc<Texture<f32> + Send + Sync>,
    roughness: Arc<Texture<f32> + Send + Sync>,
}

impl RoughGlass {
    /// Create the `RoughGlass` material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    /// `roughness`: roughness of the material
    pub fn new(reflect: Arc<Texture<Colorf> + Send + Sync>,
               transmit: Arc<Texture<Colorf> + Send + Sync>,
               eta: Arc<Texture<f32> + Send + Sync>,
               roughness: Arc<Texture<f32> + Send + Sync>) -> RoughGlass
    {
        RoughGlass { reflect: reflect, transmit: transmit, eta: eta, roughness: roughness }
    }
}

impl Material for RoughGlass {
    fn bsdf<'a, 'b, 'c>(&self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        let reflect = self.reflect.sample(hit.dg.u, hit.dg.v, hit.dg.time);
        let transmit = self.transmit.sample(hit.dg.u, hit.dg.v, hit.dg.time);
        let eta = self.eta.sample(hit.dg.u, hit.dg.v, hit.dg.time);
        let roughness = self.roughness.sample(hit.dg.u, hit.dg.v, hit.dg.time);

        let mut num_bxdfs = 0;
        if !reflect.is_black() {
            num_bxdfs += 1;
        }
        if !transmit.is_black() {
            num_bxdfs += 1;
        }

        let bxdfs = alloc.alloc_slice::<&BxDF>(num_bxdfs);
        let mut i = 0;
        let fresnel = alloc <- Dielectric::new(1.0, eta);
        let microfacet = alloc <- Beckmann::new(roughness);
        if !reflect.is_black() {
            bxdfs[i] = alloc <- TorranceSparrow::new(&reflect, fresnel, microfacet);
            i += 1;
        }
        if !transmit.is_black() {
            println!("i = {}", i);
            bxdfs[i] = alloc <- MicrofacetTransmission::new(&transmit, fresnel, microfacet);
        }
        BSDF::new(bxdfs, eta, &hit.dg)
    }
}



