//! Provides an animated transformation that moves an object between a
//! set of specified keyframes.

use std::ops::Mul;

use bspline::BSpline;

use linalg::{self, quaternion, Keyframe, Transform};
use geometry::BBox;

/// An animated transform that blends between the keyframes in its transformation
/// list over time.
#[derive(Clone, Debug)]
pub struct AnimatedTransform {
    /// List of animated transforms in hierarchical order, e.g. the lowest
    /// index is the object's, index 1 holds its direct parent's transform, etc.
    keyframes: Vec<BSpline<Keyframe>>,
}

impl AnimatedTransform {
    /// Create an animated transformation blending between the passed keyframes
    pub fn with_keyframes(mut keyframes: Vec<Keyframe>, knots: Vec<f32>, degree: usize) -> AnimatedTransform {
        // so we know what degree and so on.
        // Step through and make sure all rotations take the shortest path
        for i in 1..keyframes.len() {
            // If the dot product is negative flip the current quaternion to
            // take the shortest path through the rotation
            if quaternion::dot(&keyframes[i - 1].rotation, &keyframes[i].rotation) < 0.0 {
                keyframes[i].rotation = -keyframes[i].rotation;
            }
        }
        AnimatedTransform { keyframes: vec![BSpline::new(degree, keyframes, knots)] }
    }
    pub fn unanimated(transform: &Transform) -> AnimatedTransform {
        let key = Keyframe::new(&transform);
        AnimatedTransform { keyframes: vec![BSpline::new(0, vec![key], vec![0.0, 1.0])] }
    }
    /// Compute the transformation matrix for the animation at some time point using B-Spline
    /// interpolation.
    pub fn transform(&self, time: f32) -> Transform {
        let mut transform = Transform::identity();
        // Step through the transform stack, applying each animation transform at this
        // time as we move up
        for spline in self.keyframes.iter() {
            let domain = spline.knot_domain();
            let t =
                if spline.control_points().count() == 1 {
                    spline.control_points().next().unwrap().transform()
                } else if time < domain.0 {
                    spline.point(domain.0).transform()
                } else if time > domain.1 {
                    spline.point(domain.1).transform()
                } else {
                    spline.point(time).transform()
                };
            transform = t * transform;
        }
        transform
    }
    /// Compute the bounds of the box moving through the animation sequence by sampling time
    pub fn animation_bounds(&self, b: &BBox, start: f32, end: f32) -> BBox {
        if !self.is_animated() {
            let t = self.transform(start);
            t * *b
        } else {
            let mut ret = BBox::new();
            for i in 0..128 {
                let time = linalg::lerp((i as f32) / 127.0, &start, &end);
                let t = self.transform(time);
                ret = ret.box_union(&(t * *b));
            }
            ret
        }
    }
    /// Check if the transform is actually animated
    pub fn is_animated(&self) -> bool {
        self.keyframes.is_empty() || self.keyframes.iter().fold(true, |b, spline| b && spline.control_points().count() > 1)
    }
}

impl Mul for AnimatedTransform {
    type Output = AnimatedTransform;
    /// Compose the animated transformations
    fn mul(self, mut rhs: AnimatedTransform) -> AnimatedTransform {
        for l in &self.keyframes[..] {
            rhs.keyframes.push(l.clone());
        }
        rhs
    }
}

