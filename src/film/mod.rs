//! The film module provides color types and a render target that the image
//! is written too.

pub use self::color::Colorf;
pub use self::render_target::RenderTarget;
pub use self::camera::Camera;
pub use self::render_target::ImageSample;
pub use self::animated_color::{ColorKeyframe, AnimatedColor};

pub mod color;
pub mod render_target;
pub mod camera;
pub mod filter;
pub mod animated_color;

/// Struct to store various parameters for the frame timing
#[derive(Debug, Copy, Clone)]
pub struct FrameInfo {
    /// Total number of frames for the scene
    pub frames: usize,
    /// Total time for all the entire scene (ie. for all frames)
    pub time: f32,
    /// Frame number to start rendering at
    pub start: usize,
    /// Frame number to stop rendering at
    pub end: usize,
}

impl FrameInfo {
    pub fn new(frames: usize, time: f32, start: usize, end: usize) -> FrameInfo {
        FrameInfo { frames: frames, time: time, start: start, end: end }
    }
}

