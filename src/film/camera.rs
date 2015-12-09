//! Provides a camera based on a single transformation that positions
//! it in the scene
//!
//! # Scene Usage Example
//! The camera must specify information about its position in the world, the image dimensions
//! and the number of samples to take per pixel.
//!
//! ```json
//! "camera": {
//!     "width": 800,
//!     "height": 600,
//!     "samples" 512,
//!     "fov": 50.0
//!     "transform": [
//!         {
//!             "type": "translate",
//!             "translation": [0, 12, -60]
//!         }
//!     ]
//! }
//! ```

use linalg::{Transform, Vector, Point, Ray, AnimatedTransform};

/// Our camera for the ray tracer, has a transformation to position it in world space
#[derive(Clone)]
pub struct Camera {
    /// Transformation from camera to world space
    cam_world: AnimatedTransform,
    /// Transformation from raster to camera space
    raster_cam: Transform,
    /// Shutter open time for this frame
    shutter_open: f32,
    /// Shutter close time for this frame
    shutter_close: f32,
    /// Percentage of the shutter that is open to light. For example .5 is
    /// a standard 180 degree shutter
    shutter_size: f32,
}

impl Camera {
    /// Create the camera with some orientation in the world specified by `cam_world`
    /// and a perspective projection with `fov`. The render target dimensions `dims`
    /// are needed to construct the raster -> camera transform
    /// `animation` is used to move the camera ote that this is specified in camera space
    /// where the camera is at the origin looking down the -z axis
    pub fn new(cam_world: AnimatedTransform, fov: f32, dims: (usize, usize), shutter_size: f32) -> Camera {
        let aspect_ratio = (dims.0 as f32) / (dims.1 as f32);
        let screen =
            if aspect_ratio > 1.0 {
                [-aspect_ratio, aspect_ratio, -1.0, 1.0]
            } else {
                [-1.0, 1.0, -1.0 / aspect_ratio, 1.0 / aspect_ratio]
            };
        let screen_raster = Transform::scale(&Vector::new(dims.0 as f32, dims.1 as f32, 1.0))
            * Transform::scale(&Vector::new(1.0 / (screen[1] - screen[0]), 1.0 / (screen[2] - screen[3]), 1.0))
            * Transform::translate(&Vector::new(-screen[0], -screen[3], 0.0));
        let raster_screen = screen_raster.inverse();
        let cam_screen = Transform::perspective(fov, 1.0, 1000.0);
        Camera { cam_world: cam_world, raster_cam: cam_screen.inverse() * raster_screen,
                 shutter_open: 0.0, shutter_close: 0.0, shutter_size: shutter_size
        }
    }
    /// Update the camera's shutter open/close time for this new frame
    pub fn update_frame(&mut self, start: f32, end: f32) {
        self.shutter_open = start;
        self.shutter_close = start + self.shutter_size * (end - start);
        println!("Shutter open from {} to {}", self.shutter_open, self.shutter_close);
    }
    /// Generate a ray from the camera through the pixel `px`
    pub fn generate_ray(&self, px: &(f32, f32), time: f32) -> Ray {
        // Take the raster space position -> camera space
        let px_pos = self.raster_cam * Point::new(px.0, px.1, 0.0);
        let d = Vector::new(px_pos.x, px_pos.y, px_pos.z).normalized();
        // Compute the time being sampled for this frame based on shutter open/close times
        let frame_time = (self.shutter_close - self.shutter_open) * time + self.shutter_open;
        self.cam_world.transform(frame_time) * Ray::new(&Point::broadcast(0.0), &d, frame_time)
    }
}

