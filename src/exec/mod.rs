//! The exec module provides an abstraction of the execution backends
//! used to actually render the image

use std::path::PathBuf;

use film::{FrameInfo, RenderTarget};
use scene::Scene;

pub use self::multithreaded::MultiThreaded;

pub mod multithreaded;
#[cfg(unix)]
pub mod distrib;

/// Config passed to set up the execution environment with information
/// on what it should be rendering and where to put the results
#[derive(Debug, Clone)]
pub struct Config {
    // TODO: Maybe this should be Option
    pub out_path: PathBuf,
    pub num_threads: u32,
    pub spp: usize,
    pub frame_info: FrameInfo,
    pub current_frame: usize,
    /// Which blocks the executor should render, stored
    /// as (start, count) of the block indices
    pub select_blocks: (usize, usize)
}

impl Config {
    pub fn new(out_path: PathBuf, spp: usize, num_threads: u32, frame_info: FrameInfo,
               select_blocks: (usize, usize)) -> Config {
        Config { out_path: out_path, spp: spp, num_threads: num_threads, frame_info: frame_info,
                 current_frame: frame_info.start, select_blocks: select_blocks }
    }
}

/// Trait implemented by different execution environments that provides
/// a method to call and render the scene, given the rendering arguments
pub trait Exec {
    /// Render the scene using this rendering backend, will render out
    /// all frames of the image and save them out as instructed by
    /// the command line arguments
    /// TODO: In order to have a cleaner seperation we should pass more parameters
    /// to render. E.g. the scene. Or maybe a callback to a function that gets the
    /// frame's render target and can save it out?
    fn render(&mut self, scene: &mut Scene, rt: &mut RenderTarget, config: &Config);
}

