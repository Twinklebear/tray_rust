//! Defines the `NormalsDebug` integrator which renders out the object's normals
//!
//! # Scene Usage Example
//! The `NormalsDebug` integrator just needs a maximum ray depth to terminate specular reflection
//! and transmission rays.
//!
//! ```json
//! "integrator": {
//!     "type": "normals_debug"
//! }
//! ```

use rand::StdRng;
use light_arena::Allocator;

use scene::Scene;
use linalg::Ray;
use geometry::{Intersection, Emitter};
use film::Colorf;
use integrator::Integrator;
use sampler::Sampler;

/// The `NormalsDebug` integrator implementing the `NormalsDebug` recursive ray tracing algorithm
#[derive(Clone, Copy, Debug)]
pub struct NormalsDebug;

impl Integrator for NormalsDebug {
    fn illumination(&self, _: &Scene, _: &[&Emitter], _: &Ray,
                    hit: &Intersection, _: &mut Sampler, _: &mut StdRng,
                    alloc: &Allocator) -> Colorf {
        let bsdf = hit.material.bsdf(hit, alloc);
        (Colorf::new(bsdf.n.x, bsdf.n.y, bsdf.n.z) + Colorf::broadcast(1.0)) / 2.0
    }
}


