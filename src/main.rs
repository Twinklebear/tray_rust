extern crate image;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate clock_ticks;
extern crate tray_rust;

use std::vec::Vec;
use std::iter;
use std::path::PathBuf;
use std::io::ErrorKind;

use rand::StdRng;
use docopt::Docopt;

use tray_rust::film::{self, ImageSample, Colorf};
use tray_rust::geometry::{Geometry, Instance, Emitter};
use tray_rust::sampler::{self, Sampler};
use tray_rust::scene;
use tray_rust::integrator::Integrator;

static USAGE: &'static str = "
Usage: tray_rust <scenefile> [options]

Options:
  -o <file>               Specify the output file or directory to save the image or frames. Supported formats are
                          PNG, JPG and PPM. Default is 'frame<#>.png'.
  -n <number>             Specify the number of threads to use for rendering. Defaults to the number of cores
                          on the system.
  --start-frame <number>  Specify frame to start rendering at, specifies an inclusive range [start, end]
  --end-frame <number>    Specify frame to stop rendering at, specifies an inclusive range [start, end]
  -h, --help              Show this message.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_scenefile: String,
    flag_o: Option<String>,
    flag_n: Option<u32>,
    flag_start_frame: Option<usize>,
    flag_end_frame: Option<usize>,
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
        let mut pixel_samples = 0;
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
                } else {
                    block_samples.push(ImageSample::new(s.0, s.1, Colorf::black()));
                }
            }
            // If the samples are ok the samples for the next pixel start at the end of the current
            // pixel's samples
            if sampler.report_results(&block_samples[pixel_samples..]) {
                pixel_samples = block_samples.len();
            }
        }
        target.write(&block_samples, sampler.get_region());
        block_samples.clear();
    }
}

/// Render the scene in parallel using `n` threads and write the result to the render target
fn render_parallel(rt: &mut film::RenderTarget, scene: &scene::Scene, pool: &mut scoped_threadpool::Pool, spp: usize){
    let dim = rt.dimensions();
    let block_queue = sampler::BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8));
    let light_list: Vec<_> = scene.bvh.into_iter().filter_map(|x| {
        match x {
            &Instance::Emitter(ref e) => Some(e),
            _ => None,
        }
    }).collect();
    assert!(!light_list.is_empty(), "At least one light is required");
    let n = pool.thread_count();
    pool.scoped(|scope| {
        for _ in 0..n {
            let b = &block_queue;
            let ref r = rt;
            let l = &light_list;
            scope.execute(move || {
                thread_work(spp, b, scene, r, l);
            });
        }
    });
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    let n = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };
    let out_path = match &args.flag_o {
        &Some(ref f) => {
            let p = PathBuf::from(f);
            // If we're writing to a directory make sure it exists
            if p.extension() == None {
                match std::fs::create_dir(p.as_path()) {
                    Err(e) => {
                        if e.kind() != ErrorKind::AlreadyExists {
                            panic!("Failed to create output directory");
                        }
                    },
                    Ok(_) => {},
                }
            }
            p
        },
        &None => PathBuf::from("./"),
    };

    let (mut scene, mut rt, spp, frame_info) = scene::Scene::load_file(&args.arg_scenefile[..]);
    let image_dim = rt.dimensions();
    println!("Rendering using {} threads\n--------------------", n);
    let mut pool = scoped_threadpool::Pool::new(n);
    let scene_start = clock_ticks::precise_time_s();

    let start_frame = match args.flag_start_frame {
        Some(x) => x,
        _ => frame_info.start,
    };
    let end_frame = match args.flag_end_frame {
        Some(x) => x,
        _ => frame_info.end,
    };

    let time_step = frame_info.time / frame_info.frames as f32;
    for i in start_frame..end_frame + 1 {
        let frame_start_time = i as f32 * time_step;
        let frame_end_time = (i as f32 + 1.0) * time_step;
        scene.camera.update_shutter(frame_start_time, frame_end_time);
        // TODO: How often to re-build the BVH?
        println!("Frame {}: re-building bvh for {} to {}", i, frame_start_time, frame_end_time);
        scene.bvh.rebuild(frame_start_time, frame_end_time);
        println!("Frame {}: rendering for {} to {}", i, frame_start_time, frame_end_time);
        let start = clock_ticks::precise_time_s();
        render_parallel(&mut rt, &scene, &mut pool, spp);
        let time = clock_ticks::precise_time_s() - start;
        println!("Frame {}: rendering took {}s", i, time);

        let img = rt.get_render();
        let out_file = match out_path.extension() {
            Some(_) => out_path.clone(),
            None => out_path.join(PathBuf::from(format!("frame{:05}.png", i))),
        };
        match image::save_buffer(&out_file.as_path(), &img[..], image_dim.0 as u32, image_dim.1 as u32, image::RGB(8)) {
            Ok(_) => {},
            Err(e) => println!("Error saving image, {}", e),
        };
        rt.clear();
        println!("Frame {}: rendered to '{}'\n--------------------", i, out_file.display());
    }
    let time = clock_ticks::precise_time_s() - scene_start;
    println!("Rendering entire sequence took {}s", time);
}

