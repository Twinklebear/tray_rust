//! The worker module provides the Worker struct which receives instructions from
//! the master, renders and reports back its results

use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpListener, SocketAddr, TcpStream};

use bincode::SizeLimit;
use bincode::rustc_serialize::encode;

use scene::Scene;
use film::RenderTarget;
use exec::Config;
use exec::distrib::Instructions;

pub static PORT: u16 = 63234;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Frame {
    pub frame: u64,
    pub pixels: Vec<f32>,
}

pub struct Worker {
    instructions: Instructions,
    pub render_target: RenderTarget,
    pub scene: Scene,
    pub config: Config,
    master: SocketAddr,
}

impl Worker {
    /// Listen on the worker `PORT` for the master to contact us
    /// and send us instructions about the scene we should render and
    /// what parts of it we've been assigned
    pub fn listen_for_master(num_threads: u32) -> Worker {
        let (instructions, master) = get_instructions();
        let (scene, rt, spp, mut frame_info) = Scene::load_file(&instructions.scene);
        match instructions.frames {
            Some((start, end)) => {
                frame_info.start = start;
                frame_info.end = end;
            },
            None => {},
        }
        let config = Config::new(PathBuf::from("/tmp"), spp, num_threads, frame_info,
                                 (instructions.block_start, instructions.block_count));
        Worker { instructions: instructions, render_target: rt, scene: scene,
                 config: config, master: master }
    }
    /// Send our blocks back to the master
    pub fn send_results(&self) {
        println!("Sending results to master, {:?}", self.master);
        let frame = Frame { frame: self.config.current_frame as u64, pixels: self.render_target.get_renderf32() };
        match TcpStream::connect(self.master) {
            Ok(mut stream) => {
                let bytes = encode(&frame, SizeLimit::Infinite).unwrap();
                match stream.write_all(&bytes[..]) {
                    Err(e) => println!("Failed to send frame to {:?}: {}", self.master, e),
                    _ => {},
                }
            },
            Err(e) => println!("Failed to connect to master: {}", e),
        }
    }
}

fn get_instructions() -> (Instructions, SocketAddr) {
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("Worker failed to get port");
    println!("Worker listening for master");
    match listener.accept() {
        Ok((mut stream, sock)) => {
            let master = stream.peer_addr().unwrap();
            let mut read = String::new();
            match stream.read_to_string(&mut read) {
                Err(e) => panic!("Failed to read from master, {:?}", e),
                _ => {},
            }
            println!("Read from master {:?} content {}", sock, read);
            let instr = Instructions::from_json(read);
            let master_port = instr.master_port;
            (instr, SocketAddr::new(master.ip(), master_port))
        },
        Err(e) => panic!("Error accepting: {:?}", e),
    }
}

