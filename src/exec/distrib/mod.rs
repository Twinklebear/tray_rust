//! The distrib module provides methods for executing the rendering in a
//! distributed environment across multiple nodes. The worker module provides
//! the Worker which does the actual job of rendering a subsection of the image.
//! The master module provides the Master which instructs the Workers and collects
//! their results.
//!
//! **Note:** At this time I do nothing for distributed fault handling.

use bincode::rustc_serialize::encoded_size;

pub use self::worker::Worker;
pub use self::master::Master;

pub mod worker;
pub mod master;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
struct Instructions {
    pub encoded_size: u64,
    pub scene: String,
    pub frames: (usize, usize),
    pub block_start: usize,
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

