//! The MERL Material represents the surface's properties through data loaded from a
//! [MERL BRDF Database file](http://www.merl.com/brdf/). The material loads and parses
//! the data then hands it off to its BRDF which will be used to actually compute the
//! surface properties
//!
//! # Scene Usage Example
//! The MERL material requires a filepath to a BRDF table downloaded from the
//! [MERL BRDF Database](http://www.merl.com/brdf/).
//!
//! ```json
//! "materials": [
//!     {
//!         "name": "oxidized_steel",
//!         "type": "merl",
//!         "file": "./black-oxidized-steel.binary"
//!     },
//!     ...
//! ]
//! ```

use std::iter;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

use byteorder::{LittleEndian, ReadBytesExt};
use light_arena::Allocator;

use bxdf::{self, BSDF, BxDF};
use material::Material;
use geometry::Intersection;

/// Material that uses measured data to model the surface reflectance properties.
/// The measured data is from "A Data-Driven Reflectance Model",
/// by Wojciech Matusik, Hanspeter Pfister, Matt Brand and Leonard McMillan,
/// in ACM Transactions on Graphics 22, 3(2003), 759-769
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
    /// Create a new MERL BRDF by loading the refletance data from a MERL BRDF
    /// database file
    pub fn load_file(path: &Path) -> Merl {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                panic!("material::Merl::load_file - failed to open {:?} due to {}", path, e);
            },
        };
        let mut reader = BufReader::new(file);
        // Values we expect to read from a MERL BRDF file for each dimension
        let n_theta_h = 90;
        let n_theta_d = 90;
        let n_phi_d = 180;
        let dims = [reader.read_i32::<LittleEndian>().unwrap() as usize,
                    reader.read_i32::<LittleEndian>().unwrap() as usize,
                    reader.read_i32::<LittleEndian>().unwrap() as usize];
        if n_theta_h != dims[0] || n_theta_d != dims[1] || n_phi_d != dims[2] {
            panic!("material::Merl::load_file - Invalid MERL file header, aborting");
        }

        let n_vals = n_theta_h * n_theta_d * n_phi_d;
        let mut brdf = Vec::with_capacity(3 * n_vals);
        brdf.extend(iter::repeat(0.0).take(3 * n_vals));
        let scaling = [1.0 / 1500.0, 1.0 / 1500.0, 1.66 / 1500.0];
        // Read the n_vals corresponding to the red, green or blue component
        for (c, s) in scaling.iter().enumerate() {
            for i in 0..n_vals {
                // The BRDF data is stored in double precision with these odd scaling factors
                // so decode the value
                let x = (reader.read_f64::<LittleEndian>().unwrap() * s) as f32;
                brdf[3 * i + c] = f32::max(0.0, x);
            }
        }
        Merl { brdf: brdf, n_theta_h: n_theta_h, n_theta_d: n_theta_d, n_phi_d: n_phi_d }
    }
}

impl Material for Merl {
    fn bsdf<'a, 'b, 'c>(&'a self, hit: &Intersection<'a, 'b>,
                        alloc: &'c Allocator) -> BSDF<'c> where 'a: 'c {
        let bxdfs = alloc.alloc_slice::<&BxDF>(1);
        bxdfs[0] = alloc.alloc(bxdf::Merl::new(&self.brdf[..], self.n_theta_h, self.n_theta_d, self.n_phi_d));
        BSDF::new(bxdfs, 1.0, &hit.dg)
    }
}

