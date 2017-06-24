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

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, MicrofacetTransmission, TorranceSparrow};
use bxdf::microfacet::Beckmann;
use bxdf::fresnel::Dielectric;
use material::Material;

/// The `RoughGlass` material describes specularly transmissive and reflective glass material
pub struct RoughGlass {
    reflect: Colorf,
    transmit: Colorf,
    eta: f32,
    roughness: f32,
}

impl RoughGlass {
    /// Create the `RoughGlass` material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    /// `roughness`: roughness of the material
    pub fn new(reflect: &Colorf, transmit: &Colorf, eta: f32, roughness: f32) -> RoughGlass {
        RoughGlass { reflect: *reflect, transmit: *transmit, eta: eta, roughness: roughness }
    }
}

impl Material for RoughGlass {
    fn bsdf<'a, 'b, 'c>(&self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        let mut num_bxdfs = 0;
        if !self.reflect.is_black() {
            num_bxdfs += 1;
        }
        if !self.transmit.is_black() {
            num_bxdfs += 1;
        }

        let bxdfs = alloc.alloc_slice::<&BxDF>(num_bxdfs);
        let mut i = 0;
        let fresnel = alloc <- Dielectric::new(1.0, self.eta);
        if !self.reflect.is_black() {
            let microfacet = alloc <- Beckmann::new(self.roughness);
            bxdfs[i] = alloc <- TorranceSparrow::new(&self.reflect, fresnel, microfacet);
            i += 1;
        }
        if !self.transmit.is_black() {
            let microfacet = alloc <- Beckmann::new(self.roughness);
            bxdfs[i] = alloc <- MicrofacetTransmission::new(&self.transmit, fresnel, microfacet);
        }
        BSDF::new(bxdfs, self.eta, &hit.dg)
    }
}



