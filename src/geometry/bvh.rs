//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use geometry::Boundable;

pub struct BVH<'a, T: 'a> {
    // Maximum amount of geometry we're willing to put in a leaf
    max_geom: usize,
    objects: Vec<&'a T>,
}

impl<'a, T: Boundable> BVH<'a, T> {
    pub fn new(max_geom: usize, objects: Vec<&'a T>) -> BVH<'a, T> {
        BVH { max_geom: max_geom, objects: objects }
    }
}

