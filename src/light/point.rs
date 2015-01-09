//! Defines a simple point light with some position in the scene and desired light intensity.

use linalg;
use film::Colorf;
use light::{Light, OcclusionTester};

/// A standard point light. Has a position in the scene and light intensity. The point
/// light does have inverse square fall-off
#[derive(Copy, Show)]
pub struct Point {
    /// The position of the light
    pos: linalg::Point,
    /// The light intensity
    intensity: Colorf,
}

impl Point {
    /// Create a new point light at `pos` with some light intensity
    pub fn new(pos: &linalg::Point, intensity: &Colorf) -> Point {
        Point { pos: *pos, intensity: *intensity }
    }
}

impl Light for Point {
    fn sample_incident(&self, p: &linalg::Point, occlusion: &mut OcclusionTester) -> (Colorf, linalg::Vector) {
        let w_i = (self.pos - *p).normalized();
        *occlusion = OcclusionTester::test_points(p, &self.pos);
        (self.intensity / self.pos.distance_sqr(p), w_i)
    }
}

