//! Defines the trait implemented by all materials and exports various
//! supported material types. Materials are used to define how BxDFs are
//! composed to create the desired appearance
//!
//! # Scene Usage Example
//! The material will be specified within the materials list of the scene object. A type
//! and name for the material along with any additional parameters is required to specify one.
//! The name is used when specifying which material should be used by an object in the scene.
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "my_material",
//!         "type": "The_Material_Type",
//!          ...
//!     }
//!     ...
//! ]
//! ```

use light_arena::Allocator;

use geometry::Intersection;
use bxdf::BSDF;

pub use self::matte::Matte;
pub use self::specular_metal::SpecularMetal;
pub use self::glass::Glass;
pub use self::merl::Merl;
pub use self::plastic::Plastic;
//pub use self::metal::Metal;
pub use self::rough_glass::RoughGlass;

pub mod matte;
pub mod specular_metal;
pub mod glass;
pub mod merl;
pub mod plastic;
//pub mod metal;
pub mod rough_glass;

/// Trait implemented by materials. Provides method to get the BSDF describing
/// the material properties at the intersection
pub trait Material {
    /// Get the BSDF for the material which defines its properties at the hit point.
    ///
    /// We have the lifetime constraint on the returned BSDF to enforce it does not
    /// outlive the material which produced it. This allows us to borrow things from
    /// the parent material in the BxDFs making up the BSDF.
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c;
}

