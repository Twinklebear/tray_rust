//! An emitter is an instance of geometry that both receives and emits light
//!
//! # Scene Usage Example
//! An emitter is an object in the scene that emits light, it can be a point light
//! or an area light. The emitter takes an extra 'emitter' parameter to specify
//! whether the instance is an area or point emitter and an 'emission' parameter
//! to set the color and strength of emitted light.
//!
//! ## Point Light Example
//! The point light has no geometry, material or transformation since it's not a
//! physical object. Instead it simply takes a position to place the light at in the scene.
//!
//! ```json
//! "objects": [
//!     {
//!         "name": "my_light",
//!         "type": "emitter",
//!         "emitter": "point",
//!         "position": [0.0, 0.0, 22.0]
//!         "emission": [1, 1, 1, 100]
//!     },
//!     ...
//! ]
//! ```
//!
//! ## Area Light Example
//! The area light looks similar to a regular receiver except it has an additional emission
//! parameter. Area lights are also restricted somewhat in which geometry they can use as
//! it needs to be possible to sample the geometry. Area lights can only accept geometry
//! that implements `geometry::Sampleable`.
//!
//! ```json
//! "objects": [
//!     {
//!         "name": "my_area_light",
//!         "type": "emitter",
//!         "emitter": "area",
//!         "emission": [1, 1, 1, 100],
//!         "material": "white_matte",
//!         "geometry": {
//!             "type": "sphere",
//!              "radius": 2.5
//!         },
//!         "transform": [
//!             {
//!                 "type": "translate",
//!                 "translation": [0, 0, 22]
//!             }
//!         ]
//!     },
//!     ...
//! ]
//! ```

use std::sync::Arc;
use geometry::{Boundable, BBox, SampleableGeom, DifferentialGeometry};
use material::Material;
use linalg::{self, Transform, AnimatedTransform, Keyframe, Point, Ray, Vector, Normal};
use film::{AnimatedColor, Colorf};
use light::{Light, OcclusionTester};

/// The type of emitter, either a point light or an area light
/// in which case the emitter has associated geometry and a material
/// TODO: Am I happy with this design?
enum EmitterType {
    Point,
    /// The area light holds the geometry that is emitting the light
    /// and the material for the geometry
    Area(Arc<SampleableGeom + Send + Sync>, Arc<Material + Send + Sync>),
}

/// An instance of geometry in the scene that receives and emits light.
pub struct Emitter {
    emitter: EmitterType,
    /// The light intensity emitted
    pub emission: AnimatedColor,
    /// The transform to world space
    transform: AnimatedTransform,
    /// Tag to identify the instance
    pub tag: String,
}

impl Emitter {
    /// Create a new area light using the geometry passed to emit light
    /// TODO: We need sample methods for geometry to do this
    /// We also need MIS in the path tracer's direct light sampling so we get
    /// good quality
    pub fn area(geom: Arc<SampleableGeom + Send + Sync>, material: Arc<Material + Send + Sync>,
                emission: AnimatedColor, transform: AnimatedTransform, tag: String) -> Emitter {
        // TODO: How to change this transform to handle scaling within the animation?
        /*
        if transform.has_scale() {
            println!("Warning: scaling detected in area light transform, this may give incorrect results");
        }
        */
        Emitter { emitter: EmitterType::Area(geom, material),
                  emission: emission,
                  transform: transform,
                  tag: tag.to_string() }
    }
    /// Create a new point light. TODO: Should we just take a transform here as well?
    pub fn point(pos: Point, emission: AnimatedColor, tag: String) -> Emitter {
        Emitter { emitter: EmitterType::Point,
                  emission: emission,
                  transform: AnimatedTransform::with_keyframes(vec![Keyframe::new(&Transform::translate(&(pos - Point::broadcast(0.0))), 0.0)]),
                  tag: tag.to_string() }
    }
    /// Test the ray for intersection against this insance of geometry.
    /// returns Some(Intersection) if an intersection was found and None if not.
    /// If an intersection is found `ray.max_t` will be set accordingly
    pub fn intersect(&self, ray: &mut Ray) -> Option<(DifferentialGeometry, &Material)> {
        match &self.emitter {
            &EmitterType::Point => None,
            &EmitterType::Area(ref geom, ref mat) => {
                let transform = self.transform.transform(ray.time);
                let mut local = transform.inv_mul_ray(ray);
                let mut dg = match geom.intersect(&mut local) {
                    Some(dg) => dg,
                    None => return None,
                };
                ray.max_t = local.max_t;
                dg.p = transform * dg.p;
                dg.n = transform * dg.n;
                dg.ng = transform * dg.ng;
                dg.dp_du = transform * dg.dp_du;
                dg.dp_dv = transform * dg.dp_dv;
                Some((dg, &**mat))
            },
        }
    }
    /// Return the radiance emitted by the light in the direction `w`
    /// from point `p` on the light's surface with normal `n`
    pub fn radiance(&self, w: &Vector, _: &Point, n: &Normal, time: f32) -> Colorf {
        if linalg::dot(w, n) > 0.0 { self.emission.color(time) } else { Colorf::black() }
    }
    /// Get the transform to place the emitter into world space
    pub fn get_transform(&self) -> &AnimatedTransform {
        &self.transform
    }
    /// Set the transform to place the emitter into world space
    pub fn set_transform(&mut self, transform: AnimatedTransform) {
        self.transform = transform;
    }
}

impl Boundable for Emitter {
    fn bounds(&self, start: f32, end: f32) -> BBox {
        match &self.emitter {
            &EmitterType::Point => self.transform.animation_bounds(&BBox::singular(Point::broadcast(0.0)), start, end),
            &EmitterType::Area(ref g, _) => {
                self.transform.animation_bounds(&g.bounds(start, end), start, end)
            },
        }
    }
}

impl Light for Emitter {
    fn sample_incident(&self, p: &Point, samples: &(f32, f32), time: f32)
        -> (Colorf, Vector, f32, OcclusionTester)
    {
        match &self.emitter {
            &EmitterType::Point => {
                let transform = self.transform.transform(time);
                let pos = transform * Point::broadcast(0.0);
                let w_i = (pos - *p).normalized();
                (self.emission.color(time) / pos.distance_sqr(p), w_i, 1.0, OcclusionTester::test_points(p, &pos, time))
            }
            &EmitterType::Area(ref g, _) => {
                let transform = self.transform.transform(time);
                let p_l = transform.inv_mul_point(p);
                let (p_sampled, normal) = g.sample(&p_l, samples);
                let w_il = (p_sampled - p_l).normalized();
                let pdf = g.pdf(&p_l, &w_il);
                let radiance = self.radiance(&-w_il, &p_sampled, &normal, time);
                let p_w = transform * p_sampled;
                (radiance, transform * w_il, pdf, OcclusionTester::test_points(&p, &p_w, time))
            },
        }
    }
    fn delta_light(&self) -> bool {
        match &self.emitter { 
            &EmitterType::Point => true,
            _ => false,
        }
    }
    fn pdf(&self, p: &Point, w_i: &Vector, time: f32) -> f32 {
        match &self.emitter {
            &EmitterType::Point => 0.0,
            &EmitterType::Area(ref g, _ ) => {
                let transform = self.transform.transform(time);
                let p_l = transform.inv_mul_point(p);
                let w = (transform.inv_mul_vector(w_i)).normalized();
                g.pdf(&p_l, &w)
            }
        }
    }
}

