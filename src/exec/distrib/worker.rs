//! The worker module provides the Worker struct which receives instructions from
//! the master, renders and reports back its results

use std::path::PathBuf;
use std::io::prelude::*;
use std::net::TcpListener;

use scene::Scene;
use film::RenderTarget;
use exec::{Config, MultiThreaded};
use exec::distrib::Instructions;

pub static PORT: u16 = 63234;

pub struct Worker {
    instructions: Instructions,
    render_target: RenderTarget,
    scene: Scene,
    config: Config,
    exec: MultiThreaded,
}

impl Worker {
    /// Listen on the worker `PORT` for the master to contact us
    /// and send us instructions about the scene we should render and
    /// what parts of it we've been assigned
    pub fn listen_for_master(num_threads: u32) -> Worker {
        let instructions = get_instructions();
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
                 config: config, exec: MultiThreaded::new(num_threads) }
    }
}

fn get_instructions() -> Instructions {
    let listener = TcpListener::bind(("0.0.0.0", PORT)).expect("Worker failed to get port");
    println!("Worker listening for master");
    match listener.accept() {
        Ok((mut stream, sock)) => {
            let mut read = String::new();
            match stream.read_to_string(&mut read) {
                Err(e) => panic!("Failed to read from master, {:?}", e),
                _ => {},
            }
            println!("Read from master {:?} content {}", sock, read);
            Instructions::from_json(read)
        },
        Err(e) => panic!("Error accepting: {:?}", e),
    }
}

