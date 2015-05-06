//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use std::sync::Arc;

use geometry::Boundable;
use linalg::Ray;

pub struct BVH<T> {
    /// Maximum amount of geometry we're willing to put in a leaf
    max_geom: usize,
    /// Reference to the vec containing the unordered objects in the BVH
    objects: Arc<Vec<T>>,
    /// Should this be a vec of references? or of indices?
    /// doing ordered_geom: Vec<&T> seems to be a lifetime annotation nightmare
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
    pub fn intersect<'a, F, R>(&'a self, ray: &mut Ray, f: F) -> Option<R>
            where F: Fn(&mut Ray, &'a T) -> Option<R> {
        let mut result = None;
		for o in &*self.objects {
			result = f(ray, o).or(result);
		}
        result
    }
}

