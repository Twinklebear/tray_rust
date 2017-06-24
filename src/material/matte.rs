//! Defines a matte material used to describe diffuse materials
//!
//! # Scene Usage Example
//! The matte material requires a diffuse color and a roughness for the material. A roughness of 0
//! will select a [Lambertian](https://en.wikipedia.org/wiki/Lambertian_reflectance) model
//! while a roughness > 0 will select an
//! [Oren-Nayar](https://en.wikipedia.org/wiki/Oren%E2%80%93Nayar_reflectance_model)
//! reflectance model.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "purple_matte",
//!         "type": "matte",
//!         "diffuse": [1, 0, 1],
//!         "roughness" 0.5
//!     },
//!     ...
//! ]
//! ```

use light_arena::Allocator;

use film::Colorf;
use geometry::Intersection;
use bxdf::{BxDF, BSDF, Lambertian, OrenNayar};
use material::Material;

/// The Matte material describes diffuse materials with either a Lambertian or
/// Oren-Nayar BRDF. The Lambertian BRDF is used for materials with no roughness
/// while Oren-Nayar is used for those with some roughness.
pub struct Matte {
    diffuse: Colorf,
    roughness: f32,
}

impl Matte {
    /// Create a new Matte material with the desired diffuse color and roughness
    pub fn new(diffuse: &Colorf, roughness: f32) -> Matte {
        Matte { diffuse: *diffuse, roughness: roughness }
    }
}

impl Material for Matte {
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>, alloc: &'c Allocator) -> BSDF<'c> {
        let bsdfs = alloc.alloc_slice::<&'c BxDF>(1);
        if self.roughness == 0.0 {
            bsdfs[0] = alloc <- Lambertian::new(&self.diffuse);
        } else {
            bsdfs[0] = alloc <- OrenNayar::new(&self.diffuse, self.roughness);
        }
        BSDF::new(bsdfs, 1.0, &hit.dg)
    }
}

