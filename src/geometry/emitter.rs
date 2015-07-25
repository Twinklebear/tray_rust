//! An emitter is an instance of geometry that both receives and emits light

use std::sync::Arc;
use geometry::{Boundable, BBox, BoundableGeom, DifferentialGeometry};
use material::Material;
use linalg::{Transform, Point, Ray, Vector};
use film::Colorf;
use light::{Light, OcclusionTester};

/// The type of emitter, either a point light or an area light
/// in which case the emitter has associated geometry and a material
/// TODO: Am I happy with this design?
enum EmitterType {
    Point,
    /// The area light holds the geometry that is emitting the light
    /// and the material for the geometry
    Area(Arc<BoundableGeom + Send + Sync>, Arc<Material + Send + Sync>),
}

/// An instance of geometry in the scene that receives and emits light
/// TODO: This is currently just a placeholder, emissive geometry isn't
/// currently implemented. This is why it's identical to `Receiver` :P
pub struct Emitter {
    emitter: EmitterType,
    /// The light intensity emitted
    pub emission: Colorf,
    /// The transform to world space
    transform: Transform,
    /// Tag to identify the instance
    pub tag: String,
}

// TODO: It may look like we repeat a lot of code here but that won't be the case after I
// actually implement the emitter and unify point lights within this design.
impl Emitter {
    /// Create a new area light using the geometry passed to emit light
    /// TODO: We need sample methods for geometry to do this
    /// We also need MIS in the path tracer's direct light sampling so we get
    /// good quality
    /*
    pub fn area(geom: Arc<BoundableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
               transform: linalg::Transform, tag: &str) -> Emitter {
        Emitter { geom: geom, material: material, transform: transform, tag: tag.to_string() }
    }
    */
    pub fn point(pos: Point, emission: Colorf, tag: &str) -> Emitter {
        Emitter { emitter: EmitterType::Point,
                  emission: emission,
                  transform: Transform::translate(&(pos - Point::broadcast(0.0))),
                  tag: tag.to_string() }
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut Ray) -> Option<(DifferentialGeometry, &Material)> {
        match &self.emitter {
            &EmitterType::Point => None,
            &EmitterType::Area(ref geom, ref mat) => {
                let mut local = self.transform.inv_mul_ray(ray);
                let mut dg = match geom.intersect(&mut local) {
                    Some(dg) => dg,
                    None => return None,
                };
                ray.max_t = local.max_t;
                dg.p = self.transform * dg.p;
                dg.n = self.transform * dg.n;
                dg.ng = self.transform * dg.ng;
                dg.dp_du = self.transform * dg.dp_du;
                dg.dp_dv = self.transform * dg.dp_dv;
                Some((dg, &**mat))
            },
        }
    }
}

impl Boundable for Emitter {
    fn bounds(&self) -> BBox {
        match &self.emitter {
            &EmitterType::Point => BBox::new(),
            &EmitterType::Area(ref g, _) => {
                self.transform * g.bounds()
            },
        }
    }
}

impl Light for Emitter {
    fn sample_incident(&self, p: &Point, samples: &(f32, f32))
        -> (Colorf, Vector, f32, OcclusionTester)
    {
        match &self.emitter {
            &EmitterType::Point => {
                let pos = self.transform * Point::broadcast(0.0);
                let w_i = (pos - *p).normalized();
                (self.emission / pos.distance_sqr(p), w_i, 1.0, OcclusionTester::test_points(p, &pos))
            }
            _ => {
                (Colorf::black(), Vector::broadcast(0.0), 1.0,
                OcclusionTester::test_points(&Point::broadcast(0.0), &Point::broadcast(0.0)))
            },
        }
    }
    fn delta_light(&self) -> bool {
        match &self.emitter { 
            &EmitterType::Point => true,
            _ => false,
        }
    }
}

