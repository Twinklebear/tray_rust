//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use geometry::Boundable;
use linalg::Ray;

pub struct BVH<T> {
    /// Maximum amount of geometry we're willing to put in a leaf
    max_geom: usize,
    /// The geometry stored in this BVH, this will be re-ordered to
    /// fit the BVH construction layout. TODO: We may want to make
    /// the geometry accessible by index
    geometry: Vec<T>,
}

impl<T: Boundable> BVH<T> {
    /// Create a new BVH using a SAH construction algorithm
    pub fn new(max_geom: usize, geometry: Vec<T>) -> BVH<T> {
        BVH { max_geom: max_geom, geometry: geometry }
    }
    /// Traverse the BVH and call the function passed on the objects in the leaf nodes
    /// of the BVH, returning the value returned by the function after traversal completes
    pub fn intersect<'a, F, R>(&'a self, ray: &mut Ray, f: F) -> Option<R>
            where F: Fn(&mut Ray, &'a T) -> Option<R> {
        let mut result = None;
		for o in &*self.geometry {
			result = f(ray, o).or(result);
		}
        result
    }
}

