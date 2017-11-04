//! Defines a specular glass material
//!
//! # Scene Usage Example
//! The specular glass material describes a thin glass surface type of material,
//! not a solid block of glass (there is no absorption of light). The glass requires
//! a reflective and emissive color along with a refrective index, eta.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "clear_glass",
//!         "type": "glass",
//!         "reflect": [1, 1, 1],
//!         "transmit": [1, 1, 1],
//!         "eta": 1.52
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;

use light_arena::Allocator;

use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection, SpecularTransmission};
use bxdf::fresnel::Dielectric;
use material::Material;
use texture::Texture;

/// The Glass material describes specularly transmissive and reflective glass material
pub struct Glass {
    reflect: Arc<Texture + Send + Sync>,
    transmit: Arc<Texture + Send + Sync>,
    eta: Arc<Texture + Send + Sync>,
}

impl Glass {
    /// Create the glass material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    pub fn new(reflect: Arc<Texture + Send + Sync>,
               transmit: Arc<Texture + Send + Sync>,
               eta: Arc<Texture + Send + Sync>) -> Glass {
        Glass { reflect: reflect, transmit: transmit, eta: eta }
    }
}

impl Material for Glass {
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        // TODO: I don't like this counting and junk we have to do to figure out
        // the slice size and then the indices. Is there a better way?
        let reflect = self.reflect.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let transmit = self.transmit.sample_color(hit.dg.u, hit.dg.v, hit.dg.time);
        let eta = self.eta.sample_f32(hit.dg.u, hit.dg.v, hit.dg.time);

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
        if !reflect.is_black() {
            bxdfs[i] = alloc <- SpecularReflection::new(&reflect, fresnel);
            i += 1;
        }
        if !transmit.is_black() {
            bxdfs[i] = alloc <- SpecularTransmission::new(&transmit, fresnel);
        }
        BSDF::new(bxdfs, eta, &hit.dg)
    }
}


