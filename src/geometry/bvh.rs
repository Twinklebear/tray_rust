//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use std::sync::Arc;
use std::ops::FnMut;

use geometry::Boundable;
use linalg::Ray;

pub struct BVH<T> {
    // Maximum amount of geometry we're willing to put in a leaf
    max_geom: usize,
    objects: Arc<Vec<T>>,
}

impl<T: Boundable> BVH<T> {
    pub fn new(max_geom: usize, objects: Arc<Vec<T>>) -> BVH<T> {
        BVH { max_geom: max_geom, objects: objects }
    }
    /// Traverse the BVH and call the function passed on the objects in the leaf nodes
    /// of the BVH, returning the value returned by the function after traversal
    /// completes.
    /// TODO: I've tried to base this interface on that of `Iterator::map` however they don't seem
    /// to take `f` as a mutable parameter?
    pub fn intersect<F, R>(&self, ray: &mut Ray, mut f: F) -> Option<R>
            where F: FnMut(&mut Ray, &T) -> Option<R> {
        let mut result = None;
        result = f(ray, &self.objects[0]).or(result);
        result
    }
}

