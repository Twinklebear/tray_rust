//! The master module provides the Master struct which instructs Workers which
//! portions of the image they should render and collects their results to combine
//! into the final image.

use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};
use std::collections::HashMap;

use bincode::rustc_serialize::decode;
use image;

use film::Image;
use exec::Config;
use exec::distrib::{worker, Instructions};
use sampler::BlockQueue;

static PORT: u16 = 64455;

pub struct DistributedFrame {
    pub frame: usize,
    // The number of workers who have reported results for this
    // frame so far
    pub num_reporting: usize,
    pub render: Image,
    pub completed: bool,
}

pub struct Master {
    // Hostnames of the workers to send work too
    workers: Vec<String>,
    config: Config,
    frames: HashMap<usize, DistributedFrame>,
    img_dim: (usize, usize),
    listener: TcpListener,
}

impl Master {
    /// Create a new master that will contact the worker nodes passed and
    /// send instructions on what parts of the scene to start rendering
    pub fn start_workers(workers: Vec<String>, config: Config, scene_file: &String,
                         img_dim: (usize, usize), frames: Option<(usize, usize)>) -> Master {
        // Figure out how many blocks we have for this image and assign them to our workers
        let queue = BlockQueue::new((img_dim.0 as u32, img_dim.1 as u32), (8, 8), (0, 0));
        let blocks_per_worker = queue.len() / workers.len();
        let blocks_remainder = queue.len() % workers.len();
        println!("Have {} workers, each worker does {} blocks w/ {} remainder",
                 workers.len(), blocks_per_worker, blocks_remainder);

        // Start listeneing before we launch any workers so in the case they finish immediately
        // they don't fail to connect
        let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("Master failed to get port");

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
            match TcpStream::connect((&host[..], worker::PORT)) {
                Ok(mut stream) => {
                    let bytes = instr.to_json().into_bytes();
                    match stream.write_all(&bytes[..]) {
                        Err(e) => println!("Failed to send instructions to {}: {:?}", host, e),
                        _ => {},
                    }
                },
                Err(e) => println!("Failed to contact worker {}: {:?}", host, e),
            }
        }
        Master { workers: workers, config: config, frames: HashMap::new(), img_dim: img_dim,
                 listener: listener }
    }
    /// Listen for frames sent back from workers and add the framebuffers together to
    /// build up the final render. Once we've got results from all the workers the image
    /// is saved out
    pub fn wait_for_results(&mut self) {
        println!("Master listening for workers");
        for s in self.listener.incoming() {
            match s {
                Ok(mut stream) => {
                    println!("Got connection from worker");
                    let mut bytes = Vec::new();
                    match stream.read_to_end(&mut bytes) {
                        Err(e) => panic!("Failed to read from worker, {:?}", e),
                        _ => {},
                    }
                    let frame: worker::Frame = decode(&bytes[..]).unwrap();
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
                },
                Err(e) => panic!("Error accepting a worker: {:?}", e),
            }
            // TODO: After getting results from the worker we should check if we've completed all
            // our frames and exit if so
            if self.frames.values().fold(true, |all, v| all && v.completed) {
                println!("All frames have been rendered, master exiting");
                return;
            }
        }
    }
}

