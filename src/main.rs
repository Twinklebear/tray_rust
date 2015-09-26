#![feature(duration_span)]

extern crate image;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate tray_rust;

use std::vec::Vec;
use std::iter;
use std::path::Path;
use std::time::Duration;

use rand::StdRng;
use docopt::Docopt;

use tray_rust::film::{self, filter, ImageSample};
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

/// Threads are each sent a sender end of the channel that is
/// read from by the render target thread which then saves the
/// values recieved to the render target
fn thread_work(spp: usize, queue: &sampler::BlockQueue, scene: &scene::Scene,
               target: &film::RenderTarget, light_list: &Vec<&Emitter>) {
    let mut sampler = sampler::LowDiscrepancy::new(queue.block_dim(), spp);
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    let mut time_samples: Vec<_> = iter::repeat(0.0).take(sampler.max_spp()).collect();
    let block_dim = queue.block_dim();
    let mut block_samples = Vec::with_capacity(sampler.max_spp() * (block_dim.0 * block_dim.1) as usize);
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
            sampler.get_samples_1d(&mut time_samples[..], &mut rng);
            for (s, t) in sample_pos.iter().zip(time_samples.iter()) {
                let mut ray = scene.camera.generate_ray(s, *t);
                if let Some(hit) = scene.intersect(&mut ray) {
                    let c = scene.integrator.illumination(scene, light_list, &ray,
                                                          &hit, &mut sampler, &mut rng).clamp();
                    block_samples.push(ImageSample::new(s.0, s.1, c));
                }
            }
        }
        target.write_block(&block_samples, sampler.get_region());
        block_samples.clear();
    }
}

/// Render the scene in parallel using `n` threads and write the result to the render target
fn render_parallel(rt: &mut film::RenderTarget, scene: &scene::Scene, n: u32, spp: usize){
    let mut pool = scoped_threadpool::Pool::new(n);
    println!("Rendering using {} threads", n);
    pool.scoped(|scope| {
        let dim = rt.dimensions();
        let block_queue = sampler::BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8));
        let light_list: Vec<_> = scene.bvh.into_iter().filter_map(|x| {
            match x {
                &Instance::Emitter(ref e) => Some(e),
                _ => None,
            }
        }).collect();
        assert!(!light_list.is_empty(), "At least one light is required");

        for _ in 0..n {
            let b = &block_queue;
            let r = &*rt;
            let l = &light_list;
            scope.execute(move || {
                thread_work(spp, b, scene, r, l);
            });
        }
    });
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let (mut scene, spp, image_dim) = scene::Scene::load_file(&args.arg_scenefile[..]);
    let mut rt = film::RenderTarget::new(image_dim, (2, 2),
                    Box::new(filter::MitchellNetravali::new(2.0, 2.0, 1.0 / 3.0, 1.0 / 3.0))
                    as Box<filter::Filter + Send + Sync>);
    let n = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };

    // Render 5 frames. TODO: This should be read from scene file
    let scene_time = 0.0;
    let frames = 1;
    let time_step = scene_time / (frames as f32);
    for i in 0..frames {
        scene.camera.update_shutter(i as f32 * time_step, (i as f32 + 1.0) * time_step);
        let d = Duration::span(|| render_parallel(&mut rt, &scene, n, spp));
        let time = d.as_secs() as f64 + (d.subsec_nanos() as f64) / 1_000_000_000.0;
        println!("Rendering took {}s", time);

        let img = rt.get_render();
        let out_file = match &args.flag_o {
            &Some(ref f) => f.to_string(),
            &None => format!("out_frame{:03}.png", i).to_string(),
        };

        match image::save_buffer(&Path::new(&out_file), &img[..], image_dim.0 as u32, image_dim.1 as u32, image::RGB(8)) {
            Ok(_) => {},
            Err(e) => println!("Error saving image, {}", e),
        };
        rt.clear();
    }
}

