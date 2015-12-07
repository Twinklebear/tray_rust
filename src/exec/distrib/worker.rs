//! The worker module provides the Worker struct which receives instructions from
//! the master, renders and reports back its results

use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::iter;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, encoded_size, decode};

use scene::Scene;
use film::RenderTarget;
use exec::Config;
use exec::distrib::Instructions;

pub static PORT: u16 = 63234;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Frame {
    pub encoded_size: u64,
    pub frame: usize,
    pub block_size: (usize, usize),
    pub blocks: Vec<(usize, usize)>,
    pub pixels: Vec<f32>,
}

pub struct Worker {
    instructions: Instructions,
    pub render_target: RenderTarget,
    pub scene: Scene,
    pub config: Config,
    master: TcpStream,
}

impl Worker {
    /// Listen on the worker `PORT` for the master to contact us
    /// and send us instructions about the scene we should render and
    /// what parts of it we've been assigned
    pub fn listen_for_master(num_threads: u32) -> Worker {
        let (instructions, master) = get_instructions();
        let (scene, rt, spp, mut frame_info) = Scene::load_file(&instructions.scene);
        frame_info.start = instructions.frames.0;
        frame_info.end = instructions.frames.1;
        let config = Config::new(PathBuf::from("/tmp"), instructions.scene.clone(), spp,
                                 num_threads, frame_info,
                                 (instructions.block_start, instructions.block_count));
        Worker { instructions: instructions, render_target: rt, scene: scene,
                 config: config, master: master }
    }
    /// Send our blocks back to the master
    pub fn send_results(&mut self) {
        println!("Sending results to master, {:?}", self.master);
        let (block_size, blocks, pixels) = self.render_target.get_rendered_blocks();
        // TODO: A ctor that computes the encoded size for us would be cleaner and easier
        let mut frame = Frame { encoded_size: 0, frame: self.config.current_frame,
                                block_size: block_size, blocks: blocks, pixels: pixels };
        frame.encoded_size = encoded_size(&frame);
        let bytes = encode(&frame, SizeLimit::Infinite).unwrap();
        println!("worker sending {} bytes", bytes.len());
        match self.master.write_all(&bytes[..]) {
            Err(e) => println!("Failed to send frame to {:?}: {}", self.master, e),
            _ => {},
        }
    }
}

fn get_instructions() -> (Instructions, TcpStream) {
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("Worker failed to get port");
    println!("Worker listening for master");
    match listener.accept() {
        Ok((mut stream, sock)) => {
            println!("Got connection");
            let mut buf: Vec<_> = iter::repeat(0u8).take(8).collect();
            let mut expected_size = 8;
            let mut currently_read = 0;
            // Read the size header
            while currently_read < expected_size {
                match stream.read(&mut buf[currently_read..]) {
                    Ok(n) => currently_read += n,
                    Err(e) => panic!("Failed to read from master, {:?}", e),
                }
            }
            // How many bytes we expect to get from the worker for a frame
            expected_size = decode(&buf[..]).unwrap();
            println!("Read size header of {} bytes, now reading remainder", expected_size);
            buf.extend(iter::repeat(0u8).take(expected_size - 8));
            // Now read the rest
            while currently_read < expected_size {
                match stream.read(&mut buf[currently_read..]) {
                    Ok(n) => currently_read += n,
                    Err(e) => panic!("Failed to read from master, {:?}", e),
                }
            }
            let instr: Instructions = decode(&buf[..]).unwrap();
            println!("Read from master {:?} instructions {:?}", sock, instr);
            (instr, stream)
        },
        Err(e) => panic!("Error accepting: {:?}", e),
    }
}

