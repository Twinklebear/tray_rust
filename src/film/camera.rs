//! Provides a camera based on a single transformation that positions
//! it in the scene

use linalg::{Transform, Vector, Point, Ray};

/// Our camera for the ray tracer, has a transformation to position it in world space
#[derive(Copy, Clone)]
pub struct Camera {
    /// Transformation from camera to world space
    cam_world: Transform,
    /// Transformation from raster to camera space
    raster_cam: Transform,
}

impl Camera {
    /// Create the camera with some orientation in the world specified by `cam_world`
    /// and a perspective projection with `fov`. The render target dimensions `dims`
    /// are needed to construct the raster -> camera transform
    pub fn new(cam_world: Transform, fov: f32, dims: (usize, usize)) -> Camera {
        let aspect_ratio = (dims.0 as f32) / (dims.1 as f32);
        let screen =
            if aspect_ratio > 1.0 {
                [-aspect_ratio, aspect_ratio, -1.0, 1.0]
            } else {
                [-1.0, 1.0, -1.0 / aspect_ratio, -1.0 / aspect_ratio]
            };
        let screen_raster = Transform::scale(&Vector::new(dims.0 as f32, dims.1 as f32, 1.0))
            * Transform::scale(&Vector::new(1.0 / (screen[1] - screen[0]), 1.0 / (screen[2] - screen[3]), 1.0))
            * Transform::translate(&Vector::new(-screen[0], -screen[3], 0.0));
        let raster_screen = screen_raster.inverse();
        let cam_screen = Transform::perspective(fov, 1.0, 1000.0);
        Camera { cam_world: cam_world, raster_cam: cam_screen.inverse() * raster_screen }
    }
    /// Generate a ray from the camera through the pixel `px`
    pub fn generate_ray(&self, px: &(f32, f32)) -> Ray {
        // Take the raster space position -> camera space
        let px_pos = self.raster_cam * Point::new(px.0, px.1, 0.0);
        let d = Vector::new(px_pos.x, px_pos.y, px_pos.z).normalized();
        self.cam_world * Ray::new(&Point::broadcast(0.0), &d)
    }
}

