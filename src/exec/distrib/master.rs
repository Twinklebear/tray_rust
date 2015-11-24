//! The master module provides the Master struct which instructs Workers which
//! portions of the image they should render and collects their results to combine
//! into the final image.

use std::io::prelude::*;
use std::net::{TcpStream};

use film::RenderTarget;
use exec::Config;
use exec::distrib::{worker, Instructions};
use sampler::BlockQueue;

static PORT: u16 = 64455;

pub struct Master {
    // Hostnames of the workers to send work too
    workers: Vec<String>,
    render_target: RenderTarget,
    config: Config,
}

impl Master {
    /// Create a new master that will contact the worker nodes passed and
    /// send instructions on what parts of the scene to start rendering
    pub fn start_workers(workers: Vec<String>, config: Config, scene_file: &String,
                        rt: RenderTarget, frames: Option<(usize, usize)>) -> Master {
        let dim = rt.dimensions();
        // Figure out how many blocks we have for this image and assign them to our workers
        let queue = BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8), (0, 0));
        let blocks_per_worker = queue.len() / workers.len();
        let blocks_remainder = queue.len() % workers.len();
        println!("Have {} workers, each worker does {} blocks w/ {} remainder",
                 workers.len(), blocks_per_worker, blocks_remainder);
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
        Master { workers: workers, render_target: rt, config: config }
    }
}

