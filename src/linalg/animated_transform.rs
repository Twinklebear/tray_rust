//! Provides an animated transformation that moves an object between a
//! set of specified keyframes.

use std::collections::BTreeSet;
use std::ops::Mul;

use linalg::{keyframe, Keyframe, Transform};

/// An animated transform that blends between the keyframes in its transformation
/// list over time.
#[derive(Debug, Clone)]
pub struct AnimatedTransform {
    /// List of animated transforms in hierarchical order, e.g. the lowest
    /// index is the object's, index 1 holds its direct parent's transform, etc.
    keyframes: Vec<BTreeSet<Keyframe>>,
}

impl AnimatedTransform {
    /// Create a new empty animated transform
    pub fn new() -> AnimatedTransform {
        AnimatedTransform { keyframes: Vec::new() }
    }
    /// Create an animated transformation blending between the passed keyframes
    pub fn with_keyframes(keyframes: Vec<Keyframe>) -> AnimatedTransform {
        AnimatedTransform { keyframes: vec![keyframes.into_iter().collect()] }
    }
    /// Insert a keyframe into the animation sequence
    pub fn insert(&mut self, keyframe: Keyframe) {
        self.keyframes[0].insert(keyframe);
    }
    /// Compute the transformation matrix for the animation at some time point.
    /// The transform is found by interpolating the two keyframes nearest to the
    /// time point being evaluated. **TODO** a binary search of some kind to find
    /// the two keyframes to blend would be much better.
    pub fn transform(&self, time: f32) -> Transform {
        let mut transform = Transform::identity();
        // Step through the transform stack, applying each animation transform at this
        // time as we move up
        for stack in &self.keyframes[..] {
            let t = if stack.len() == 1 {
                let first = stack.iter().next().unwrap();
                first.transform()
            } else {
                // TODO: Binary search here somehow? Or does the BTreeSet have some faster impl
                // of take/skip while?
                let first = stack.iter().take_while(|k| k.time < time).last();
                let second = stack.iter().skip_while(|k| k.time < time).next();
                if first.is_none() {
                    stack.iter().next().unwrap().transform()
                } else if second.is_none() {
                    stack.iter().last().unwrap().transform()
                } else {
                    keyframe::interpolate(time, first.unwrap(), second.unwrap())
                }
            };
            transform = t * transform;
        }
        transform
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

