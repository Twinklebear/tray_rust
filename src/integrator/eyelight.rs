//! Defines a simple eye light integrator used for debugging

use std::num::Float;

use scene::Scene;
use linalg;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;
use bxdf::BxDFType;

/// The EyeLight integrator illuminates all objects in the scene from the camera
/// and only accounts for direct lighting. Rays from the camera to objects bring
/// light back from the objects along the same direction towards the camera
#[derive(Copy)]
pub struct EyeLight {
    /// Strength of the eye light
    intensity: Colorf,
}

impl EyeLight {
    /// Create a new eye light integrator and set the intensity of the light at the
    /// camera
    pub fn new(intensity: &Colorf) -> EyeLight { EyeLight { intensity: *intensity } }
}

impl Integrator for EyeLight {
    fn illumination(&self, _: &Scene, ray: &Ray, hit: &Intersection) -> Colorf {
        let bsdf = hit.instance.material.bsdf(hit);
        let w_o = -ray.d;
        self.intensity * bsdf.eval(&w_o, &w_o, BxDFType::all())
            * Float::abs(linalg::dot(&ray.d, &bsdf.n))
    }
}

