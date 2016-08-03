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

use std::vec::Vec;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, MicrofacetTransmission, TorranceSparrow};
use bxdf::microfacet::{Beckmann, MicrofacetDistribution};
use bxdf::fresnel::{Dielectric, Fresnel};
use material::Material;

/// The `RoughGlass` material describes specularly transmissive and reflective glass material
pub struct RoughGlass {
    bxdfs: Vec<Box<BxDF + Send + Sync>>,
    eta: f32,
}

impl RoughGlass {
    /// Create the `RoughGlass` material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    /// `roughness`: roughness of the material
    pub fn new(reflect: &Colorf, transmit: &Colorf, eta: f32, roughness: f32) -> RoughGlass {
        let mut bxdfs = Vec::new();
        if !reflect.is_black() {
            let fresnel = Box::new(Dielectric::new(1.0, eta)) as Box<Fresnel + Send + Sync>;
            let microfacet = Box::new(Beckmann::new(roughness)) as Box<MicrofacetDistribution + Send + Sync>;
            bxdfs.push(Box::new(TorranceSparrow::new(reflect, fresnel, microfacet)) as Box<BxDF + Send + Sync>);
        }
        if !transmit.is_black() {
            let fresnel = Dielectric::new(1.0, eta);
            let microfacet = Box::new(Beckmann::new(roughness)) as Box<MicrofacetDistribution + Send + Sync>;
            bxdfs.push(Box::new(MicrofacetTransmission::new(transmit, fresnel, microfacet))
                       as Box<BxDF + Send + Sync>);
        }
        RoughGlass { bxdfs: bxdfs, eta: eta }
    }
}

impl Material for RoughGlass {
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a> {
        BSDF::new(&self.bxdfs, self.eta, &hit.dg)
    }
}



