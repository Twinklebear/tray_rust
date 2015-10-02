//! Provides an animated color value, so you can have colors change over time

use std::cmp::{Eq, Ord, PartialOrd, PartialEq, Ordering};
use std::collections::BTreeSet;

use linalg;
use film::Colorf;

/// ColorKeyframe is a color associated with a specific time
#[derive(Debug, Copy, Clone)]
pub struct ColorKeyframe {
    pub color: Colorf,
    pub time: f32,
}

impl ColorKeyframe {
    pub fn new(color: &Colorf, time: f32) -> ColorKeyframe {
        ColorKeyframe { color: *color, time: time }
    }
}
impl Ord for ColorKeyframe {
    fn cmp(&self, other: &ColorKeyframe) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for ColorKeyframe {
    fn partial_cmp(&self, other: &ColorKeyframe) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
impl Eq for ColorKeyframe {}
impl PartialEq for ColorKeyframe {
    fn eq(&self, other: &ColorKeyframe) -> bool {
        self.time == other.time
    }
}

/// AnimatedColor is a list of colors associated with time points in the scene
/// that will compute the color at the desired time by blending the two nearest ones
#[derive(Debug, Clone)]
pub struct AnimatedColor {
    /// List of color keyframes in time order
    keyframes: BTreeSet<ColorKeyframe>,
}

impl AnimatedColor {
    /// Create a new empty animated color
    pub fn new() -> AnimatedColor {
        AnimatedColor { keyframes: BTreeSet::new() }
    }
    /// Create an animated transform that will blend between the passed keyframes
    pub fn with_keyframes(keyframes: Vec<ColorKeyframe>) -> AnimatedColor {
        AnimatedColor { keyframes: keyframes.into_iter().collect() }
    }
    /// Compute the color at the desired time
    pub fn color(&self, time: f32) -> Colorf {
        if self.keyframes.is_empty() {
            Colorf::black()
        } else if self.keyframes.len() == 1 {
            self.keyframes.iter().next().unwrap().color
        } else {
            // TODO: Binary search here somehow? Or does the BTreeSet have some faster impl
            // of take/skip while?
            let first = self.keyframes.iter().take_while(|k| k.time < time).last();
            let second = self.keyframes.iter().skip_while(|k| k.time < time).next();
            if first.is_none() {
                self.keyframes.iter().next().unwrap().color
            } else if second.is_none() {
                self.keyframes.iter().last().unwrap().color
            } else {
                let mut color = Colorf::black();
                let f = first.unwrap().color;
                let s = second.unwrap().color;
                color.r = linalg::lerp(time, &f.r, &s.r);
                color.g = linalg::lerp(time, &f.g, &s.g);
                color.b = linalg::lerp(time, &f.b, &s.b);
                color.a = linalg::lerp(time, &f.a, &s.a);
                color
            }
        }
    }
}

