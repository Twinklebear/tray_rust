use std::num::FloatMath;
use linalg;
use linalg::Matrix4;
use linalg::Vector;
use linalg::Point;
use linalg::Normal;
use linalg::Ray;

/// Transform describes an affine transformation in 3D space
/// and stores both the transformation and its inverse
#[deriving(Show, Copy, PartialEq)]
pub struct Transform {
    pub mat: Matrix4,
    pub inv: Matrix4,
}

impl Transform {
    /// Construct the identity transformation
    pub fn identity() -> Transform {
        Transform {
            mat: Matrix4::identity(),
            inv: Matrix4::identity(),
        }
    }
    /// Construct a transform from an existing matrix
    pub fn from_mat(mat: &Matrix4) -> Transform {
        Transform { mat: *mat, inv: mat.inverse() }
    }
    /// Construct a transform from an existing matrix/inverse pair
    pub fn from_pair(mat: &Matrix4, inv: &Matrix4) -> Transform {
        Transform { mat: *mat, inv: *inv }
    }
    /// Construct a transformation matrix to translate by the vector
    pub fn translate(v: &Vector) -> Transform {
        Transform {
            mat: Matrix4::new([1f32, 0f32, 0f32, v.x,
                               0f32, 1f32, 0f32, v.y,
                               0f32, 0f32, 1f32, v.z,
                               0f32, 0f32, 0f32, 1f32]),
            inv: Matrix4::new([1f32, 0f32, 0f32, -v.x,
                               0f32, 1f32, 0f32, -v.y,
                               0f32, 0f32, 1f32, -v.z,
                               0f32, 0f32, 0f32, 1f32]),
        }
    }
    /// Construct a transform to scale x, y and z by the values in the vector
    pub fn scale(v: &Vector) -> Transform {
        Transform {
            mat: Matrix4::new([v.x, 0f32, 0f32, 0f32,
                               0f32, v.y, 0f32, 0f32,
                               0f32, 0f32, v.z, 0f32,
                               0f32, 0f32, 0f32, 1f32]),
            inv: Matrix4::new([1f32 / v.x, 0f32, 0f32, 0f32,
                               0f32, 1f32 / v.y, 0f32, 0f32,
                               0f32, 0f32, 1f32 / v.z, 0f32,
                               0f32, 0f32, 0f32, 1f32]),
        }
    }
    /// Construct a transform to rotate `deg` degrees about the x axis
    pub fn rotate_x(deg: f32) -> Transform {
        let r = linalg::radians(deg);
        let s = FloatMath::sin(r);
        let c = FloatMath::cos(r);
        let m = Matrix4::new([1f32, 0f32, 0f32, 0f32,
                              0f32, c, -s, 0f32,
                              0f32, s, c, 0f32,
                              0f32, 0f32, 0f32, 1f32]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate `deg` degrees about the y axis
    pub fn rotate_y(deg: f32) -> Transform {
        let r = linalg::radians(deg);
        let s = FloatMath::sin(r);
        let c = FloatMath::cos(r);
        let m = Matrix4::new([c, 0f32, s, 0f32,
                              0f32, 1f32, 0f32, 0f32,
                              -s, 0f32, c, 0f32,
                              0f32, 0f32, 0f32, 1f32]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate `deg` degrees about the z axis
    pub fn rotate_z(deg: f32) -> Transform {
        let r = linalg::radians(deg);
        let s = FloatMath::sin(r);
        let c = FloatMath::cos(r);
        let m = Matrix4::new([c, -s, 0f32, 0f32,
                              s, c, 0f32, 0f32,
                              0f32, 0f32, 1f32, 0f32,
                              0f32, 0f32, 0f32, 1f32]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate about `axis` by `deg` degrees
    pub fn rotate(axis: &Vector, deg: f32) -> Transform {
        let a = axis.normalized();
        let r = linalg::radians(deg);
        let s = FloatMath::sin(r);
        let c = FloatMath::cos(r);
        let mut m = Matrix4::identity();
        *m.at_mut(0, 0) = a.x * a.x + (1f32 - a.x * a.x) * c;
        *m.at_mut(0, 1) = a.x * a.y * (1f32 - c) - a.z * s;
        *m.at_mut(0, 2) = a.x * a.z * (1f32 - c) + a.y * s;

        *m.at_mut(1, 0) = a.x * a.y * (1f32 - c) + a.z * s;
        *m.at_mut(1, 1) = a.y * a.y + (1f32 - a.y * a.y) * c;
        *m.at_mut(1, 2) = a.y * a.z * (1f32 - c) - a.x * s;

        *m.at_mut(2, 0) = a.x * a.z * (1f32 - c) - a.y * s;
        *m.at_mut(2, 1) = a.y * a.z * (1f32 - c) + a.x * s;
        *m.at_mut(2, 2) = a.z * a.z + (1f32 - a.z * a.z) * c;
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct the look at transform for a camera at `pos` looking at
    /// the point `center` oriented with up vector `up`
    pub fn look_at(pos: &Point, center: &Point, up: &Vector) -> Transform {
        let dir = (*center - *pos).normalized();
        let right = linalg::cross(&dir, up).normalized();
        let u = linalg::cross(&dir, &right).normalized();
        let mut m = Matrix4::identity();
        for i in range(0u, 3u) {
            *m.at_mut(i, 0) = right[i];
            *m.at_mut(i, 1) = u[i];
            *m.at_mut(i, 2) = dir[i];
            *m.at_mut(i, 3) = pos[i];
        }
        Transform { mat: m, inv: m.inverse() }
    }
    /// Construct a perspective transformation
    pub fn perspective(fovy: f32, near: f32, far: f32) -> Transform {
        let proj_div = Matrix4::new([1f32, 0f32, 0f32, 0f32,
                                     0f32, 1f32, 0f32, 0f32,
                                     0f32, 0f32, far / (far - near), -far * near / (far - near),
                                     0f32, 0f32, 1f32, 0f32]);
        let inv_tan = 1f32 / FloatMath::atan(linalg::radians(fovy) / 2f32);
        Transform::scale(&Vector::new(inv_tan, inv_tan, 1f32)) * Transform::from_mat(&proj_div)
    }
    /// Return the inverse of the transformation
    pub fn inverse(&self) -> Transform {
        Transform { mat: self.inv, inv: self.mat }
    }
    /// Apply the transformation to a point
    pub fn point(&self, p: &Point) -> Point {
        let mut res = Point::broadcast(0f32);
        for i in range(0u, 3u) {
            res[i] = *self.mat.at(i, 0) * p.x + *self.mat.at(i, 1) * p.y
                + *self.mat.at(i, 2) * p.z + *self.mat.at(i, 3);
        }
        let w = *self.mat.at(3, 0) * p.x + *self.mat.at(3, 1) * p.y
            + *self.mat.at(3, 2) * p.z + *self.mat.at(3, 3);
        if w != 1f32 {
            res / w
        } else {
            res
        }
    }
    /// Apply the transformation to a vector
    pub fn vector(&self, v: &Vector) -> Vector {
        let mut res = Vector::broadcast(0f32);
        for i in range(0u, 3u) {
            res[i] = *self.mat.at(i, 0) * v.x + *self.mat.at(i, 1) * v.y
                + *self.mat.at(i, 2) * v.z;
        }
        res
    }
    /// Apply the transformation to a normal
    pub fn normal(&self, n: &Normal) -> Normal {
        let mut res = Normal::broadcast(0f32);
        for i in range(0u, 3u) {
            res[i] = *self.inv.at(0, i) * n.x + *self.inv.at(1, i) * n.y
                + *self.inv.at(2, i) * n.z;
        }
        res
    }
    /// Apply the transformation to a ray
    pub fn ray(&self, ray: &Ray) -> Ray {
        let mut res = *ray;
        res.o = self.point(&res.o);
        res.d = self.vector(&res.d);
        res
    }
}

impl Mul<Transform, Transform> for Transform {
    /// Compose two transformations
    fn mul(self, rhs: Transform) -> Transform {
        Transform { mat: self.mat * rhs.mat, inv: rhs.inv * self.inv }
    }
}

