#![feature(duration, duration_span, drain)]

extern crate image;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize as rustc_serialize;
extern crate threadpool;
extern crate num_cpus;
extern crate tray_rust;

use std::vec::Vec;
use std::sync::mpsc::{self, Sender, Receiver};
use std::path::Path;
use std::time::Duration;

use threadpool::ScopedPool;
use rand::StdRng;
use docopt::Docopt;

use tray_rust::film;
use tray_rust::geometry::{Geometry, Instance, Emitter};
use tray_rust::sampler::{self, Sampler};
use tray_rust::scene;
use tray_rust::integrator::Integrator;

static WIDTH: usize = 800;
static HEIGHT: usize = 600;
static USAGE: &'static str = "
Usage: tray_rust [options]

Options:
  -o <file>     Specify the output file to save the image. Supported formats are
                PNG, JPG and PPM. Default is 'out.png'.
  -n <number>   Specify the number of threads to use for rendering. Defaults to the number of cores
                on the system.
  -h, --help    Show this message.
";

#[derive(RustcDecodable, Debug)]
struct Args {
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
fn thread_work(tx: Sender<Vec<ImageSample>>, queue: &sampler::BlockQueue,
               scene: &scene::Scene, light_list: &Vec<&Emitter>) {
    let mut sampler = sampler::LowDiscrepancy::new(queue.block_dim(), 16);
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
fn spawn_workers<'a>(pool: &ScopedPool<'a>, n: u32, scene: &'a scene::Scene,
                     queue: &'a sampler::BlockQueue, light_list: &'a Vec<&Emitter>)
                 -> Receiver<Vec<ImageSample>> {
    let (tx, rx) = mpsc::channel();
    for _ in 0..n {
        let t = tx.clone();
        pool.execute(move || {
            thread_work(t, queue, scene, light_list);
        });
    }
    rx
}

/// Render the scene in parallel to the render target
fn render_parallel(rt: &mut film::RenderTarget, n: u32){
    let scene = scene::Scene::new(WIDTH, HEIGHT);
    let block_queue = sampler::BlockQueue::new((WIDTH as u32, HEIGHT as u32), (8, 8));
    // TODO: Actually put lights in the BVH and pass the list through to the
    // integrator's illumination method
    let light_list: Vec<_> = scene.bvh.into_iter().filter_map(|x| {
        match x {
            &Instance::Emitter(ref e) => Some(e),
            _ => None,
        }
    }).collect();
    assert!(!light_list.is_empty(), "At least one light is required");
    {
        let pool = ScopedPool::new(n);
        let rx = spawn_workers(&pool, n, &scene, &block_queue, &light_list);
        for mut v in rx.iter() {
            for s in v.drain(..) {
                rt.write(s.x, s.y, &s.color);
            }
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let mut rt = film::RenderTarget::new(WIDTH, HEIGHT);
    let n = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };
    println!("Rendering using {} threads", n);
    let d = Duration::span(|| render_parallel(&mut rt, n));
    println!("Rendering took {}", d);
    let img = rt.get_render();
    let out_file = match args.flag_o {
        Some(f) => f,
        None => "out.png".to_string(),
    };
    match image::save_buffer(&Path::new(&out_file), &img[..], WIDTH as u32, HEIGHT as u32, image::RGB(8)) {
        Ok(_) => {},
        Err(e) => println!("Error saving image, {}", e),
    };
}

