//! Defines the BxDF interface implemented by BRDF/BTDFs to describe
//! material properties. Also provides the BSDF type which composes
//! various BRDF/BTDFs to describe materials

use std::collections::enum_set::{EnumSet, CLike};
use std::mem;
use std::num::{FloatMath, Float};

use linalg;
use linalg::Vector;
use film::Colorf;

pub use self::bsdf::BSDF;
pub use self::lambertian::Lambertian;
pub use self::oren_nayar::OrenNayar;

pub mod bsdf;
pub mod lambertian;
pub mod oren_nayar;

/// Various types of BxDFs that can be selected to specify which
/// types of surface functions should be evaluated
#[repr(uint)]
#[derive(Copy, Show)]
pub enum BxDFType {
    Reflection, Transmission, Diffuse, Glossy, Specular,
}

impl BxDFType {
    /// Get an EnumSet containing all flags for the different types of
    /// BxDFs: Diffuse, Glossy, Specular
    pub fn all_types() -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Diffuse);
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Specular);
        e
    }
    /// Get an EnumSet containing all flags for reflective BxDFs (eg. BRDFs)
    pub fn all_brdf() -> EnumSet<BxDFType> {
        let mut e = BxDFType::all_types();
        e.insert(BxDFType::Reflection);
        e
    }
    /// Get an EnumSet containing all flags for transmissive BxDFs (eg. BTDFs)
    pub fn all_btdf() -> EnumSet<BxDFType> {
        let mut e = BxDFType::all_types();
        e.insert(BxDFType::Transmission);
        e
    }
    /// Get an EnumSet containing all flags for all BxDFs. This would be all
    /// types of BRDFs and BTDFs
    pub fn all() -> EnumSet<BxDFType> { BxDFType::all_brdf().union(BxDFType::all_btdf()) }
    /// Get an EnumSet containing flags for all types of specular BxDFs
    pub fn specular() -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Specular);
        e.insert(BxDFType::Reflection);
        e.insert(BxDFType::Transmission);
        e
    }
    /// Get an EnumSet containing flags for all non-specular BxDFs
    pub fn non_specular() -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Diffuse);
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Reflection);
        e.insert(BxDFType::Transmission);
        e
    }
}

impl CLike for BxDFType {
    fn to_uint(&self) -> uint { *self as uint }
    fn from_uint(v: uint) -> BxDFType { unsafe { mem::transmute(v) } }
}

/// Trait implemented by BRDF/BTDFs in tray_rust. Provides methods for
/// evaluating and sampling the function
pub trait BxDF {
    /// Get the type of this BxDF
    fn bxdf_type(&self) -> EnumSet<BxDFType>;
    /// Evaluate the BxDF for the pair of incident and outgoing light directions,
    /// `w_i` and `w_o`.
    fn eval(&self, w_o: &Vector, w_i: &Vector) -> Colorf;
    /// Check if this BxDF matches the type flags passed
    fn matches(&self, flags: EnumSet<BxDFType>) -> bool {
        self.bxdf_type().is_subset(&flags)
    }
}

/// Compute the value of cosine theta for a vector in shading space
pub fn cos_theta(v: &Vector) -> f32 { v.z }
/// Compute the value of (sine theta)^2  for a vector in shading space
pub fn sin_theta_sqr(v: &Vector) -> f32 { FloatMath::max(0.0, 1.0 - v.z * v.z) }
/// Compute the value of sine theta for a vector in shading space
pub fn sin_theta(v: &Vector) -> f32 { Float::sqrt(sin_theta_sqr(v)) }
/// Compute the value of cosine phi for a vector in shading space
pub fn cos_phi(v: &Vector) -> f32 {
    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        1.0
    } else {
        linalg::clamp(v.x / sin_theta, -1.0, 1.0)
    }
}
/// Compute the value of sine phi for a vector in shading space
pub fn sin_phi(v: &Vector) -> f32 {
    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        0.0
    } else {
        linalg::clamp(v.y / sin_theta, -1.0, 1.0)
    }
}
/// Check if two vectors are in the same hemisphere in shading space
pub fn same_hemisphere(a: &Vector, b: &Vector) -> bool { a.z * b.z > 0.0 }

