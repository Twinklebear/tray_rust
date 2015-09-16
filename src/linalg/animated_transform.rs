//! Provides an animated transformation that moves an object between a
//! set of specified keyframes.

use std::collections::BTreeSet;

use linalg::{keyframe, Keyframe, Transform};

/// An animated transform that blends between the keyframes in its transformation
/// list over time.
#[derive(Debug, Clone)]
pub struct AnimatedTransform {
    // TODO: Need to handle composing animated transforms. This basically
    // will be done by keeping a stack of these lists and working our way up/down
    // it when we want to compute the transform at some time.
    keyframes: BTreeSet<Keyframe>,
}

impl AnimatedTransform {
    /// Create a new empty animated transform
    pub fn new() -> AnimatedTransform {
        AnimatedTransform { keyframes: BTreeSet::new() }
    }
    /// Create an animated transformation blending between the passed keyframes
    pub fn with_keyframes(keyframes: Vec<Keyframe>) -> AnimatedTransform {
        AnimatedTransform { keyframes: keyframes.into_iter().collect() }
    }
    /// Insert a keyframe into the animation sequence
    pub fn insert(&mut self, keyframe: Keyframe) {
        self.keyframes.insert(keyframe);
    }
    /// Compute the transformation matrix for the animation at some time point.
    /// The transform is found by interpolating the two keyframes nearest to the
    /// time point being evaluated. **TODO** a binary search of some kind to find
    /// the two keyframes to blend would be much better.
    pub fn transform(&self, time: f32) -> Transform {
        if self.keyframes.is_empty() {
            Transform::identity()
        } else {
            let first = self.keyframes.iter().take_while(|k| k.time < time).last();
            let second = self.keyframes.iter().skip_while(|k| k.time < time).next();
            if first.is_none() {
                self.keyframes.iter().next().unwrap().transform()
            } else if second.is_none() {
                self.keyframes.iter().last().unwrap().transform()
            } else {
                keyframe::interpolate(time, first.unwrap(), second.unwrap())
            }
        }
    }
}

