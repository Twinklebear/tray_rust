//! The integrator module defines the Integrator trait implemented by
//! the various surface integrators used to render the scene with different
//! integration methods, eg. path tracing, photon mapping etc.

use scene::Scene;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;

pub use self::hit_marker::HitMarker;
pub use self::eyelight::EyeLight;
pub use self::whitted::Whitted;

pub mod hit_marker;
pub mod eyelight;
pub mod whitted;

/// Trait implemented by the various integration methods that can be used to render
/// the scene.
pub trait Integrator {
    /// Compute the illumination at the intersection in the scene
    /// TODO: Later we'll need to pass `&mut Sampler` through here as well
    /// for integrators that need randomness along with a memory pool once
    /// we implement that as well.
    fn illumination(&self, scene: &Scene, ray: &Ray, hit: &Intersection) -> Colorf;
}

