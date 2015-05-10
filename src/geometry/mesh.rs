//! Defines a triangle mesh geometry. Intersection tests are accelerated internally
//! by storing the triangles of the mesh in a BVH

use std::sync::Arc;

use geometry::{Geometry, DifferentialGeometry, Boundable, BBox, BVH};
use linalg;
use linalg::{Normal, Vector, Ray, Point};

/// A mesh composed of triangles, specified by directly passing the position,
/// normal and index buffers for the triangles making up the mesh
pub struct Mesh {
    bvh: BVH<Triangle>,
}

impl Mesh {
    /// Create a new Mesh from the triangles described in the buffers passed
    /// This data could come from an OBJ file via [tobj](https://github.com/Twinklebear/tobj)
    /// for example.
    pub fn new(positions: Arc<Vec<Point>>, normals: Arc<Vec<Normal>>, texcoords: Arc<Vec<Point>>, indices: Vec<usize>) -> Mesh {
        let triangles = indices.chunks(3).map(|i| {
            Triangle::new(i[0], i[1], i[2], positions.clone(), normals.clone(), texcoords.clone())
            }).collect();
        Mesh { bvh: BVH::new(16, triangles) }
    }
}

impl Geometry for Mesh {
    fn intersect(&self, ray: &mut linalg::Ray) -> Option<DifferentialGeometry> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
}

impl Boundable for Mesh {
    fn bounds(&self) -> BBox {
        self.bvh.bounds()
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
        if bary[0] < -1.0e-8 || bary[0] > 1.0 {
            return None;
        }

        s[1] = linalg::cross(&d, &e[0]);
        bary[1] = linalg::dot(&ray.d, &s[1]) * div;
        // Check the second barycentric coordinate is in the triangle bounds
        if bary[1] < -1.0e-8 || bary[0] + bary[1] > 1.0 {
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
        Some(DifferentialGeometry::new(&p, &n, &dp_du, &dp_dv, self))
    }
}

impl Boundable for Triangle {
    fn bounds(&self) -> BBox {
        BBox::singular(self.positions[self.a])
            .point_union(&self.positions[self.b])
            .point_union(&self.positions[self.c])
    }
}

