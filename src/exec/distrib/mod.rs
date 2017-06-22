//! The distrib module provides methods for executing the rendering in a
//! distributed environment across multiple machines. The worker module provides
//! the Worker which does the actual job of rendering a subregion of the image.
//! The master module provides the Master which instructs the Workers what to render
//! and collects their results to save out the final image.
//!
//! # Usage
//!
//! The worker process takes very few arguments, just a flag indicating it's a worker
//! and optionally the number of threads to use with `-n`.
//!
//! ```text
//! ./tray_rust --worker
//! ```
//!
//! The worker processes will listen on a hard-coded port for the master to send them instructions
//! about what parts of the image they should render. This is `exec::distrib::worker::PORT` which
//! you can change and re-compile if the default of 63234 conflicts with other applications.
//!
//! The master process can be run on the same machine as a worker since it doesn't take
//! up too much CPU time. To run the master you'll pass it the scene file, a list of the
//! worker hostnames or IP addresses and optionally an output path and start/end frame numbers.
//! You can also run tray\_rust with the `-h` or `--help` flag to see a list of options.
//!
//! ```text
//! ./tray_rust cornell_box.json --master worker1 worker2 192.168.32.129
//! ```
//!
//! The master will send the workers the location of the scene file which is assumed to
//! be on some shared filesystem or otherwise available at the same path on all the workers.
//!
//! # Running on GCE or EC2
//!
//! You can run on any network of home machines but you can also run on virtual machines from
//! Google or Amazon if you want to rent a mini cluster. On GCE or EC2 you'll want machines in the
//! same region for faster communication and will then pass the local IPs of the workers to
//! the master. For example on GCE you're given a virtual local network, you would use
//! these IP addresses instead of the public IPs of the worker nodes.
//!

use bincode::serialized_size;

pub use self::worker::Worker;
pub use self::master::Master;

pub mod worker;
pub mod master;

/// Stores instructions sent to a worker about which blocks it should be rendering,
/// block size is assumed to be 8x8
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn new(scene: &str, frames: (usize, usize), block_start: usize,
               block_count: usize) -> Instructions {
        let mut instr = Instructions { encoded_size: 0, scene: scene.to_owned(), frames: frames,
                       block_start: block_start, block_count: block_count };
        instr.encoded_size = serialized_size(&instr);
        instr
    }
}

/// Frame is used by the worker to send its results back to the master. Sends information
/// about which frame is being sent, which blocks were rendered and the data for the blocks
#[derive(Serialize, Deserialize)]
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
        frame.encoded_size = serialized_size(&frame);
        frame
    }
}

