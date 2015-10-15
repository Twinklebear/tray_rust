//! Provides a keyframe transformation which is a transform associated
//! with a specific point in time

use std::f32;
use std::cmp::{Eq, Ord, PartialOrd, PartialEq, Ordering};

use bspline;

use linalg::{self, quaternion, Vector, Matrix4, Quaternion, Transform};

/// A transformation associated with a specific point in time
#[derive(Debug, Copy, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub translation: Vector,
    pub rotation: Quaternion,
    pub scaling: Matrix4,
}

impl Keyframe {
    /// Construct a new keyframe transformation by associating the
    /// transform passed with the time point `time`. The transform will
    /// be stored in a decomposed form, M = TRS.
    pub fn new(transform: &Transform, time: f32) -> Keyframe {
        assert!(f32::is_finite(time));
        let (t, r, s) = Keyframe::decompose(transform);
        Keyframe { time: time, translation: t, rotation: r, scaling: s }
    }
    /// Construct the keyframe from the decomposed transformation
    pub fn from_parts(translation: &Vector, rotation: &Quaternion, scaling: &Matrix4, time: f32) -> Keyframe {
        Keyframe { time: time, translation: *translation, rotation: *rotation, scaling: *scaling }
    }
    /// Decompose the transformation into its component translation, rotation and
    /// scaling operations.
    fn decompose(transform: &Transform) -> (Vector, Quaternion, Matrix4) {
        let mut m = transform.mat;
        // Extract the translation component and remove it from the matrix
        let translation = Vector::new(*m.at(0, 3), *m.at(1, 3), *m.at(2, 3));
        for i in 0..3 {
            *m.at_mut(i, 3) = 0.0;
            *m.at_mut(3, i) = 0.0;
        }
        *m.at_mut(3, 3) = 1.0;
        // Extract rotation component using polar decomposition by computing
        // M_{i + 1} = 1/2 (M_i + (M_i^T)^-1) to convergence
        let mut rot_mat = m;
        for _ in 0..100 {
            let m_inv_trans = rot_mat.transpose().inverse();
            let r_next: Matrix4 = rot_mat.iter().zip(m_inv_trans.iter())
                .map(|(&a, &b)| 0.5 * (a + b)).collect();
            let mut norm = 0.0;
            for i in 0..3 {
                let n = f32::abs(*rot_mat.at(i, 0) - *r_next.at(i, 0))
                    + f32::abs(*rot_mat.at(i, 1) - *r_next.at(i, 1))
                    + f32::abs(*rot_mat.at(i, 2) - *r_next.at(i, 2));
                norm = f32::max(norm, n);
            }
            rot_mat = r_next;
            if norm <= 0.0001 {
                break;
            }
        }
        (translation, Quaternion::from_matrix(&rot_mat), rot_mat.inverse() * m)
    }
    /// Return the transformation stored for this keyframe
    pub fn transform(&self) -> Transform {
        let m = self.rotation.to_matrix() * self.scaling;
        Transform::translate(&self.translation) * Transform::from_mat(&m)
    }
}

impl bspline::Interpolate for Keyframe {
    fn interpolate(&self, other: &Keyframe, t: f32) -> Keyframe {
        let translation = (1.0 - t) * self.translation + t * other.translation;
        let rotation = quaternion::slerp(t, &self.rotation, &other.rotation);
        let scaling: Matrix4 = self.scaling.iter().zip(other.scaling.iter())
            .map(|(a, b)| linalg::lerp(t, a, b)).collect();
        Keyframe::from_parts(&translation, &rotation, &scaling, t)
    }
}

/// Interpolate between the two keyframes at some time. If time is before
/// or after the interval spanned by the keyframes then the nearest one is
/// returned. Otherwise the transforms are interpolated and this new
/// transform is returned.
pub fn interpolate(time: f32, a: &Keyframe, b: &Keyframe) -> Transform {
    // Order the transformations by their time points
    let (first, second) = if a.time < b.time { (a, b) } else { (b, a) };
    if time <= first.time {
        let m = first.rotation.to_matrix() * first.scaling;
        Transform::translate(&first.translation) * Transform::from_mat(&m)
    } else if time >= second.time {
        let m = second.rotation.to_matrix() * second.scaling;
        Transform::translate(&second.translation) * Transform::from_mat(&m)
    } else {
        let dt = (time - first.time) / (second.time - first.time);
        let translation = (1.0 - dt) * first.translation + dt * second.translation;
        let rotation = quaternion::slerp(dt, &first.rotation, &second.rotation);
        let scaling: Matrix4 = first.scaling.iter().zip(second.scaling.iter())
                .map(|(a, b)| linalg::lerp(dt, a, b)).collect();
        let m = rotation.to_matrix() * scaling;
        Transform::translate(&translation) * Transform::from_mat(&m)
    }
}

impl Ord for Keyframe {
    fn cmp(&self, other: &Keyframe) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Keyframe {
    fn partial_cmp(&self, other: &Keyframe) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Eq for Keyframe {}

impl PartialEq for Keyframe {
    fn eq(&self, other: &Keyframe) -> bool {
        self.time == other.time
    }
}

