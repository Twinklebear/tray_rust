#![feature(duration_span, drain)]

extern crate image;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;
extern crate thread_scoped;
extern crate tray_rust;

use std::vec::Vec;
use std::sync::mpsc::{self, Sender, Receiver};
use std::path::Path;
use std::time::Duration;

use rand::StdRng;
use docopt::Docopt;

use tray_rust::film;
use tray_rust::geometry::{Geometry, Instance, Emitter};
use tray_rust::sampler::{self, Sampler};
use tray_rust::scene;
use tray_rust::integrator::Integrator;

static USAGE: &'static str = "
Usage: tray_rust <scenefile> [options]

Options:
  -o <file>     Specify the output file to save the image. Supported formats are
                PNG, JPG and PPM. Default is 'out.png'.
  -n <number>   Specify the number of threads to use for rendering. Defaults to the number of cores
                on the system.
  -h, --help    Show this message.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_scenefile: String,
    flag_o: Option<String>,
    flag_n: Option<u32>,
}

/// A struct containing results of an image sample where a ray was fired through
/// continuous pixel coordinates [x, y] and color `color` was computed
struct ImageSample {
    x: f32,
    y: f32,
    color: film::Colorf,
}

impl ImageSample {
    pub fn new(x: f32, y: f32, color: film::Colorf) -> ImageSample {
        ImageSample { x: x, y: y, color: color }
    }
}

/// Threads are each sent a sender end of the channel that is
/// read from by the render target thread which then saves the
/// values recieved to the render target
fn thread_work(tx: Sender<Vec<ImageSample>>, spp: usize, queue: &sampler::BlockQueue,
               scene: &scene::Scene, light_list: &Vec<&Emitter>) {
    let mut sampler = sampler::LowDiscrepancy::new(queue.block_dim(), spp);
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    let mut rng = match StdRng::new() {
        Ok(r) => r,
        Err(e) => { println!("Failed to get StdRng, {}", e); return }
    };
    // Grab a block from the queue and start working on it, submitting samples
    // to the render target thread after each pixel
    for b in queue.iter() {
        sampler.select_block(b);
        while sampler.has_samples() {
            // Get samples for a pixel and render them
            sampler.get_samples(&mut sample_pos, &mut rng);
            let mut samples = Vec::with_capacity(sampler.max_spp());
            for s in sample_pos.iter() {
                let mut ray = scene.camera.generate_ray(s);
                if let Some(hit) = scene.intersect(&mut ray) {
                    let c = scene.integrator.illumination(scene, light_list, &ray,
                                                          &hit, &mut sampler, &mut rng).clamp();
                    samples.push(ImageSample::new(s.0, s.1, c));
                }
            }
            if let Err(e) = tx.send(samples) {
                println!("Worker thread exiting with send error {:?}", e);
                return;
            }
        }
    }
}

/// Spawn `n` worker threads to render the scene in parallel. Returns the receive end
/// of the channel where the threads will write their samples so that the receiver
/// can write these samples to the render target
fn spawn_workers<'a>(n: u32, spp: usize, scene: &'a scene::Scene,
                     queue: &'a sampler::BlockQueue, light_list: &'a Vec<&Emitter>)
                 -> (Receiver<Vec<ImageSample>>, Vec<thread_scoped::JoinGuard<'a, ()>>) {
    let (tx, rx) = mpsc::channel();
    let mut guards = Vec::new();
    for _ in 0..n {
        let t = tx.clone();
        unsafe {
            guards.push(thread_scoped::scoped(move || {
                thread_work(t, spp, queue, scene, light_list);
            }));
        }
    }
    (rx, guards)
}

/// Render the scene in parallel to the render target
fn render_parallel(rt: &mut film::RenderTarget, scene: &scene::Scene, n: u32, spp: usize){
    let dim = rt.dimensions();
    let block_queue = sampler::BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8));
    let light_list: Vec<_> = scene.bvh.into_iter().filter_map(|x| {
        match x {
            &Instance::Emitter(ref e) => Some(e),
            _ => None,
        }
    }).collect();
    assert!(!light_list.is_empty(), "At least one light is required");
    {
        println!("Rendering using {} threads", n);
        let (rx, guards) = spawn_workers(n, spp, &scene, &block_queue, &light_list);
        for mut v in rx.iter() {
            for s in v.drain(..) {
                rt.write(s.x, s.y, &s.color);
            }
        }
        for g in guards {
            g.join();
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let (scene, spp, (width, height)) = scene::Scene::load_file(&args.arg_scenefile[..]);
    let mut rt = film::RenderTarget::new(width, height);
    let n = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };

    let d = Duration::span(|| render_parallel(&mut rt, &scene, n, spp));
    let time = d.as_secs() as f64 + (d.subsec_nanos() as f64) / 1_000_000_000.0;
    println!("Rendering took {}s", time);

    let img = rt.get_render();
    let out_file = match args.flag_o {
        Some(f) => f,
        None => "out.png".to_string(),
    };

    match image::save_buffer(&Path::new(&out_file), &img[..], width as u32, height as u32, image::RGB(8)) {
        Ok(_) => {},
        Err(e) => println!("Error saving image, {}", e),
    };
}

