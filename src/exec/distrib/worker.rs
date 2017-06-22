//! The worker module provides the Worker struct which receives instructions from
//! the master, renders and reports back its results

use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::iter;

use bincode::{Infinite, serialize, deserialize};

use scene::Scene;
use film::RenderTarget;
use exec::Config;
use exec::distrib::{Instructions, Frame};

/// Port that the workers listen for the master on
pub static PORT: u16 = 63234;

/// A worker process for distributed rendering. Accepts instructions from
/// the master process telling it what to render, after each frame is finished
/// results are sent back to the master and the next frame is started. Once all
/// frames are finished the worker exits
pub struct Worker {
    instructions: Instructions,
    /// Render target the worker will write the current frame too
    pub render_target: RenderTarget,
    pub scene: Scene,
    pub config: Config,
    /// Our connection to the master
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
        let (block_size, blocks, pixels) = self.render_target.get_rendered_blocks();
        let frame = Frame::new(self.config.current_frame, block_size, blocks, pixels);
        let bytes = serialize(&frame, Infinite).unwrap();
        if let Err(e) = self.master.write_all(&bytes[..]) {
            panic!("Failed to send frame to {:?}: {}", self.master, e);
        }
    }
}

fn get_instructions() -> (Instructions, TcpStream) {
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("Worker failed to get port");
    println!("Worker listening for master on {}", PORT);
    match listener.accept() {
        Ok((mut stream, _)) => {
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
            expected_size = deserialize(&buf[..]).unwrap();
            buf.extend(iter::repeat(0u8).take(expected_size - 8));
            // Now read the rest
            while currently_read < expected_size {
                match stream.read(&mut buf[currently_read..]) {
                    Ok(n) => currently_read += n,
                    Err(e) => panic!("Failed to read from master, {:?}", e),
                }
            }
            let instr = deserialize(&buf[..]).unwrap();
            println!("Received instructions: {:?}", instr);
            (instr, stream)
        },
        Err(e) => panic!("Error accepting: {:?}", e),
    }
}

