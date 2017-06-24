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

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, SpecularReflection, SpecularTransmission};
use bxdf::fresnel::Dielectric;
use material::Material;

/// The Glass material describes specularly transmissive and reflective glass material
pub struct Glass {
    reflect: Colorf,
    transmit: Colorf,
    eta: f32,
}

impl Glass {
    /// Create the glass material with the desired color and index of refraction
    /// `reflect`: color of reflected light
    /// `transmit`: color of transmitted light
    /// `eta`: refractive index of the material
    pub fn new(reflect: &Colorf, transmit: &Colorf, eta: f32) -> Glass {
        Glass { reflect: *reflect, transmit: *transmit, eta: eta }
    }
}

impl Material for Glass {
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>, alloc: &'c Allocator) -> BSDF<'c> {
        // TODO: I don't like this counting and junk we have to do to figure out
        // the slice size and then the indices. Is there a better way?
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
            bxdfs[i] = alloc <- SpecularReflection::new(&self.reflect, fresnel);
            i += 1;
        }
        if !self.transmit.is_black() {
            bxdfs[i] = alloc <- SpecularTransmission::new(&self.transmit, fresnel);
        }
        BSDF::new(bxdfs, self.eta, &hit.dg)
    }
}


