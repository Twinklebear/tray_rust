//! The master module provides the Master struct which instructs Workers which
//! portions of the image they should render and collects their results to combine
//! into the final image.

use std::path::PathBuf;
use std::io::prelude::*;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::iter;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use image;
use mio::tcp::{TcpStream, Shutdown};
use mio::*;

use film::Image;
use exec::Config;
use exec::distrib::{worker, Instructions, Frame};
use sampler::BlockQueue;

/// Stores distributed rendering status. The frame is either InProgress and contains
/// partially rendered results from the workers who've reported the frame or is Completed
/// and has been saved out to disk.
#[derive(Debug)]
enum DistributedFrame {
    InProgress {
        // Which frame number this is
        frame: usize,
        // The number of workers who have reported results for this
        // frame so far
        num_reporting: usize,
        render: Image,
    },
    Completed,
}

impl DistributedFrame {
    pub fn start(frame_num: usize, img_dim: (usize, usize)) -> DistributedFrame {
        DistributedFrame::InProgress { frame: frame_num, num_reporting: 0, render: Image::new(img_dim) }
    }
}

/// Buffer for collecting results from a worker asynchronously. The buffer is filled
/// as we get readable events from the workers until it reaches the expected size.
/// After this the Frame is decoded and accumulated in the appropriate DistributedFrame
#[derive(Clone, Debug)]
struct WorkerBuffer {
    pub buf: Vec<u8>,
    pub expected_size: usize,
    pub currently_read: usize,
}

impl WorkerBuffer {
    pub fn new() -> WorkerBuffer {
        WorkerBuffer { buf: Vec::new(), expected_size: 8, currently_read: 0 }
    }
}

/// The Master organizes the set of Worker processes and instructions them what parts
/// of the scene to render. As workers report results the master collects them and
/// saves out the PNG once all workers have reported the frame.
pub struct Master {
    /// Hostnames of the workers to send work too
    workers: Vec<String>,
    connections: Vec<TcpStream>,
    /// Temporary buffers to store worker results in as they're
    /// read in over TCP
    worker_buffers: Vec<WorkerBuffer>,
    config: Config,
    /// List of the frames we're collecting or have completed
    frames: HashMap<usize, DistributedFrame>,
    img_dim: (usize, usize),
    /// Number of 8x8 blocks we're assigning per worker
    blocks_per_worker: usize,
    /// Remainder of blocks that will be tacked on to the last
    /// worker's assignment
    blocks_remainder: usize,
}

impl Master {
    /// Create a new master that will contact the worker nodes passed and
    /// send instructions on what parts of the scene to start rendering
    pub fn start_workers(workers: Vec<String>, config: Config, img_dim: (usize, usize))
                         -> (Master, EventLoop<Master>) {
        // Figure out how many blocks we have for this image and assign them to our workers
        let queue = BlockQueue::new((img_dim.0 as u32, img_dim.1 as u32), (8, 8), (0, 0));
        let blocks_per_worker = queue.len() / workers.len();
        let blocks_remainder = queue.len() % workers.len();

        let mut event_loop = EventLoop::<Master>::new().unwrap();
        let mut connections = Vec::new();

        // Connect to each worker and add them to the event loop
        for (i, host) in workers.iter().enumerate() {
            let addr = (&host[..], worker::PORT).to_socket_addrs().unwrap().next().unwrap();
            match TcpStream::connect(&addr) {
                Ok(stream) => {
                    // Each worker is identified in the event loop by their index in the vec
                    match event_loop.register(&stream, Token(i), EventSet::all(), PollOpt::level()){
                        Err(e) => println!("Error registering stream from {}: {}", host, e),
                        Ok(_) => {},
                    }
                    connections.push(stream);
                },
                Err(e) => println!("Failed to contact worker {}: {:?}", host, e),
            }
        }
        let worker_buffers: Vec<_> = iter::repeat(WorkerBuffer::new()).take(workers.len()).collect();
        let master = Master { workers: workers, connections: connections,
                              worker_buffers: worker_buffers, config: config,
                              frames: HashMap::new(),
                              img_dim: img_dim,
                              blocks_per_worker: blocks_per_worker,
                              blocks_remainder: blocks_remainder };
        (master, event_loop)
    }
    /// Read a result frame from a worker and save it into the list of frames we're collecting from
    /// all workers. Will save out the final render if all workers have reported results for this
    /// frame.
    fn save_results(&mut self, frame: Frame) {
        let frame_num = frame.frame as usize;
        let img_dim = self.img_dim;
        // Find the frame being reported and create it if we haven't received parts of this frame yet
        let mut df = self.frames.entry(frame_num).or_insert_with(
                        || DistributedFrame::start(frame_num, img_dim));

        let mut finished = false;
        match df {
            &mut DistributedFrame::InProgress { frame: _, ref mut num_reporting, ref mut render } => {
                // Collect results from the worker and see if we've finished the frame and can save
                // it out
                render.add_blocks(frame.block_size, &frame.blocks, &frame.pixels);
                *num_reporting += 1;
                if *num_reporting == self.workers.len() {
                    let out_file = match self.config.out_path.extension() {
                        Some(_) => self.config.out_path.clone(),
                        None => self.config.out_path.join(
                            PathBuf::from(format!("frame{:05}.png", frame_num))),
                    };
                    let img = render.get_srgb8();
                    let dim = render.dimensions();
                    match image::save_buffer(&out_file.as_path(), &img[..], dim.0 as u32,
                    dim.1 as u32, image::RGB(8)) {
                        Ok(_) => {},
                        Err(e) => println!("Error saving image, {}", e),
                    };
                    println!("Frame {}: rendered to '{}'\n--------------------",
                             frame_num, out_file.display());
                    finished = true;
                }
            },
            &mut DistributedFrame::Completed => println!("Worker reporting on completed frame {}?", frame_num),
        }
        // This is a bit awkward, since we borrow df in the match we can't mark it finished in there
        if finished {
            *df = DistributedFrame::Completed;
        }
    }
    /// Read results from a worker and accumulate this data in its worker buffer. Returns true if
    /// we've read the data being sent and can decode the buffer
    fn read_worker_buffer(&mut self, worker: usize) -> bool {
        let mut buf = &mut self.worker_buffers[worker];
        // If we haven't read the size of data being sent, read that now
        if buf.currently_read < 8 {
            // First 8 bytes are a u64 specifying the number of bytes being sent
            buf.buf.extend(iter::repeat(0u8).take(8));
            match self.connections[worker].read(&mut buf.buf[buf.currently_read..]) {
                Ok(n) => buf.currently_read += n,
                Err(e) => println!("Error reading results from worker {}: {}", self.workers[worker], e),
            }
            if buf.currently_read == buf.expected_size {
                // How many bytes we expect to get from the worker for a frame
                buf.expected_size = decode(&buf.buf[..]).unwrap();
                // Extend the Vec so we've got enough room for the remaning bytes, minus the 8 for the
                // encoded size header
                buf.buf.extend(iter::repeat(0u8).take(buf.expected_size - 8));
            }
        }
        // If we've finished reading the size header we can now start reading the frame data
        if buf.currently_read >= 8 {
            match self.connections[worker].read(&mut buf.buf[buf.currently_read..]) {
                Ok(n) => buf.currently_read += n,
                Err(e) => println!("Error reading results from worker {}: {}", self.workers[worker], e),
            }
        }
        buf.currently_read == buf.expected_size
    }
}

impl Handler for Master {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Master>, token: Token, event: EventSet) {
        let worker = token.as_usize();
        if event.is_error() {
            println!("Error connecting too {}", self.workers[worker]);
            match self.connections[worker].shutdown(Shutdown::Both) {
                Err(e) => println!("Error shutting down worker {}: {}", worker, e),
                Ok(_) => {},
            }
            // Remove the connection from the event loop
            match event_loop.deregister(&self.connections[worker]) {
                Err(e) => println!("Error deregistering worker {}: {}", worker, e),
                Ok(_) => {},
            }
        }
        // If the worker has terminated, shutdown the read end of the connection
        if event.is_hup() {
            match self.connections[worker].shutdown(Shutdown::Both) {
                Err(e) => println!("Error shutting down worker {}: {}", worker, e),
                Ok(_) => {},
            }
            // Remove the connection from the event loop
            match event_loop.deregister(&self.connections[worker]) {
                Err(e) => println!("Error deregistering worker {}: {}", worker, e),
                Ok(_) => {},
            }
        }
        // A worker is ready to receive instructions from us
        if event.is_writable() {
            let b_start = worker * self.blocks_per_worker;
            let b_count =
                if worker == self.workers.len() - 1 {
                    self.blocks_per_worker + self.blocks_remainder
                } else {
                    self.blocks_per_worker
                };
            let instr = Instructions::new(&self.config.scene_file,
                                          (self.config.frame_info.start, self.config.frame_info.end),
                                          b_start, b_count);
            // Encode and send our instructions to the worker
            let bytes = encode(&instr, SizeLimit::Infinite).unwrap();
            match self.connections[worker].write_all(&bytes[..]) {
                Err(e) => println!("Failed to send instructions to {}: {:?}", self.workers[worker], e),
                Ok(_) => {},
            }
            // Register that we no longer care about writable events on this connection
            event_loop.reregister(&self.connections[worker], token,
                                  EventSet::readable() | EventSet::error() | EventSet::hup(),
                                  PollOpt::level()).expect("Re-registering failed");
        }
        // Some results are available from a worker
        if event.is_readable() {
            // Read results from the worker, if we've accumulated all the data being sent
            // decode and accumulate the frame
            if self.read_worker_buffer(worker) {
                let frame: Frame = decode(&self.worker_buffers[worker].buf[..]).unwrap();
                self.save_results(frame);
                // Clean up the worker buffer for the next frame
                self.worker_buffers[worker].buf.clear();
                self.worker_buffers[worker].expected_size = 8;
                self.worker_buffers[worker].currently_read = 0;
            }
        }
        // After getting results from the worker we check if we've completed all our frames
        // and exit if so
        let all_complete = self.frames.values().fold(true,
                                |all, v| {
                                    match v {
                                        &DistributedFrame::Completed => true && all,
                                        _ => false,
                                    }
                                });
        // The frame start/end range is inclusive, so we must add 1 here
        let num_frames = self.config.frame_info.end - self.config.frame_info.start + 1;
        if self.frames.len() == num_frames && all_complete {
            event_loop.shutdown();
        }
    }
}

