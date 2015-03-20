use std::num::Float;
use std::ops::Mul;

use linalg;
use linalg::Matrix4;
use linalg::Vector;
use linalg::Point;
use linalg::Normal;
use linalg::Ray;

/// Transform describes an affine transformation in 3D space
/// and stores both the transformation and its inverse
#[derive(Debug, Copy, PartialEq)]
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
            mat: Matrix4::new([1.0, 0.0, 0.0, v.x,
                               0.0, 1.0, 0.0, v.y,
                               0.0, 0.0, 1.0, v.z,
                               0.0, 0.0, 0.0, 1.0]),
            inv: Matrix4::new([1.0, 0.0, 0.0, -v.x,
                               0.0, 1.0, 0.0, -v.y,
                               0.0, 0.0, 1.0, -v.z,
                               0.0, 0.0, 0.0, 1.0]),
        }
    }
    /// Construct a transform to scale x, y and z by the values in the vector
    pub fn scale(v: &Vector) -> Transform {
        Transform {
            mat: Matrix4::new([v.x, 0.0, 0.0, 0.0,
                               0.0, v.y, 0.0, 0.0,
                               0.0, 0.0, v.z, 0.0,
                               0.0, 0.0, 0.0, 1.0]),
            inv: Matrix4::new([1.0 / v.x, 0.0, 0.0, 0.0,
                               0.0, 1.0 / v.y, 0.0, 0.0,
                               0.0, 0.0, 1.0 / v.z, 0.0,
                               0.0, 0.0, 0.0, 1.0]),
        }
    }
    /// Construct a transform to rotate `deg` degrees about the x axis
    pub fn rotate_x(deg: f32) -> Transform {
        let r = Float::to_radians(deg);
        let s = Float::sin(r);
        let c = Float::cos(r);
        let m = Matrix4::new([1.0, 0.0, 0.0, 0.0,
                              0.0, c, -s, 0.0,
                              0.0, s, c, 0.0,
                              0.0, 0.0, 0.0, 1.0]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate `deg` degrees about the y axis
    pub fn rotate_y(deg: f32) -> Transform {
        let r = Float::to_radians(deg);
        let s = Float::sin(r);
        let c = Float::cos(r);
        let m = Matrix4::new([c, 0.0, s, 0.0,
                              0.0, 1.0, 0.0, 0.0,
                              -s, 0.0, c, 0.0,
                              0.0, 0.0, 0.0, 1.0]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate `deg` degrees about the z axis
    pub fn rotate_z(deg: f32) -> Transform {
        let r = Float::to_radians(deg);
        let s = Float::sin(r);
        let c = Float::cos(r);
        let m = Matrix4::new([c, -s, 0.0, 0.0,
                              s, c, 0.0, 0.0,
                              0.0, 0.0, 1.0, 0.0,
                              0.0, 0.0, 0.0, 1.0]);
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct a transform to rotate about `axis` by `deg` degrees
    pub fn rotate(axis: &Vector, deg: f32) -> Transform {
        let a = axis.normalized();
        let r = Float::to_radians(deg);
        let s = Float::sin(r);
        let c = Float::cos(r);
        let mut m = Matrix4::identity();
        *m.at_mut(0, 0) = a.x * a.x + (1.0 - a.x * a.x) * c;
        *m.at_mut(0, 1) = a.x * a.y * (1.0 - c) - a.z * s;
        *m.at_mut(0, 2) = a.x * a.z * (1.0 - c) + a.y * s;

        *m.at_mut(1, 0) = a.x * a.y * (1.0 - c) + a.z * s;
        *m.at_mut(1, 1) = a.y * a.y + (1.0 - a.y * a.y) * c;
        *m.at_mut(1, 2) = a.y * a.z * (1.0 - c) - a.x * s;

        *m.at_mut(2, 0) = a.x * a.z * (1.0 - c) - a.y * s;
        *m.at_mut(2, 1) = a.y * a.z * (1.0 - c) + a.x * s;
        *m.at_mut(2, 2) = a.z * a.z + (1.0 - a.z * a.z) * c;
        Transform { mat: m, inv: m.transpose() }
    }
    /// Construct the look at transform for a camera at `pos` looking at
    /// the point `center` oriented with up vector `up`
    pub fn look_at(pos: &Point, center: &Point, up: &Vector) -> Transform {
        let dir = (*center - *pos).normalized();
        let left = linalg::cross(&up.normalized(), &dir).normalized();
        let u = linalg::cross(&dir, &left).normalized();
        let mut m = Matrix4::identity();
        for i in 0..3 {
            *m.at_mut(i, 0) = -left[i];
            *m.at_mut(i, 1) = u[i];
            *m.at_mut(i, 2) = dir[i];
            *m.at_mut(i, 3) = pos[i];
        }
        Transform { mat: m, inv: m.inverse() }
    }
    /// Construct a perspective transformation
    pub fn perspective(fovy: f32, near: f32, far: f32) -> Transform {
        let proj_div = Matrix4::new(
            [1.0, 0.0, 0.0, 0.0,
             0.0, 1.0, 0.0, 0.0,
             0.0, 0.0, far / (far - near), -far * near / (far - near),
             0.0, 0.0, 1.0, 0.0]);
        let inv_tan = 1.0 / Float::tan(Float::to_radians(fovy) / 2.0);
        Transform::scale(&Vector::new(inv_tan, inv_tan, 1.0))
            * Transform::from_mat(&proj_div)
    }
    /// Return the inverse of the transformation
    pub fn inverse(&self) -> Transform {
        Transform { mat: self.inv, inv: self.mat }
    }
    /// Multiply the point by the inverse transformation
    /// TODO: These inverse mults are a bit hacky since Rust doesn't currently
    /// have function overloading, clean up when it's added
    pub fn inv_mul_point(&self, p: &Point) -> Point {
        let mut res = Point::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.inv.at(i, 0) * p.x + *self.inv.at(i, 1) * p.y
                + *self.inv.at(i, 2) * p.z + *self.inv.at(i, 3);
        }
        let w = *self.inv.at(3, 0) * p.x + *self.inv.at(3, 1) * p.y
            + *self.inv.at(3, 2) * p.z + *self.inv.at(3, 3);
        if w != 1.0 {
            res / w
        } else {
            res
        }
    }
    /// Multiply the vector with the inverse transformation
    pub fn inv_mul_vector(&self, v: &Vector) -> Vector {
        let mut res = Vector::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.inv.at(i, 0) * v.x + *self.inv.at(i, 1) * v.y
                + *self.inv.at(i, 2) * v.z;
        }
        res
    }
    /// Multiply the normal with the inverse transformation
    pub fn inv_mul_normal(&self, n: &Normal) -> Normal {
        let mut res = Normal::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.mat.at(0, i) * n.x + *self.mat.at(1, i) * n.y
                + *self.mat.at(2, i) * n.z;
        }
        res
    }
    /// Multiply the ray with the inverse transformation
    pub fn inv_mul_ray(&self, ray: &Ray) -> Ray {
        let mut res = *ray;
        res.o = self.inv_mul_point(&res.o);
        res.d = self.inv_mul_vector(&res.d);
        res
    }
}

impl Mul for Transform {
    type Output = Transform;
    /// Compose two transformations
    fn mul(self, rhs: Transform) -> Transform {
        Transform { mat: self.mat * rhs.mat, inv: rhs.inv * self.inv }
    }
}

impl Mul<Point> for Transform {
    type Output = Point;
    /// Multiply the point by the transform to apply the transformation
    fn mul(self, p: Point) -> Point {
        let mut res = Point::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.mat.at(i, 0) * p.x + *self.mat.at(i, 1) * p.y
                + *self.mat.at(i, 2) * p.z + *self.mat.at(i, 3);
        }
        let w = *self.mat.at(3, 0) * p.x + *self.mat.at(3, 1) * p.y
            + *self.mat.at(3, 2) * p.z + *self.mat.at(3, 3);
        if w != 1.0 {
            res / w
        } else {
            res
        }
    }
}

impl Mul<Vector> for Transform {
    type Output = Vector;
    /// Multiply the vector by the transform to apply the transformation
    fn mul(self, v: Vector) -> Vector {
        let mut res = Vector::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.mat.at(i, 0) * v.x + *self.mat.at(i, 1) * v.y
                + *self.mat.at(i, 2) * v.z;
        }
        res
    }
}

impl Mul<Normal> for Transform {
    type Output = Normal;
    /// Multiply the normal by the transform to apply the transformation
    fn mul(self, n: Normal) -> Normal {
        let mut res = Normal::broadcast(0.0);
        for i in 0..3 {
            res[i] = *self.inv.at(0, i) * n.x + *self.inv.at(1, i) * n.y
                + *self.inv.at(2, i) * n.z;
        }
        res
    }
}

impl Mul<Ray> for Transform {
    type Output = Ray;
    /// Multiply the ray by the transform to apply the transformation
    fn mul(self, ray: Ray) -> Ray {
        let mut res = ray;
        res.o = self * res.o;
        res.d = self * res.d;
        res
    }
}

// Just test against multiplying by the identity matrix as a sanity baseline
#[test]
fn test_mult_sanity() {
    let t = Transform::identity();
    let p = Point::new(1.0, 2.0, 3.0);
    let v = Vector::new(1.0, 2.0, 3.0);
    let n = Normal::new(1.0, 2.0, 3.0);
    assert_eq!(t * p, p);
    assert_eq!(t * v, v);
    assert_eq!(t * n, n);
}
#[test]
fn test_translate() {
    let t = Transform::translate(&Vector::new(1.0, 2.0, 3.0));
    let p = Point::new(1.0, 2.0, -1.0);
    let v = Vector::new(1.0, 0.0, 1.0);
    let n = Normal::new(1.0, 0.0, 1.0);
    assert_eq!(t * p, p + Vector::new(1.0, 2.0, 3.0));
    assert_eq!(t * v, v);
    assert_eq!(t * n, n);
}
#[test]
fn test_scale() {
    let t = Transform::scale(&Vector::new(0.5, 0.1, 2.0));
    let p = Point::new(10.0, 20.0, 30.0);
    let v = Vector::new(10.0, 20.0, 30.0);
    let n = Normal::new(1.0, 2.0, 10.0);
    assert_eq!(t * p, Point::new(p.x * 0.5, p.y * 0.1, p.z * 2.0));
    assert_eq!(t * v, v * Vector::new(0.5, 0.1, 2.0));
    assert_eq!(t * n, n * Normal::new(2.0, 10.0, 0.5));
}
#[test]
fn test_rotate_x() {
    let t = Transform::rotate_x(90.0);
    let p = t * Point::new(0.0, 1.0, 0.0);
    let v = t * Vector::new(0.0, 1.0, 0.0);
    let n = t * Normal::new(0.0, 1.0, 0.0);
    // Need to now deal with some floating annoyances in these tests
    assert_eq!(p.x, 0.0);
    assert_eq!(Float::abs_sub(p.y, 0.0), 0.0);
    assert_eq!(p.z, 1.0);

    assert_eq!(v.x, 0.0);
    assert_eq!(Float::abs_sub(v.y, 0.0), 0.0);
    assert_eq!(v.z, 1.0);

    assert_eq!(n.x, 0.0);
    assert_eq!(Float::abs_sub(n.y, 0.0), 0.0);
    assert_eq!(n.z, 1.0);
}
#[test]
fn test_rotate_y() {
    let t = Transform::rotate_y(-90.0);
    let p = t * Point::new(1.0, 0.0, 0.0);
    let v = t * Vector::new(1.0, 0.0, 0.0);
    let n = t * Normal::new(1.0, 0.0, 0.0);
    // Need to now deal with some floating annoyances in these tests
    assert_eq!(Float::abs_sub(p.x, 0.0), 0.0);
    assert_eq!(p.y, 0.0);
    assert_eq!(p.z, 1.0);

    assert_eq!(Float::abs_sub(v.x, 0.0), 0.0);
    assert_eq!(v.y, 0.0);
    assert_eq!(v.z, 1.0);

    assert_eq!(Float::abs_sub(n.x, 0.0), 0.0);
    assert_eq!(n.y, 0.0);
    assert_eq!(n.z, 1.0);
}
#[test]
fn test_rotate_z() {
    let t = Transform::rotate_z(90.0);
    let p = t * Point::new(1.0, 0.0, 0.0);
    let v = t * Vector::new(1.0, 0.0, 0.0);
    let n = t * Normal::new(1.0, 0.0, 0.0);
    // Need to now deal with some floating annoyances in these tests
    assert_eq!(Float::abs_sub(p.x, 0.0), 0.0);
    assert_eq!(p.y, 1.0);
    assert_eq!(p.z, 0.0);

    assert_eq!(Float::abs_sub(v.x, 0.0), 0.0);
    assert_eq!(v.y, 1.0);
    assert_eq!(v.z, 0.0);

    assert_eq!(Float::abs_sub(n.x, 0.0), 0.0);
    assert_eq!(n.y, 1.0);
    assert_eq!(n.z, 0.0);
}
#[test]
fn test_rotate() {
    assert_eq!(Transform::rotate(&Vector::new(1.0, 0.0, 0.0), 32.0),
                Transform::rotate_x(32.0));
    assert_eq!(Transform::rotate(&Vector::new(0.0, 1.0, 0.0), 104.0),
                Transform::rotate_y(104.0));
    assert_eq!(Transform::rotate(&Vector::new(0.0, 0.0, 1.0), 243.0),
                Transform::rotate_z(243.0));
}

