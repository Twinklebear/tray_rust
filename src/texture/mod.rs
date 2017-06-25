//! Defines the trait implemented by all textured values

/// Any T which can be copied to return can be computed
/// based on some texture or procedural value.
pub trait Texture<T: Copy> {
    /// Sample the textured value at texture coordinates u,v
    /// at some time. u and v should be in [0, 1]
    fn sample(&self, u: f32, v: f32, time: f32) -> T;
}

/// A single valued, solid 'color' texture for any T
pub struct Constant<T: Copy> {
    val: T,
}
impl<T: Copy> Constant<T> {
    pub fn new(val: T) -> Constant<T> {
        Constant { val: val }
    }
}
impl<T: Copy> Texture<T> for Constant<T> {
    fn sample(&self, _: f32, _: f32, _: f32) -> T {
        self.val
    }
}

