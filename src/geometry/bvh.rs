//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use std::mem;
use std::f32;

use geometry::{BBox, Boundable};
use linalg::{Point, Ray, Axis};

/// A standard BVH2 that stores objects that can report their bounds in some space
/// via the `Boundable` trait. The BVH is constructed using a SAH partitioning scheme
pub struct BVH<T> {
    /// Maximum amount of geometry we're willing to put in a leaf
    max_geom: usize,
    /// The geometry stored in this BVH, this will be re-ordered to
    /// fit the BVH construction layout. TODO: We may want to make
    /// the geometry accessible by index
    geometry: Vec<T>,
}

/// Information about the location and bounds of some geometry 
struct GeomInfo<'a, T: 'a> {
    geom: &'a T,
    geom_idx: usize,
    center: Point,
    bounds: BBox,
}

impl<'a, T: Boundable> GeomInfo<'a, T> {
    /// Create a new reference to some geometry
    fn new(geom: &'a T, geom_idx: usize) -> GeomInfo<T> {
        let bounds = geom.bounds();
        GeomInfo { geom: geom, geom_idx: geom_idx,
                   center: bounds.lerp(0.5, 0.5, 0.5),
                   bounds: bounds }
    }
}

/// Data needed by a build node during construction
enum BuildNodeData {
    /// Interior node of a BVH, stores two child nodes
    Interior {
        /// Left and Right children of the node
        children: [Box<BuildNode>; 2],
        /// Axis that geomtry was partitioned along to split into
        /// the child nodes
        split_axis: Axis,
    },
    /// Leaf node of a BVH, stores geometry
    Leaf {
        /// Number of objects stored in this node
        ngeom: usize,
        /// Offset into the array holding the sorted geometry
        geom_offset: usize,
    },
}

/// Temporary datastructure for constructing the BVH into a tree before
/// flattening it down into a Vec for performance
struct BuildNode {
    /// The data stored at this node, either information about an Interior
    /// or Lead node
    node: BuildNodeData,
    /// Bounding box of this node
    bounds: BBox,
}

impl BuildNode {
    fn interior(children: [Box<BuildNode>; 2], split_axis: Axis) -> BuildNode {
        let bounds = children[0].bounds.box_union(&children[1].bounds);
        BuildNode { node: BuildNodeData::Interior { children: children, split_axis: split_axis },
                    bounds: bounds }
    }
    fn leaf(ngeom: usize, geom_offset: usize, bounds: BBox) -> BuildNode {
        BuildNode { node: BuildNodeData::Leaf { ngeom: ngeom, geom_offset: geom_offset }, bounds: bounds }
    }
}

#[derive(Copy, Clone)]
struct SAHBucket {
    count: usize,
    bounds: BBox,
}

impl SAHBucket {
    /// Return a SAHBucket with no items and degenerate bounds
    fn new() -> SAHBucket {
        SAHBucket { count: 0, bounds: BBox::new() }
    }
}

impl<T: Boundable> BVH<T> {
    /// Create a new BVH using a SAH construction algorithm
    pub fn new(max_geom: usize, mut geometry: Vec<T>) -> BVH<T> {
        assert!(!geometry.is_empty());
        {
            let mut build_geom = Vec::with_capacity(geometry.len());
            for (i, g) in geometry.iter().enumerate() {
                build_geom.push(GeomInfo::new(g, i));
            }
            // TODO: How to sort the geometry into the flatten tree ordering?
            // we have the indices things should end up in stored in ordered geom
            // but how to use this information in sort_by for example?
            // Should we move things into/out of build_geom instead of borrowing?
            // it knows the index of the items
            let mut total_nodes = 0;
            let mut ordered_geom = Vec::with_capacity(geometry.len());
            let root = BVH::build(&mut build_geom[..], &mut ordered_geom, &mut total_nodes,
                                  max_geom);
        }
        // TODO: does the BVH even need to store max geom after building?
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
    /// Construct the BVH tree using SAH splitting heuristic to determine split locations
    /// returns the root node of the subtree constructed over the slice of geom info passed
    /// and will increment `total_nodes` by the number of nodes in this subtree
    /// `ordered_geom` will be filled out with the indices of the geometry in the flattened
    /// tree ordering for more efficient access
    fn build(build_info: &mut [GeomInfo<T>], ordered_geom: &mut Vec<usize>,
             total_nodes: &mut usize, max_geom: usize) -> BuildNode {
        *total_nodes += 1;
        // Find bounding box for all geometry we're trying to store at this level
        let bounds = build_info.iter().fold(BBox::new(), |b, g| b.box_union(&g.geom.bounds()));
        let ngeom = build_info.len();
        if ngeom == 1 {
            return BVH::build_leaf(build_info, ordered_geom, bounds);
        }
        // Time to build an interior node
        // Start by figuring out which axis we should be splitting on by finding
        // the axis along which there is the most variation in the geometry's centroids
        let centroids = build_info.iter().fold(BBox::new(), |b, g| b.point_union(&g.center));
        let split_axis = centroids.max_extent();
        let mid = build_info.len() / 2;
        // If all the geometry's centers are on the same point there's no partitioning that makes
        // sense to do
        if centroids.max[split_axis] == centroids.min[split_axis] {
            if ngeom < max_geom {
                return BVH::build_leaf(&mut build_info[..], ordered_geom, bounds);
            } else {
                let l = Box::new(BVH::build(&mut build_info[..mid], ordered_geom,
                                            total_nodes, max_geom));
                let r = Box::new(BVH::build(&mut build_info[mid..], ordered_geom,
                                            total_nodes, max_geom));
                return BuildNode::interior([l, r], split_axis);
            }
        }
        // If there's only a few objects just use an equal partitioning to split
        // Otherwise do a full SAH based split on the geometry
        if ngeom < 5 {
            // TODO: I'd prefer to use something like nth_element like I do in tray
            // here, but I guess a full sort is kind of meh on 5 elements
            // There shouldn't be NaNs in these positions so just give up if there are
            build_info.sort_by(|a, b| {
                match a.center[split_axis].partial_cmp(&b.center[split_axis]) {
                    Some(o) => o,
                    None => panic!("NaNs in build info centers?!"),
                }
            });
        } else {
            // We only consider binning into 12 buckets
            let mut buckets = [SAHBucket::new(); 12];
            // Place geometry into nearest bucket
            for g in build_info.iter() {
                let b = ((g.center[split_axis] - centroids.min[split_axis])
                    / (centroids.max[split_axis] - centroids.min[split_axis]) * buckets.len() as f32) as usize;
                let b = if b == buckets.len() { b - 1 } else { b };
                buckets[b].count += 1;
                buckets[b].bounds = buckets[b].bounds.box_union(&g.bounds);
            }
            // Compute cost of each bucket but the last using the surface area heuristic
            let mut cost = [0.0; 11];
            for (i, c) in cost.iter_mut().enumerate() {
                let left = buckets.iter().take(i).fold(SAHBucket::new(), |mut s, b| {
                    s.bounds = s.bounds.box_union(&b.bounds);
                    s.count += b.count;
                    s
                });
                let right = buckets.iter().skip(i).fold(SAHBucket::new(), |mut s, b| {
                    s.bounds = s.bounds.box_union(&b.bounds);
                    s.count += b.count;
                    s
                });
                *c = 0.125 * (left.count as f32 * left.bounds.surface_area()
                             + right.count as f32 * right.bounds.surface_area()) / bounds.surface_area();
            }
            let (min_bucket, min_cost) = cost.iter().enumerate().fold((0, f32::INFINITY),
                |(pi, pc), (i, c)| {
                    if *c < pc { (i, *c) } else { (pi, pc) }
                });
            // If we're forced to split by the amount of geometry or it's cheaper to split, do so
            if ngeom > max_geom || min_cost < ngeom as f32 {
                // TODO: Implement partition
            }
        }
        let l = Box::new(BVH::build(&mut build_info[..mid], ordered_geom,
                                    total_nodes, max_geom));
        let r = Box::new(BVH::build(&mut build_info[mid..], ordered_geom,
                                    total_nodes, max_geom));
        return BuildNode::interior([l, r], split_axis);
    }
    /// Construct a new leaf node containing the passed geometry. Indices will be
    /// added to `ordered_geom` to instruct how the flattened tree should be placed
    /// in memory for the geometry in this leaf node
    fn build_leaf(build_info: &mut [GeomInfo<T>], ordered_geom: &mut Vec<usize>, bounds: BBox)
        -> BuildNode {
        let geom_offset = ordered_geom.len();
        // TODO: Function to append an iterator? Then we don't need this loop and
        // could do like: `ordered_geom.append(build_info.map(|g| g.geom_idx))`
        for g in build_info.iter() {
            ordered_geom.push(g.geom_idx);
        }
        BuildNode::leaf(build_info.len(), geom_offset, bounds)
    }
}

pub fn partition<'a, T: 'a, I, F>(mut it: I, pred: F) -> usize
        where I: DoubleEndedIterator<Item = &'a mut T>,
        F: Fn(&T) -> bool {
    let mut split_idx = 0;
    loop {
        let mut front = None;
        let mut back = None;
        while let Some(f) = it.next() {
            if pred(f) {
                split_idx += 1;
            } else {
                front = Some(f);
                break;
            }
        }
        while let Some(b) = it.next_back() {
            if pred(b) {
                back = Some(b);
                break;
            }
        }
        match (front, back) {
            (Some(f), Some(b)) => mem::swap(f, b),
            _ => break,
        }
    }
    split_idx
}

