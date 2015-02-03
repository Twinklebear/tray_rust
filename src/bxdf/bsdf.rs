//! Defines the BSDF which acts as a container for composing the various BRDFs
//! and BTDFs that describe the surface's properties

use std::vec::Vec;
use std::cmp;
use collect::enum_set::EnumSet;

use linalg;
use linalg::{Normal, Vector, Point};
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
    /// The hit point
    pub p: Point,
    /// Shading normal, may be perturbed by bump mapping
    pub n: Normal,
    /// The actual geometry normal
    pub ng: Normal,
    /// Tangent vector for the surface
    pub tan: Vector,
    /// Bitangent vector for the surface
    pub bitan: Vector,
    /// Refractive index of the geometry
    pub eta: f32,
    /// TODO: Currently a Vec is safe to use but once in the memory pool it
    /// will leak since it won't be dropped. This would also migrate our BxDFs
    /// from Box<BxDF> to &BxDF. When unboxed traits land we can move to unboxed
    /// BxDFs here though.
    bxdfs: &'a Vec<Box<BxDF + Send + Sync>>,
}

impl<'a> BSDF<'a> {
    /// Create a new BSDF using the BxDFs passed to shade the differential geometry with
    /// refractive index `eta`
    pub fn new(bxdfs: &'a Vec<Box<BxDF + Send + Sync>>, eta: f32,
               dg: &DifferentialGeometry<'a>)
               -> BSDF<'a> {
        let n = dg.n.normalized();
        let bitan = dg.dp_du.normalized();
        let tan = linalg::cross(&n, &bitan);
        BSDF { p: dg.p, n: n, ng: dg.ng, tan: tan, bitan: bitan, bxdfs: bxdfs, eta: eta }
    }
    /// Return the total number of BxDFs
    pub fn num_bxdfs(&self) -> usize { self.bxdfs.len() }
    /// Return the number of BxDFs matching the flags
    pub fn num_matching(&self, flags: EnumSet<BxDFType>) -> usize {
        self.bxdfs.iter().filter(|ref x| x.matches(flags)).count()
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
    /// selected by the flags passed. `wo_world` and `wi_world` should point from
    /// the hit point in the outgoing and incident light directions respectively.
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
        // Find all matching BxDFs and add their contribution to the material's color
        self.bxdfs.iter().filter_map(|ref x| if x.matches(flags) { Some(x.eval(&w_o, &w_i)) } else { None })
            .fold(Colorf::broadcast(0.0), |x, y| x + y)
    }
    /// Sample a component of the BSDF to get an incident light direction for light
    /// leaving the surface along `w_o`.
    /// `samples` are the 3 random values to use when sampling a component of the BSDF
    /// and a the chosen BSDF
    /// Returns the color, direction, pdf and the type of BxDF that was sampled.
    pub fn sample(&self, wo_world: &Vector, flags: EnumSet<BxDFType>, samples: &[f32])
        -> (Colorf, Vector, f32, EnumSet<BxDFType>) {
        // TODO: Is there a better way to accept slices but require they be of some length?
        assert!(samples.len() > 2);

        let n_matching = self.num_matching(flags);
        if n_matching == 0 {
            return (Colorf::broadcast(0.0), Vector::broadcast(0.0), 0.0, EnumSet::new());
        }
        let comp = cmp::min((samples[0] * n_matching as f32) as usize, n_matching - 1);
        let bxdf = self.matching_at(comp, flags);
        let w_o = self.to_shading(wo_world);
        let (f, w_i, pdf) = bxdf.sample(&w_o, &samples[1..]);
        // TODO sample other mats if non-specular
        (f, self.from_shading(&w_i), pdf, bxdf.bxdf_type())
    }
    /// Compute the pdf for sampling the pair of incident and outgoing light directions for
    /// the BxDFs matching the flags set
    pub fn pdf(&self, wo_world: &Vector, wi_world: &Vector, flags: EnumSet<BxDFType>) -> f32 {
        let w_o = self.to_shading(wo_world);
        let w_i = self.to_shading(wi_world);
        let (pdf_val, n_comps) = self.bxdfs.iter()
            .filter_map(|ref x| if x.matches(flags) { Some(x.pdf(&w_o, &w_i)) } else { None })
            .fold((0.0, 0), |(p, n), y| (p + y, n + 1));
        if n_comps == 0 {
            0.0
        } else {
            pdf_val / n_comps as f32
        }
    }
    /// Get the `i`th BxDF that matches the flags passed. There should not be fewer than i
    /// BxDFs that match the flags
    fn matching_at(&self, i: usize, flags: EnumSet<BxDFType>) -> &Box<BxDF + Send + Sync> {
        let mut it = self.bxdfs.iter().filter(|ref x| x.matches(flags)).skip(i);
        match it.next() {
            Some(b) => b,
            None => panic!("Out of bounds index for BxDF type {:?}", flags)
        }
    }
}

