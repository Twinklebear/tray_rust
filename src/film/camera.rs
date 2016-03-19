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

use bspline::BSpline;
use linalg::{self, Transform, Vector, Point, Ray, AnimatedTransform, Matrix4};

#[derive(Clone, Debug)]
enum CameraFov {
    Unanimated(f32),
    Animated(BSpline<f32>),
}

/// Our camera for the ray tracer, has a transformation to position it in world space
#[derive(Clone, Debug)]
pub struct Camera {
    /// Transformation from camera to world space
    cam_world: AnimatedTransform,
    /// Transformation from raster space to screen space
    raster_screen: Transform,
    /// The projective division matrix, the perspective matrix is changing in the
    /// case of animated FOV so we deconstruct it some to reduce creating a new
    /// transform each time
    proj_div_inv: Transform,
    /// Shutter open time for this frame
    shutter_open: f32,
    /// Shutter close time for this frame
    shutter_close: f32,
    /// Percentage of the shutter that is open to light. For example .5 is
    /// a standard 180 degree shutter
    shutter_size: f32,
    /// Animation points for the field of view
    fov: CameraFov,
    /// Scaling for the fov part of the projection matrix for the frame
    scaling: Vector,
    /// The frame this camera becomes active on
    pub active_at: usize,
}

impl Camera {
    /// Create the camera with some orientation in the world specified by `cam_world`
    /// and a perspective projection with `fov`. The render target dimensions `dims`
    /// are needed to construct the raster -> camera transform
    /// `animation` is used to move the camera ote that this is specified in camera space
    /// where the camera is at the origin looking down the -z axis
    pub fn new(cam_world: AnimatedTransform, fov: f32, dims: (usize, usize), shutter_size: f32, active_at: usize)
        -> Camera {
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
        let far = 1.0;
        let near = 1000.0;
        let proj_div = Matrix4::new(
            [1.0, 0.0, 0.0, 0.0,
             0.0, 1.0, 0.0, 0.0,
             0.0, 0.0, far / (far - near), -far * near / (far - near),
             0.0, 0.0, 1.0, 0.0]);
        let tan_fov = f32::tan(linalg::to_radians(fov) / 2.0);
        let scaling = Vector::new(tan_fov, tan_fov, 1.0);
        Camera { cam_world: cam_world, raster_screen: raster_screen,
                 proj_div_inv: Transform::from_mat(&proj_div).inverse(),
                 shutter_open: 0.0, shutter_close: 0.0, shutter_size: shutter_size,
                 fov: CameraFov::Unanimated(fov), scaling: scaling, active_at: active_at
        }
    }
    /// Create a camera with some orientation in the world specified by `cam_world`
    /// and an animated perspective projection with `fov`. The render target dimensions `dims`
    /// are needed to construct the raster -> camera transform
    /// `animation` is used to move the camera ote that this is specified in camera space
    /// where the camera is at the origin looking down the -z axis
    pub fn animated_fov(cam_world: AnimatedTransform, fovs: Vec<f32>, fov_knots: Vec<f32>, fov_spline_degree: usize,
                        dims: (usize, usize), shutter_size: f32, active_at: usize) -> Camera {
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
        let far = 1.0;
        let near = 1000.0;
        let proj_div = Matrix4::new(
            [1.0, 0.0, 0.0, 0.0,
             0.0, 1.0, 0.0, 0.0,
             0.0, 0.0, far / (far - near), -far * near / (far - near),
             0.0, 0.0, 1.0, 0.0]);
        let tan_fov = f32::tan(linalg::to_radians(fovs[0]) / 2.0);
        let scaling = Vector::new(tan_fov, tan_fov, 1.0);
        Camera { cam_world: cam_world, raster_screen: raster_screen,
                 proj_div_inv: Transform::from_mat(&proj_div).inverse(),
                 shutter_open: 0.0, shutter_close: 0.0, shutter_size: shutter_size,
                 fov: CameraFov::Animated(BSpline::new(fov_spline_degree, fovs, fov_knots)),
                 scaling: scaling, active_at: active_at
        }
    }
    /// Update the camera's shutter open/close time for this new frame
    pub fn update_frame(&mut self, start: f32, end: f32) {
        self.shutter_open = start;
        self.shutter_close = start + self.shutter_size * (end - start);
        // TODO: Is this the right spot to update the projection transform? It seems like
        // you'd want to do it for each ray but this produces some very odd results, maybe
        // resulting from different rays have different projection transformations?
        let fov = match self.fov {
            CameraFov::Unanimated(f) => f,
            CameraFov::Animated(ref spline) => {
                let domain = spline.knot_domain();
                let t = linalg::clamp((start + end) / 2.0, domain.0, domain.1);
                spline.point(t)
            },
        };
        let tan_fov = f32::tan(linalg::to_radians(fov) / 2.0);
        self.scaling = Vector::new(tan_fov, tan_fov, 1.0);
        println!("Shutter open from {} to {}", self.shutter_open, self.shutter_close);
    }
    /// Get the time that the shutter opens and closes at
    pub fn shutter_time(&self) -> (f32, f32) {
        (self.shutter_open, self.shutter_close)
    }
    /// Generate a ray from the camera through the pixel `px`
    pub fn generate_ray(&self, px: &(f32, f32), time: f32) -> Ray {
        // Take the raster space position -> camera space
        let px_pos = self.scaling * (self.proj_div_inv * self.raster_screen * Point::new(px.0, px.1, 0.0));
        let d = Vector::new(px_pos.x, px_pos.y, px_pos.z).normalized();
        // Compute the time being sampled for this frame based on shutter open/close times
        let frame_time = (self.shutter_close - self.shutter_open) * time + self.shutter_open;
        self.cam_world.transform(frame_time) * Ray::new(&Point::broadcast(0.0), &d, frame_time)
    }
}

