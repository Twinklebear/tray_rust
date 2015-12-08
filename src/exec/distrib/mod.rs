//! The distrib module provides methods for executing the rendering in a
//! distributed environment across multiple nodes. The worker module provides
//! the Worker which does the actual job of rendering a subsection of the image.
//! The master module provides the Master which instructs the Workers and collects
//! their results.
//!
//! **Note:** At this time I do nothing for distributed fault handling or work
//! stealing. If a node crashes during rendering it's results will be lost and the
//! master will hang forever waiting to hear back from the crashed node.

use bincode::rustc_serialize::encoded_size;

pub use self::worker::Worker;
pub use self::master::Master;

pub mod worker;
pub mod master;

/// Stores instructions sent to a worker about which blocks it should be rendering,
/// block size is assumed to be 8x8
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct Instructions {
    /// Size header for binary I/O with bincode
    pub encoded_size: u64,
    /// Scene file for the worker to load
    pub scene: String,
    /// Frames to be rendered (inclusive)
    pub frames: (usize, usize),
    /// Block in the z-order queue of blocks this worker will
    /// start at
    pub block_start: usize,
    /// Number of blocks this worker will render
    pub block_count: usize,
}

impl Instructions {
    pub fn new(scene: &String, frames: (usize, usize), block_start: usize,
               block_count: usize) -> Instructions {
        let mut instr = Instructions { encoded_size: 0, scene: scene.clone(), frames: frames,
                       block_start: block_start, block_count: block_count };
        instr.encoded_size = encoded_size(&instr);
        instr
    }
}

/// Frame is used by the worker to send its results back to the master. Sends information
/// about which frame is being sent, which blocks were rendered and the data for the blocks
#[derive(RustcEncodable, RustcDecodable)]
struct Frame {
    /// Size header for binary I/O with bincode
    pub encoded_size: u64,
    /// Which frame the worker is sending its results for
    pub frame: usize,
    /// Block size of the blocks being sent
    pub block_size: (usize, usize),
    /// Starting locations of each block
    pub blocks: Vec<(usize, usize)>,
    /// Sample data for each block, RGBW_F32 (W = weight)
    pub pixels: Vec<f32>,
}

impl Frame {
    pub fn new(frame: usize, block_size: (usize, usize), blocks: Vec<(usize, usize)>,
               pixels: Vec<f32>) -> Frame {
        let mut frame = Frame { encoded_size: 0, frame: frame, block_size: block_size,
                            blocks: blocks, pixels: pixels };
        frame.encoded_size = encoded_size(&frame);
        frame
    }
}

