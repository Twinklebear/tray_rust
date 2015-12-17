//! The multithreaded module provides a multithreaded execution for rendering
//! the image.

use std::iter;

use clock_ticks;
use scoped_threadpool::Pool;
use rand::StdRng;

use sampler::BlockQueue;
use film::{RenderTarget, ImageSample, Colorf};
use geometry::{Geometry, Instance, Emitter};
use sampler::{self, Sampler};
use integrator::Integrator;
use scene::Scene;
use exec::{Config, Exec};

/// The MultiThreaded execution uses a configurable number of threads in
/// a threadpool to render each frame
pub struct MultiThreaded {
    pool: Pool,
}

impl MultiThreaded {
    /// Create a new multithreaded renderer which will use `num_threads` to render the image
    pub fn new(num_threads: u32) -> MultiThreaded {
        MultiThreaded { pool: Pool::new(num_threads) }
    }
    /// Launch a rendering job in parallel across the threads and wait for it to finish
    fn render_parallel(&mut self, scene: &Scene, rt: &RenderTarget, config: &Config) {
        let dim = rt.dimensions();
        let block_queue = BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8), config.select_blocks);
        let light_list: Vec<_> = scene.bvh.iter().filter_map(|x| {
            match x {
                &Instance::Emitter(ref e) => Some(e),
                _ => None,
            }
        }).collect();
        assert!(!light_list.is_empty(), "At least one light is required");
        let n = self.pool.thread_count();
        self.pool.scoped(|scope| {
            for _ in 0..n {
                let b = &block_queue;
                let ref r = rt;
                let l = &light_list;
                scope.execute(move || {
                    thread_work(config.spp, b, scene, r, l);
                });
            }
        });
    }
}

impl Exec for MultiThreaded {
    fn render(&mut self, scene: &mut Scene, rt: &mut RenderTarget, config: &Config) {
        println!("Rendering using {} threads\n--------------------", self.pool.thread_count());
        let time_step = config.frame_info.time / config.frame_info.frames as f32;
        let frame_start_time = config.current_frame as f32 * time_step;
        let frame_end_time = (config.current_frame as f32 + 1.0) * time_step;
        scene.camera.update_frame(frame_start_time, frame_end_time);

        // TODO: How often to re-build the BVH?
        let shutter_time = scene.camera.shutter_time();
        println!("Frame {}: re-building bvh for {} to {}", config.current_frame,
                 shutter_time.0, shutter_time.1);
        scene.bvh.rebuild(shutter_time.0, shutter_time.1);

        println!("Frame {}: rendering for {} to {}", config.current_frame,
                 frame_start_time, frame_end_time);
        let start = clock_ticks::precise_time_s();
        self.render_parallel(scene, rt, config);
        let time = clock_ticks::precise_time_s() - start;
        println!("Frame {}: rendering took {}s", config.current_frame, time);
    }
}

fn thread_work(spp: usize, queue: &BlockQueue, scene: &Scene,
               target: &RenderTarget, light_list: &Vec<&Emitter>) {
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

