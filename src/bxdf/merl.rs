//! The MERL BRDF represents the surface's properties through data loaded from a
//! [MERL BRDF Database file](http://www.merl.com/brdf/). The BRDF itself just stores
//! the data loaded from the BRDF file while actual loading is done by the MERL material
//! when it's created.

use std::f32;
use enum_set::EnumSet;

use linalg::{self, Vector};
use film::Colorf;
use bxdf::{self, BxDF, BxDFType};

/// BRDF that uses measured data to model the surface reflectance properties.
/// The measured data is from "A Data-Driven Reflectance Model",
/// by Wojciech Matusik, Hanspeter Pfister, Matt Brand and Leonard McMillan,
/// in ACM Transactions on Graphics 22, 3(2003), 759-769
#[derive(Clone, Debug)]
pub struct Merl {
    /// Vec containing the BRDF values for various incident/exiting angles 
    brdf: Vec<f32>,
    /// Number of theta_h measurements in `brdf`
    n_theta_h: usize,
    /// Number of theta_d measurements in `brdf`
    n_theta_d: usize,
    /// Number of phi_d measurements in `brdf`
    n_phi_d: usize,
}

impl Merl {
    /// Create a MERL BRDF to use data loaded from a MERL BRDF data file
    pub fn new(brdf: Vec<f32>, n_theta_h: usize, n_theta_d: usize, n_phi_d: usize) -> Merl {
        Merl { brdf: brdf, n_theta_h: n_theta_h, n_theta_d: n_theta_d, n_phi_d: n_phi_d }
    }
    /// Re-map values from an angular value to the index in the MERL data table
    fn map_index(val: f32, max: f32, n_vals: usize) -> usize {
        linalg::clamp((val / max * n_vals as f32) as usize, 0, n_vals - 1)
    }
}

impl BxDF for Merl {
    fn bxdf_type(&self) -> EnumSet<BxDFType> {
        let mut e = EnumSet::new();
        e.insert(BxDFType::Glossy);
        e.insert(BxDFType::Reflection);
        e
    }
    fn eval(&self, w_oi: &Vector, w_ii: &Vector) -> Colorf {
        // Find the half-vector and transform into the half angle coordinate system used by MERL
        // BRDF files
        let (w_o, w_i, w_h) =
            if w_oi.z + w_ii.z < 0.0 {
                (-*w_oi, -*w_ii, -(*w_oi + *w_ii))
            } else {
                (*w_oi, *w_ii, *w_oi + *w_ii)
            };

        if w_h.length_sqr() == 0.0 {
            return Colorf::black();
        }

        let w_h = w_h.normalized();
        // Directly compute the rows of the matrix performing the rotation of w_h to (0, 0, 1)
        let theta_h = linalg::spherical_theta(&w_h);
        let cos_phi_h = bxdf::cos_phi(&w_h);
        let sin_phi_h = bxdf::sin_phi(&w_h);
        let cos_theta_h = bxdf::cos_theta(&w_h);
        let sin_theta_h = bxdf::sin_theta(&w_h);
        let w_hx = Vector::new(cos_phi_h * cos_theta_h, sin_phi_h * cos_theta_h, -sin_theta_h);
        let w_hy = Vector::new(-sin_phi_h, cos_phi_h, 0.0);
        let w_d = Vector::new(linalg::dot(&w_i, &w_hx), linalg::dot(&w_i, &w_hy), linalg::dot(&w_i, &w_h));
        let theta_d = linalg::spherical_theta(&w_d);
        // Wrap phi_d if needed to keep it in range
        let phi_d = match linalg::spherical_phi(&w_d) {
            d if d > f32::consts::PI => d - f32::consts::PI,
            d => d,
        };
        let theta_h_idx = Merl::map_index(f32::sqrt(f32::max(0.0, 2.0 * theta_h / f32::consts::PI)), 1.0, self.n_theta_h);
        let theta_d_idx = Merl::map_index(theta_d, f32::consts::PI / 2.0, self.n_theta_d);
        let phi_d_idx = Merl::map_index(phi_d, f32::consts::PI, self.n_phi_d);
        let i = phi_d_idx + self.n_phi_d * (theta_d_idx + theta_h_idx * self.n_theta_d);
        assert!(i < self.brdf.len());
        Colorf::new(self.brdf[3 * i], self.brdf[3 * i + 1], self.brdf[3 * i + 2])
    }
}

