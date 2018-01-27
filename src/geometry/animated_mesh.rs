//! Defines an animated triangle mesh geometry. It is required that the topology
//! of the mesh being animated does not change. While properties of triangles can
//! be changed over time triangles can't be added or removed. Intersection test
//! are accelerated internally by storing the triangles of the mesh in a BVH
//!
//! # Scene Usage Example
//! The mesh is specified by the OBJ file to load and the name of the specific
//! model within the file to use. The file and other loaded models are kept loaded
//! so you can easily use the same or other models in the file as well. If no name is
//! assigned to the model in the file it will be given the name "`unnamed_model`",
//! however it's recommended to name your models.
//!
//! ```json
//! "geometry": {
//!     "type": "animated_mesh",
//!     "model": "Suzanne"
//!     "keyframes": [
//!         {
//!             "file": "./suzanne001.obj",
//!             "time": 0
//!         },
//!         {
//!             "file": "./suzanne002.obj",
//!             "time": 0.1
//!         }
//!     ]
//! }
//! ```

extern crate tobj;

use std::sync::Arc;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox, BVH, Mesh};
use geometry::mesh::intersect_triangle;
use linalg::{self, Normal, Vector, Ray, Point, lerp};

pub struct AnimatedMeshData {
    positions: Vec<Arc<Vec<Point>>>,
    normals: Vec<Arc<Vec<Normal>>>,
    texcoords: Vec<Arc<Vec<Point>>>,
    times: Vec<f32>,
}
impl AnimatedMeshData {
    pub fn new(positions: Vec<Arc<Vec<Point>>>, normals: Vec<Arc<Vec<Normal>>>,
               texcoords: Vec<Arc<Vec<Point>>>, times: Vec<f32>) -> AnimatedMeshData {
        AnimatedMeshData {
            positions: positions,
            normals: normals,
            texcoords: texcoords,
            times: times
        }
    }
    /// Get the active indices in the buffers for some time
    fn active_keyframes(&self, time: f32) -> (usize, Option<usize>) {
        match self.times.binary_search_by(|t| t.partial_cmp(&time).unwrap()) {
            Ok(i) => (i, None),
            Err(i) => {
                if i == self.times.len() {
                    (i - 1, None)
                } else if i == 0 {
                    (0, None)
                } else {
                    (i - 1, Some(i))
                }
            },
        }
    }
    /// Get the position at some time
    fn position(&self, i: usize, time: f32) -> Point {
        match self.active_keyframes(time) {
            (lo, None) => (*self.positions[lo])[i],
            (lo, Some(hi)) => {
                let a = (*self.positions[lo])[i];
                let b = (*self.positions[hi])[i];
                let x = (time - self.times[lo]) / (self.times[hi] - self.times[lo]);
                lerp(x, &a, &b)
            }
        }
    }
    /// Get the normal at some time
    fn normal(&self, i: usize, time: f32) -> Normal {
        match self.active_keyframes(time) {
            (lo, None) => (*self.normals[lo])[i],
            (lo, Some(hi)) => {
                let a = (*self.normals[lo])[i];
                let b = (*self.normals[hi])[i];
                let x = (time - self.times[lo]) / (self.times[hi] - self.times[lo]);
                lerp(x, &a, &b)
            }
        }
    }
    /// Get the texture coordinate at some time
    fn texcoord(&self, i: usize, time: f32) -> Point {
        match self.active_keyframes(time) {
            (lo, None) => (*self.texcoords[lo])[i],
            (lo, Some(hi)) => {
                let a = (*self.texcoords[lo])[i];
                let b = (*self.texcoords[hi])[i];
                let x = (time - self.times[lo]) / (self.times[hi] - self.times[lo]);
                lerp(x, &a, &b)
            }
        }
    }
}

/// An animated mesh composed of a series of meshes linearly interpolated between
/// over time. It's assumed the mesh topology does not change.
pub struct AnimatedMesh {
    bvh: BVH<AnimatedTriangle>,
}

impl AnimatedMesh {
    /// Create a new AnimatedMesh from the meshes passed. It's assumed the meshes
    /// are sorted in ascending time
    pub fn new(meshes: Vec<Arc<Mesh>>, times: Vec<f32>) -> AnimatedMesh {
        let pos = meshes.iter().map(|m| m.bvh.iter().next().unwrap().positions.clone()).collect();
        let normals = meshes.iter().map(|m| m.bvh.iter().next().unwrap().normals.clone()).collect();
        let tex = meshes.iter().map(|m| m.bvh.iter().next().unwrap().texcoords.clone()).collect();
        let data = Arc::new(AnimatedMeshData::new(pos, normals, tex, times));
        let tris = meshes[0].bvh.iter().map(|t| {
            AnimatedTriangle::new(t.a, t.b, t.c, data.clone())
        }).collect();
        AnimatedMesh {
            bvh: BVH::new(16, tris, data.times[0], data.times[1]),
        }
    }
}

impl Geometry for AnimatedMesh {
    fn intersect(&self, ray: &mut linalg::Ray) -> Option<DifferentialGeometry> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
}

impl Boundable for AnimatedMesh {
    fn bounds(&self, start: f32, end: f32) -> BBox {
        self.bvh.bounds(start, end)
    }
    fn update_deformation(&mut self, start: f32, end: f32) {
        self.bvh.rebuild(start, end);
    }
}

/// An animated triangle in the mesh. Just stores a reference to the mesh
/// and the indices of each vertex
pub struct AnimatedTriangle {
    a: usize,
    b: usize,
    c: usize,
    data: Arc<AnimatedMeshData>,
}

impl AnimatedTriangle {
    /// Create a new triangle representing a triangle within the mesh passed
    pub fn new(a: usize, b: usize, c: usize, data: Arc<AnimatedMeshData>) -> AnimatedTriangle {
        AnimatedTriangle { a: a, b: b, c: c, data: data }
    }
}

impl Geometry for AnimatedTriangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let pa = self.data.position(self.a, ray.time);
        let pb = self.data.position(self.b, ray.time);
        let pc = self.data.position(self.c, ray.time);
        let na = self.data.normal(self.a, ray.time);
        let nb = self.data.normal(self.b, ray.time);
        let nc = self.data.normal(self.c, ray.time);
        let ta = self.data.texcoord(self.a, ray.time);
        let tb = self.data.texcoord(self.b, ray.time);
        let tc = self.data.texcoord(self.c, ray.time);
        intersect_triangle(self, ray, &pa, &pb, &pc, &na, &nb, &nc, &ta, &tb, &tc)
    }
}

impl Boundable for AnimatedTriangle {
    fn bounds(&self, start: f32, end: f32) -> BBox {
        BBox::singular(self.data.position(self.a, start))
            .point_union(&self.data.position(self.b, start))
            .point_union(&self.data.position(self.c, start))
            .point_union(&self.data.position(self.a, end))
            .point_union(&self.data.position(self.b, end))
            .point_union(&self.data.position(self.c, end))
    }
}

