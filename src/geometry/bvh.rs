//! Provides a simple SAH split based BVH2 that stores types implementing the Boundable trait

use std::f32;
use std::iter::repeat;

use partition::partition;
use geometry::{BBox, Boundable};
use linalg::{Point, Ray, Axis, Vector};

/// A standard BVH2 that stores objects that can report their bounds in some space
/// via the `Boundable` trait. The BVH is constructed using a SAH partitioning scheme
pub struct BVH<T: Boundable> {
    /// The geometry stored in this BVH, this will be re-ordered to
    /// fit the BVH construction layout. TODO: We may want to make
    /// the geometry accessible by index
    geometry: Vec<T>,
    /// Indices into `geometry` sorted by the order they're accessed by BVH leaf nodes
    /// TODO: How can we re-sort `geometry to match this ordering?
    ordered_geom: Vec<usize>,
    /// The flattened tree structure of the BVH
    tree: Vec<FlatNode>,
}

impl<T: Boundable> BVH<T> {
    /// Create a new BVH using a SAH construction algorithm
    pub fn new(max_geom: usize, geometry: Vec<T>) -> BVH<T> {
        assert!(!geometry.is_empty());
        let mut flat_tree = Vec::new();
        let mut ordered_geom = Vec::with_capacity(geometry.len());
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
            let root = Box::new(BVH::build(&mut build_geom[..], &mut ordered_geom, &mut total_nodes,
                                  max_geom));
            flat_tree.reserve(total_nodes);
            BVH::<T>::flatten_tree(&root, &mut flat_tree);
            assert_eq!(flat_tree.len(), total_nodes);
            assert_eq!(ordered_geom.len(), geometry.len());
            // TODO: I'm not sure if there's a better way that we can re-sort the geometry by the
            // indices in ordered geom
        }
        BVH { geometry: geometry, ordered_geom: ordered_geom, tree: flat_tree }
    }
    /// Traverse the BVH and call the function passed on the objects in the leaf nodes
    /// of the BVH, returning the value returned by the function after traversal completes
    pub fn intersect<'a, F, R>(&'a self, ray: &mut Ray, f: F) -> Option<R>
            where F: Fn(&mut Ray, &'a T) -> Option<R> {
        let mut result = None;
        let inv_dir = Vector::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let neg_dir = [(ray.d.x < 0.0) as usize, (ray.d.y < 0.0) as usize, (ray.d.z < 0.0) as usize];
        let mut stack = [0; 64];
        let mut stack_ptr = 0;
        let mut current = 0;
        loop {
            let node = &self.tree[current];
            if node.bounds.fast_intersect(ray, &inv_dir, &neg_dir) {
                match node.node {
                    FlatNodeData::Leaf { ref geom_offset, ref ngeom } => {
                        // Call function on all geometry in this leaf
                        for i in &self.ordered_geom[*geom_offset..*geom_offset + *ngeom] {
                            let o = &self.geometry[*i];
                            result = f(ray, o).or(result);
                        }
                        if stack_ptr == 0 {
                            break;
                        }
                        stack_ptr -= 1;
                        current = stack[stack_ptr];
                    },
                    FlatNodeData::Interior { ref second_child, ref axis } => {
                        let a = match *axis {
                            Axis::X => 0,
                            Axis::Y => 1,
                            Axis::Z => 2,
                        };
                        if neg_dir[a] != 0 {
                            stack[stack_ptr] = current + 1;
                            current = *second_child;
                        } else {
                            stack[stack_ptr] = *second_child;
                            current += 1;
                        }
                        stack_ptr += 1;
                    },
                }
            } else {
                if stack_ptr == 0 {
                    break;
                }
                stack_ptr -= 1;
                current = stack[stack_ptr];
            }
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
        let mut mid = build_info.len() / 2;
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
                mid = partition(build_info.iter_mut(),
                    |g| {
                        let b = ((g.center[split_axis] - centroids.min[split_axis])
                                 / (centroids.max[split_axis] - centroids.min[split_axis]) * buckets.len() as f32) as usize;
                        let b = if b == buckets.len() { b - 1 } else { b };
                        b <= min_bucket
                    });
                // partition returns the index of the first element in the false group
                // TODO: Something is wrong, we shouldn't be getting mid like this
                mid =
                    if mid > 1 { mid - 1 }
                    else {
                        println!("Bad mid encountered! mid = {}, build_info.len() = {}", mid, build_info.len());
                        mid
                    };
            }
            else {
                return BVH::build_leaf(build_info, ordered_geom, bounds);
            }
        }
        assert!(mid != 0 && mid != build_info.len());
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
    /// Flatten the BVH sub-tree pointed too by the node passed into the vec passed
    /// Returns the index that the node was inserted at in the tree
    fn flatten_tree(node: &Box<BuildNode>, tree: &mut Vec<FlatNode>) -> usize {
        let offset = tree.len();
        match node.node {
            BuildNodeData::Interior { children: ref c, split_axis: ref a } => {
                // Push this node on followed by the first child and the second
                tree.push(FlatNode::interior(node.bounds, 0, *a));
                BVH::<T>::flatten_tree(&c[0], tree);
                let second_child = BVH::<T>::flatten_tree(&c[1], tree);
                // This is a little awkward, TODO: maybe better to call resize?
                match &mut tree[offset].node {
                    &mut FlatNodeData::Interior { second_child: ref mut s, axis: _ } => *s = second_child,
                    _ => panic!("Interior node switched to leaf!?"),
                };
            },
            BuildNodeData::Leaf { ngeom: ref n, geom_offset: ref o } => {
                tree.push(FlatNode::leaf(node.bounds, *o, *n));
            },
        }
        offset
    }
}

impl<T: Boundable> Boundable for BVH<T> {
    fn bounds(&self) -> BBox {
        self.tree[0].bounds
    }
}

/// Data for flattened BVH nodes
#[derive(Debug)]
enum FlatNodeData {
    /// An interior node is flattened with its first child following it
    /// and the second child at some later index in the tree
    Interior { second_child: usize, axis: Axis },
    /// A leaf node stores information about the offset too its geometry
    /// and the number of geometry references
    Leaf { geom_offset: usize, ngeom: usize },
}

/// Final datastructure that the flattened BVH is stored in
#[derive(Debug)]
struct FlatNode {
    /// Bounding box of this node
    bounds: BBox,
    /// Information about this node, storing interior/leaf node data corresponding
    /// to the type of the node
    node: FlatNodeData,
}

impl FlatNode {
    /// Construct a new flattened interior node
    fn interior(bounds: BBox, second_child: usize, axis: Axis) -> FlatNode {
        FlatNode { bounds: bounds, node: FlatNodeData::Interior { second_child: second_child, axis: axis } }
    }
    /// Construct a new flattened leaf node
    fn leaf(bounds: BBox, geom_offset: usize, ngeom: usize) -> FlatNode {
        FlatNode { bounds: bounds, node: FlatNodeData::Leaf { geom_offset: geom_offset, ngeom: ngeom } }
    }
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
#[derive(Debug)]
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
#[derive(Debug)]
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
    fn print_tree(&self, depth: usize) {
        let ident: String = repeat(" ").take(depth).collect();
        println!("{}BuildNode: {{", ident);
        let pad: String = repeat(" ").take(depth + 2).collect();
        println!("{}bounds: {:?}", pad, self.bounds);
        match self.node {
            BuildNodeData::Interior { children: ref c, split_axis: ref a } => {
                println!("{}type: Interior", pad);
                println!("{}split axis: {:?}", pad, a);
                println!("{}left child:", pad);
                c[0].print_tree(depth + 2);
                println!("{}right child:", pad);
                c[1].print_tree(depth + 2);
            },
            BuildNodeData::Leaf { ngeom: ref n, geom_offset: ref o } => {
                println!("{}type: Leaf", pad);
                println!("{}ngeom: {}", pad, n);
                println!("{}geom offset: {}", pad, o);
            },
        }
        println!("{}}}", ident);
    }
}

#[derive(Copy, Clone, Debug)]
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

