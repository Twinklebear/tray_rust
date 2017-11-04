//! Defines a specular metal material
//!
//! # Scene Usage Example
//! The specular metal material requires a refractive index and absorption coefficient
//! that describe the physical properties of the metal.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "specular_silver",
//!         "type": "specular_metal",
//!         "refractive_index": [0.155265, 0.116723, 0.138381],
//!         "absorption_coefficient": [4.82835, 3.12225, 2.14696]
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection};
use bxdf::fresnel::Conductor;
use material::Material;
use texture::Texture;

/// The Specular Metal material describes specularly reflective metals using their
/// refractive index and absorption coefficient
pub struct SpecularMetal {
    eta: Arc<Texture + Send + Sync>,
    k: Arc<Texture + Send + Sync>,
}

impl SpecularMetal {
    /// Create a new specular metal with the desired metal properties.
    /// `eta`: refractive index of the metal
    /// `k`: absorption coefficient of the metal
    pub fn new(eta: Arc<Texture + Send + Sync>,
               k: Arc<Texture + Send + Sync>) -> SpecularMetal
    {
        SpecularMetal { eta: eta.clone(), k: k.clone() }
    }
}

impl Material for SpecularMetal {
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        let eta = self.eta.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let k = self.k.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);

        let bxdfs = alloc.alloc_slice::<&BxDF>(1);
        let fresnel = alloc <- Conductor::new(&eta, &k);
        bxdfs[0] = alloc <- SpecularReflection::new(&Colorf::broadcast(1.0), fresnel);
        BSDF::new(bxdfs, 1.0, &hit.dg)
    }
}

