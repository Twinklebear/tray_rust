//! The master module provides the Master struct which instructs Workers which
//! portions of the image they should render and collects their results to combine
//! into the final image.

use std::path::PathBuf;
use std::io::prelude::*;
use std::collections::HashMap;
use std::iter;

use bincode::rustc_serialize::decode;
use image;
use mio::tcp::{TcpStream, Shutdown};
use mio::*;

use film::Image;
use exec::Config;
use exec::distrib::{worker, Instructions};
use sampler::BlockQueue;

pub static PORT: u16 = 63245;

pub struct DistributedFrame {
    pub frame: usize,
    // The number of workers who have reported results for this
    // frame so far
    pub num_reporting: usize,
    pub render: Image,
    pub completed: bool,
}

#[derive(Clone, Debug)]
struct WorkerBuffer {
    pub buf: Vec<u8>,
    pub expected_size: usize,
    pub currently_read: usize,
}

impl WorkerBuffer {
    pub fn new() -> WorkerBuffer {
        WorkerBuffer { buf: Vec::new(), expected_size: 0, currently_read: 0 }
    }
}

pub struct Master {
    // Hostnames of the workers to send work too
    workers: Vec<String>,
    connections: Vec<TcpStream>,
    // Temporary buffers to store worker results in as they're
    // read in over TCP
    worker_buffers: Vec<WorkerBuffer>,
    config: Config,
    frames: HashMap<usize, DistributedFrame>,
    img_dim: (usize, usize),
}

impl Master {
    /// Create a new master that will contact the worker nodes passed and
    /// send instructions on what parts of the scene to start rendering
    pub fn start_workers(workers: Vec<String>, config: Config, scene_file: &String,
                         img_dim: (usize, usize), frames: Option<(usize, usize)>)
                         -> (Master, EventLoop<Master>) {
        // Figure out how many blocks we have for this image and assign them to our workers
        let queue = BlockQueue::new((img_dim.0 as u32, img_dim.1 as u32), (8, 8), (0, 0));
        let blocks_per_worker = queue.len() / workers.len();
        let blocks_remainder = queue.len() % workers.len();
        println!("Have {} workers, each worker does {} blocks w/ {} remainder",
                 workers.len(), blocks_per_worker, blocks_remainder);

        let mut event_loop = EventLoop::<Master>::new().unwrap();
        let mut connections = Vec::new();

        // Connect to each worker and send instructions on what to render
        for (i, host) in workers.iter().enumerate() {
            let b_start = i * blocks_per_worker;
            let b_count =
                if i == workers.len() - 1 {
                    blocks_per_worker + blocks_remainder
                } else {
                    blocks_per_worker
                };
            let instr = Instructions::new(scene_file, PORT, frames, b_start, b_count);
            println!("Sending instructions {:?} too {}", instr, host);
            let addr = format!("{}:{}", host, worker::PORT).parse().unwrap();
            match TcpStream::connect(&addr) {
                Ok(mut stream) => {
                    println!("Connected and writing");
                    let bytes = instr.to_json().into_bytes();
                    match stream.write_all(&bytes[..]) {
                        Err(e) => println!("Failed to send instructions to {}: {:?}", host, e),
                        _ => {},
                    }
                    // We no longer need to write anything, so close the write end
                    match stream.shutdown(Shutdown::Write) {
                        Err(e) => panic!(format!("Failed to shutdown write end to worker {}: {}", host, e)),
                        Ok(_) => {},
                    }
                    println!("Adding stream to event loop");
                    // Each worker is identified in the event loop by their index in the vec
                    match event_loop.register(&stream, Token(i)) {
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
                              frames: HashMap::new(), img_dim: img_dim };
        (master, event_loop)
    }
    /// Read a result frame from a worker and save it into the list of frames we're collecting from
    /// all workers. Will save out the final render if all workers have reported results for this
    /// frame.
    fn save_results(&mut self, frame: worker::Frame) {
        // Find the frame this worker is referring to and add its results
        let frame_num = frame.frame as usize;
        println!("Worker reporting frame {}", frame_num);
        if let Some(df) = self.frames.get_mut(&frame_num) {
            df.render.add_pixels(&frame.pixels);
            df.num_reporting += 1;
            println!("We have parts of this frame already");
        }
        // TODO: Better way here? We get stuck because self.frames is counted as borrowed
        // from the if above, making this really awkward.
        if self.frames.get_mut(&frame_num).is_none() {
            // TODO: I need to seperate the render target from the actual image as the
            // filter here isn't used in this path and it doesn't make sense to have one
            let render = Image::new(self.img_dim);
            let mut df = DistributedFrame { frame: frame_num, num_reporting: 1, render: render, completed: false };
            df.render.add_pixels(&frame.pixels);
            self.frames.insert(df.frame, df);
            println!("This is a new frame");
        }
        if let Some(df) = self.frames.get_mut(&frame_num) {
            if df.num_reporting == self.workers.len() {
                df.completed = true;
                println!("We have completed frame {}", frame_num);
                let out_file = match self.config.out_path.extension() {
                    Some(_) => self.config.out_path.clone(),
                    None => self.config.out_path.join(PathBuf::from(format!("frame{:05}.png", df.frame))),
                };
                println!("Frame {}: rendered to '{}'\n--------------------", frame_num, out_file.display());
                let img = df.render.get_srgb8();
                let dim = df.render.dimensions();
                match image::save_buffer(&out_file.as_path(), &img[..], dim.0 as u32, dim.1 as u32, image::RGB(8)) {
                    Ok(_) => {},
                    Err(e) => println!("Error saving image, {}", e),
                };
            }
        }
    }
    fn read_worker_buffer(&mut self, worker: usize) -> bool {
        let mut buf = &mut self.worker_buffers[worker];
        // If we haven't read the size of data being sent, read that now
        if buf.currently_read < 8 {
            // First 8 bytes are a u64 specifying the number of bytes being sent
            buf.buf.extend(iter::repeat(0u8).take(8));
            while buf.currently_read != 8 {
                println!("Reading size header...");
                match self.connections[worker].read(&mut buf.buf[buf.currently_read..]) {
                    Ok(n) => buf.currently_read += n,
                    Err(e) => println!("Error reading results from worker {}: {}", self.workers[worker], e),
                }
                println!("Read {} bytes so far, must read {} in total", buf.currently_read, 8);
            }
            // How many bytes we expect to get from the worker for a frame
            buf.expected_size = decode(&buf.buf[..]).unwrap();
            // Extend the Vec so we've got enough room for the remaning bytes, minus the 8 for the
            // encoded size header
            buf.buf.extend(iter::repeat(0u8).take(buf.expected_size - 8));
        }
        println!("Reading from worker");
        match self.connections[worker].read(&mut buf.buf[buf.currently_read..]) {
            Ok(n) => buf.currently_read += n,
            Err(e) => println!("Error reading results from worker {}: {}", self.workers[worker], e),
        }
        println!("Read {} bytes so far, must read {} in total", buf.currently_read, buf.expected_size);
        buf.currently_read == buf.expected_size
    }
}

impl Handler for Master {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<Master>, token: Token, event: EventSet) {
        // Some results are read for reading from a worker
        if event.is_readable() {
            let worker = token.as_usize();
            let finished = self.read_worker_buffer(worker);
            if finished {
                println!("Got all frame data, now parsing");
                let frame: worker::Frame = decode(&self.worker_buffers[worker].buf[..]).unwrap();
                println!("Saving frame results");
                self.save_results(frame);
                // Clean up our read buffer
                self.worker_buffers[worker].buf.clear();
                self.worker_buffers[worker].expected_size = 0;
                self.worker_buffers[worker].currently_read = 0;
            }
        }
        // After getting results from the worker we check if we've completed all our frames
        // and exit if so
        if !self.frames.is_empty() && self.frames.values().fold(true, |all, v| all && v.completed) {
            println!("All frames have been rendered, master exiting");
            event_loop.shutdown();
        }
    }
}

