//! Defines the trait implemented by all materials and exports various
//! supported material types. Materials are used to define how BxDFs are
//! composed to create the desired appearance

use geometry::Intersection;
use bxdf::BSDF;

pub use self::matte::Matte;
pub use self::specular_metal::SpecularMetal;
pub use self::glass::Glass;

pub mod matte;
pub mod specular_metal;
pub mod glass;

/// Trait implemented by materials. Provides method to get the BSDF describing
/// the material properties at the intersection
pub trait Material {
    /// Get the BSDF for the material which defines its properties at the
    /// hit point. TODO: When we implement a memory pool we need to pass it
    /// here, currently the BxDFs and BSDF are allocated once at surface
    /// creation instead of as needed based on material properties.
    fn bsdf<'a, 'b>(&'a self, hit: &Intersection<'a, 'b>) -> BSDF<'a>;
}

