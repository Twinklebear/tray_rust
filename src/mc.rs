//! Defines various Monte Carlo sampling functions for sampling
//! points/directions on objects and computing the corresponding pdfs

use std::f32;

use linalg::{self, Vector};

/// Sample a hemisphere using a cosine distribution to produce cosine weighted samples
/// `samples` should be two random samples in range [0, 1)
/// directions returned will be in the hemisphere around (0, 0, 1)
pub fn cos_sample_hemisphere(u: &(f32, f32)) -> Vector {
    //We use Malley's method here, generate samples on a disk then project
    //them up to the hemisphere
    let d = concentric_sample_disk(u);
    Vector::new(d.0, d.1, f32::sqrt(f32::max(0.0, 1.0 - d.0 * d.0 - d.1 * d.1)))
}
/// Compute the PDF of the cosine weighted hemisphere sampling
pub fn cos_hemisphere_pdf(cos_theta: f32) -> f32 { cos_theta * f32::consts::FRAC_1_PI }
/// Compute concentric sample positions on a unit disk mapping input from range [0, 1)
/// to sample positions on a disk
/// `samples` should be two random samples in range [0, 1)
/// See: [Shirley and Chiu, A Low Distortion Map Between Disk and Square](https://mediatech.aalto.fi/~jaakko/T111-5310/K2013/JGT-97.pdf)
pub fn concentric_sample_disk(u: &(f32, f32)) -> (f32, f32) {
    let s = (2.0 * u.0 - 1.0, 2.0 * u.1 - 1.0);
    let radius;
    let theta;
    if s.0 == 0.0 && s.1 == 0.0 {
        return s;
    }
    if s.0 >= -s.1 {
        if s.0 > s.1 {
            radius = s.0;
            if s.1 > 0.0 {
                theta = s.1 / s.0;
            } else {
                theta = 8.0 + s.1 / s.0;
            }
        }
        else {
            radius = s.1;
            theta = 2.0 - s.0 / s.1;
        }
    }
    else {
        if s.0 <= s.1 {
            radius = -s.0;
            theta = 4.0 + s.1 / s.0;
        }
        else {
            radius = -s.1;
            theta = 6.0 - s.0 / s.1;
        }
    }
    let theta = theta * f32::consts::FRAC_PI_4;
    (radius * f32::cos(theta), radius * f32::sin(theta))
}
/// Power heuristic for multiple importance sampling for two functions being sampled, f & g
/// where beta is hard-coded to be two following PBR & Veach
/// - `n_f`, `n_g` number of samples taken of each
/// - `pdf_f`, `pdf_g` pdf of each function
pub fn power_heuristic(n_f: f32, pdf_f: f32, n_g: f32, pdf_g: f32) -> f32 {
    let f = n_f * pdf_f;
    let g = n_g * pdf_g;
    (f * f) / (f * f + g * g)
}
/// Return the PDF for uniformly sampling a cone with some max solid angle
pub fn uniform_cone_pdf(cos_theta: f32) -> f32 {
    1.0 / (f32::consts::PI * 2.0 * (1.0 - cos_theta))
}
/// Uniformly sample a direction in a cone with max angle `cos_theta_max` where
/// the cone lies along the z-axis
pub fn uniform_sample_cone(samples: &(f32, f32), cos_theta_max: f32) -> Vector {
    let cos_theta = linalg::lerp(samples.0, &cos_theta_max, &1.0);
    let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
    let phi = samples.1 * f32::consts::PI * 2.0;
    Vector::new(f32::cos(phi) * sin_theta, f32::sin(phi) * sin_theta, cos_theta)
}
/// Uniformly sample a direction on the unit sphere about the origin
pub fn uniform_sample_sphere(samples: &(f32, f32)) -> Vector {
    let z = 1.0 - 2.0 * samples.0;
    let r = f32::sqrt(f32::max(0.0, 1.0 - z * z));
    let phi = f32::consts::PI * 2.0 * samples.1;
    Vector::new(f32::cos(phi) * r, f32::sin(phi) * r, z)
}

