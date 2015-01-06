//! Defines the BSDF which acts as a container for composing the various BRDFs
//! and BTDFs that describe the surface's properties

use std::vec::Vec;
use std::collections::enum_set::EnumSet;

use linalg;
use linalg::{Normal, Vector};
use film::Colorf;
use geometry::DifferentialGeometry;
use bxdf::{BxDF, BxDFType};

/// The BSDF contains the various BRDFs and BTDFs that describe the surface's properties
/// at some point. It also transforms incident and outgoing light directions into
/// shading space to make the BxDFs easier to implement.
/// TODO: We really need the memory pool. Each time we get the bsdf from a
/// material we need to allocate a decent amount of stuff since they each need
/// their own tangent, bitangent and differential geometry reference.
pub struct BSDF<'a> {
    /// Shading normal, may be perturbed by bump mapping
    n: Normal,
    /// The actual geometry normal
    ng: Normal,
    /// Tangent vector for the surface
    tan: Vector,
    /// Bitangent vector for the surface
    bitan: Vector,
    /// TODO: Currently a Vec is safe to use but once in the memory pool it
    /// will leak since it won't be dropped. This would also migrate our BxDFs
    /// from Box<BxDF> to &BxDF. When unboxed traits land we can move to unboxed
    /// BxDFs here though.
    bxdfs: &'a Vec<Box<BxDF + 'static>>,
    /// Refractive index of the geometry
    pub eta: f32,
}

impl<'a> BSDF<'a> {
    /// Create a new BSDF using the BxDFs passed to shade the differential geometry with
    /// refractive index `eta`
    pub fn new(bxdfs: &'a Vec<Box<BxDF + 'static>>, eta: f32, dg: &DifferentialGeometry<'a>) -> BSDF<'a> {
        let n = dg.n.normalized();
        let tan = linalg::cross(&n,  &dg.dp_du.normalized()).normalized();
        let bitan = linalg::cross(&tan, &n).normalized();
        BSDF { n: n, ng: dg.ng.normalized(), tan: tan, bitan: bitan, bxdfs: bxdfs, eta: eta }
    }
    /// Return the total number of BxDFs
    pub fn num_bxdfs(&self) -> uint { self.bxdfs.len() }
    /// Return the number of BxDFs matching the flags
    pub fn num_matching(&self, flags: EnumSet<BxDFType>) -> uint {
        self.bxdfs.iter().filter(|&x| x.matches(flags)).count()
    }
    /// Transform the vector from world space to shading space
    pub fn to_shading(&self, v: &Vector) -> Vector {
        Vector::new(linalg::dot(v, &self.bitan), linalg::dot(v, &self.tan),
                    linalg::dot(v, &self.n))
    }
    /// Transform the vectro from shading space to world space
    pub fn from_shading(&self, v: &Vector) -> Vector {
        Vector::new(self.bitan.x * v.x + self.tan.x * v.y + self.n.x * v.z,
                    self.bitan.y * v.x + self.tan.y * v.y + self.n.y * v.z,
                    self.bitan.z * v.x + self.tan.z * v.y + self.n.z * v.z)
    }
    /// Evaluate the BSDF for the outgoing and incident light directions
    /// `w_o` and `w_i` in world space, sampling the desired subset of BxDFs
    /// selected by the flags passed
    pub fn eval(&self, wo_world: &Vector, wi_world: &Vector, mut flags: EnumSet<BxDFType>) -> Colorf {
        let w_o = self.to_shading(wo_world);
        let w_i = self.to_shading(wi_world);
        // Determine if we should evaluate reflection or transmission based on the
        // geometry normal and the light directions
        if linalg::dot(wo_world, &self.ng) * linalg::dot(wi_world, &self.ng) > 0.0 {
            flags.remove(&BxDFType::Transmission);
        } else {
            flags.remove(&BxDFType::Reflection);
        }
        // Find all matching BxDFs and add their contribution to the material's
        // final color
        self.bxdfs.iter().filter(|&x| x.matches(flags)).map(|ref x| x.eval(&w_o, &w_i))
            .fold(Colorf::broadcast(0.0), |x, y| x + y)
    }
}

