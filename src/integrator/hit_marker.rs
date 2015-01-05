//! Defines the HitMarker integrator which colors all hits white to simply
//! mark all hits in the scene

use scene::Scene;
use linalg::Ray;
use geometry::Intersection;
use film::Colorf;
use integrator::Integrator;

/// The HitMarker returns white for all illumination. The end result of this is
/// that any objects in the scene will be shaded a flat white while areas without
/// objects will remain black.
#[derive(Copy)]
pub struct HitMarker;

impl Integrator for HitMarker {
    fn illumination(&self, _: &Scene, _: &Ray, _: &Intersection) -> Colorf {
        Colorf::broadcast(1.0)
    }
}

