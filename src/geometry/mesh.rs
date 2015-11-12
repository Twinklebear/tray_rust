//! Defines a triangle mesh geometry. Intersection tests are accelerated internally
//! by storing the triangles of the mesh in a BVH
//!
//! # Scene Usage Example
//! The mesh is specified by the OBJ file to load and the name of the specific
//! model within the file to use. The file and other loaded models are kept loaded
//! so you can easily use the same or other models in the file as well. If no name is
//! assigned to the model in the file it will be given the name "unnamed_model",
//! however it's recommended to name your models.
//!
//! ```json
//! "geometry": {
//!     "type": "mesh",
//!     "file": "./suzanne.obj",
//!     "model": "Suzanne"
//! }
//! ```

extern crate tobj;

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox, BVH};
use linalg::{self, Normal, Vector, Ray, Point};

/// A mesh composed of triangles, specified by directly passing the position,
/// normal and index buffers for the triangles making up the mesh
pub struct Mesh {
    bvh: BVH<Triangle>,
}

impl Mesh {
    /// Create a new Mesh from the triangles described in the buffers passed
    /// This data could come from an OBJ file via [tobj](https://github.com/Twinklebear/tobj)
    /// for example.
    pub fn new(positions: Arc<Vec<Point>>, normals: Arc<Vec<Normal>>, texcoords: Arc<Vec<Point>>,
               indices: Vec<u32>) -> Mesh {
        let triangles = indices.chunks(3).map(|i| {
            Triangle::new(i[0] as usize, i[1] as usize, i[2] as usize, positions.clone(),
                          normals.clone(), texcoords.clone())
            }).collect();
        Mesh { bvh: BVH::unanimated(16, triangles) }
    }
    /// Load all the meshes defined in an OBJ file and return them in a hashmap that maps the
    /// model's name in the file to its loaded mesh
    /// TODO: Currently materials are ignored
    pub fn load_obj(file_name: &Path) -> HashMap<String, Arc<Mesh>> {
        match tobj::load_obj(file_name) {
            Ok((models, _)) => {
                let mut meshes = HashMap::new();
                for m in models {
                    println!("Loading model {}", m.name);
                    let mesh = m.mesh;
                    if mesh.normals.is_empty() || mesh.texcoords.is_empty() {
                        print!("Mesh::load_obj error! Normals and texture coordinates are required!");
                        println!("Skipping {}", m.name);
                        continue;
                    }
                    println!("{} has {} triangles", m.name, mesh.indices.len() / 3);
                    let positions = Arc::new(mesh.positions.chunks(3).map(|i| Point::new(i[0], i[1], i[2]))
                                             .collect());
                    let normals = Arc::new(mesh.normals.chunks(3).map(|i| Normal::new(i[0], i[1], i[2]))
                                           .collect());
                    let texcoords = Arc::new(mesh.texcoords.chunks(2).map(|i| Point::new(i[0], i[1], 0.0))
                                             .collect());
                    meshes.insert(m.name, Arc::new(Mesh::new(positions, normals, texcoords, mesh.indices)));
                }
                meshes
            },
            Err(e) => {
                println!("Failed to load {:?} due to {:?}", file_name, e);
                HashMap::new()
            },
        }
    }
}

impl Geometry for Mesh {
    fn intersect(&self, ray: &mut linalg::Ray) -> Option<DifferentialGeometry> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
}

impl Boundable for Mesh {
    fn bounds(&self, start: f32, end: f32) -> BBox {
        self.bvh.bounds(start, end)
    }
}

/// A triangle in some mesh. Just stores a reference to the mesh
/// and the indices of each vertex
pub struct Triangle {
    a: usize,
    b: usize,
    c: usize,
    positions: Arc<Vec<Point>>,
    normals: Arc<Vec<Normal>>,
    texcoords: Arc<Vec<Point>>,
}

impl Triangle {
    /// Create a new triangle representing a triangle within the mesh passed
    pub fn new(a: usize, b: usize, c: usize, positions: Arc<Vec<Point>>,
               normals: Arc<Vec<Normal>>, texcoords: Arc<Vec<Point>>) -> Triangle {
        Triangle { a: a, b: b, c: c, positions: positions, normals: normals,
                   texcoords: texcoords }
    }
}

impl Geometry for Triangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let pa = &self.positions[self.a];
        let pb = &self.positions[self.b];
        let pc = &self.positions[self.c];

        let e = [*pb - *pa, *pc - *pa];
        let mut s = [Vector::broadcast(0.0); 2];
        s[0] = linalg::cross(&ray.d, &e[1]);
        let div = match linalg::dot(&s[0], &e[0]) {
            // 0.0 => degenerate triangle, can't hit
            0.0 => return None,
            d => 1.0 / d,
        };

        let d = ray.o - *pa;
        let mut bary = [0.0; 3];
        bary[0] = linalg::dot(&d, &s[0]) * div;
        // Check that the first barycentric coordinate is in the triangle bounds
        if bary[0] < 0.0 || bary[0] > 1.0 {
            return None;
        }

        s[1] = linalg::cross(&d, &e[0]);
        bary[1] = linalg::dot(&ray.d, &s[1]) * div;
        // Check the second barycentric coordinate is in the triangle bounds
        if bary[1] < 0.0 || bary[0] + bary[1] > 1.0 {
            return None;
        }

        // We've hit the triangle with the ray, now check the hit location is in the ray range
        let t = linalg::dot(&e[1], &s[1]) * div;
        if t < ray.min_t || t > ray.max_t {
            return None;
        }
        bary[2] = 1.0 - bary[0] - bary[1];
        ray.max_t = t;
        let p = ray.at(t);

        // Now compute normal at this location on the triangle
        let na = &self.normals[self.a];
        let nb = &self.normals[self.b];
        let nc = &self.normals[self.c];
        let n = (bary[2] * *na + bary[0] * *nb + bary[1] * *nc).normalized();

        // Compute parameterization of surface and various derivatives for texturing
        // Triangles are parameterized by the obj texcoords at the vertices
        let ta = &self.texcoords[self.a];
        let tb = &self.texcoords[self.b];
        let tc = &self.texcoords[self.c];
        // Triangle points can be found by p_i = p_0 + u_i dp/du + v_i dp/dv
        // we use this property to find the derivatives dp/du and dp/dv
        let du = [ta.x - tc.x, tb.x - tc.x];
        let dv = [ta.y - tc.y, tb.y - tc.y];
        let det = du[0] * dv[1] - dv[0] * du[1];
        //If the texcoords are degenerate pick arbitrary coordinate system
        let (dp_du, dp_dv) =
            if det == 0.0 {
                linalg::coordinate_system(&linalg::cross(&e[1], &e[0]).normalized())
            }
            else {
                let det = 1.0 / det;
                let dp = [*pa - *pc, *pb - *pc];
                let dp_du = (dv[1] * dp[0] - dv[0] * dp[1]) * det;
                let dp_dv = (-du[1] * dp[0] + du[0] * dp[1]) * det;
                (dp_du, dp_dv)
            };
        Some(DifferentialGeometry::with_normal(&p, &n, &dp_du, &dp_dv, self))
    }
}

impl Boundable for Triangle {
    fn bounds(&self, _: f32, _: f32) -> BBox {
        BBox::singular(self.positions[self.a])
            .point_union(&self.positions[self.b])
            .point_union(&self.positions[self.c])
    }
}

