//! The integrator module defines the Integrator trait implemented by
//! the various surface integrators used to render the scene with different
//! integration methods, eg. path tracing, photon mapping etc.

use std::num::Float;
use collect::enum_set::EnumSet;

use scene::Scene;
use linalg;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;
use bxdf::{BSDF, BxDFType};

pub use self::hit_marker::HitMarker;
pub use self::eyelight::EyeLight;
pub use self::whitted::Whitted;
pub use self::path::Path;

pub mod hit_marker;
pub mod eyelight;
pub mod whitted;
pub mod path;

/// Trait implemented by the various integration methods that can be used to render
/// the scene.
pub trait Integrator {
    /// Compute the illumination at the intersection in the scene
    /// TODO: Later we'll need to pass `&mut Sampler` through here as well
    /// for integrators that need randomness along with a memory pool once
    /// we implement that as well.
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection) -> Colorf;
    /// Compute the color of specularly reflecting light off the intersection
    fn specular_reflection(&self, scene: &Scene, ray: &Ray, bsdf: &BSDF) -> Colorf {
        let w_o = -ray.d;
        let mut spec_refl = EnumSet::new();
        spec_refl.insert(BxDFType::Specular);
        spec_refl.insert(BxDFType::Reflection);
        // TODO: Generate actual samples here
        let junk_samples = [0f32; 3];
        let (f, w_i, _, pdf) = bsdf.sample(&w_o, spec_refl, &junk_samples[]);
        let mut refl = Colorf::broadcast(0.0);
        if !f.is_black() && Float::abs(linalg::dot(&w_i, &bsdf.n)) != 0.0 {
            let mut refl_ray = ray.child(&bsdf.p, &w_i);
            refl_ray.min_t = 0.001;
            if let Some(hit) = scene.intersect(&mut refl_ray) {
                let li = self.illumination(scene, &refl_ray, &hit);
                refl = f * li * Float::abs(linalg::dot(&w_i, &bsdf.n))
            }
        }
        refl
    }
    /// Compute the color of specularly transmitted light through the intersection
    fn specular_transmission(&self, scene: &Scene, ray: &Ray, bsdf: &BSDF) -> Colorf {
        let w_o = -ray.d;
        let mut spec_trans = EnumSet::new();
        spec_trans.insert(BxDFType::Specular);
        spec_trans.insert(BxDFType::Transmission);
        // TODO: Generate actual samples here
        let junk_samples = [0f32; 3];
        let (f, w_i, _, pdf) = bsdf.sample(&w_o, spec_trans, &junk_samples[]);
        // TODO: include pdf val
        let mut transmit = Colorf::broadcast(0.0);
        if !f.is_black() && Float::abs(linalg::dot(&w_i, &bsdf.n)) != 0.0 {
            let mut trans_ray = ray.child(&bsdf.p, &w_i);
            trans_ray.min_t = 0.001;
            if let Some(hit) = scene.intersect(&mut trans_ray) {
                let li = self.illumination(scene, &trans_ray, &hit);
                transmit = f * li * Float::abs(linalg::dot(&w_i, &bsdf.n))
            }
        }
        transmit
    }
}

