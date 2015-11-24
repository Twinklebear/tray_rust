//! The multithreaded module provides a multithreaded execution for rendering
//! the image.

use std::path::PathBuf;
use std::iter;

use clock_ticks;
use image;
use scoped_threadpool::Pool;
use rand::StdRng;

use sampler::BlockQueue;
use film::{RenderTarget, ImageSample, Colorf};
use geometry::{Geometry, Instance, Emitter};
use sampler::{self, Sampler};
use integrator::Integrator;
use scene::Scene;
use exec::{Config, Exec};

pub struct MultiThreaded {
   config: Config,
   scene: Scene,
   render_target: RenderTarget,
   spp: usize,
}

impl MultiThreaded {
    pub fn new(config: &Config, scene: Scene, rt: RenderTarget, spp: usize) -> MultiThreaded {
        MultiThreaded { config: config.clone(),
                        scene: scene,
                        render_target: rt,
                        spp: spp,
        }
    }
    /// Launch a rendering job in parallel across the threads
    fn render_parallel(&self, pool: &mut Pool) {
        let dim = self.render_target.dimensions();
        let block_queue = BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8));
        let light_list: Vec<_> = self.scene.bvh.into_iter().filter_map(|x| {
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
                let ref r = self.render_target;
                let l = &light_list;
                scope.execute(move || {
                    thread_work(self.spp, b, &self.scene, r, l);
                });
            }
        });
    }
}

impl Exec for MultiThreaded {
    fn render(&mut self) {
        let mut pool = Pool::new(self.config.num_threads);
        println!("Rendering using {} threads\n--------------------", self.config.num_threads);
        let image_dim = self.render_target.dimensions();

        let scene_start = clock_ticks::precise_time_s();
        let time_step = self.config.frame_info.time / self.config.frame_info.frames as f32;
        for i in self.config.frame_info.start..self.config.frame_info.end + 1 {
            let frame_start_time = i as f32 * time_step;
            let frame_end_time = (i as f32 + 1.0) * time_step;
            self.scene.camera.update_shutter(frame_start_time, frame_end_time);

            // TODO: How often to re-build the BVH?
            println!("Frame {}: re-building bvh for {} to {}", i, frame_start_time, frame_end_time);
            self.scene.bvh.rebuild(frame_start_time, frame_end_time);

            println!("Frame {}: rendering for {} to {}", i, frame_start_time, frame_end_time);
            let start = clock_ticks::precise_time_s();
            self.render_parallel(&mut pool);
            let time = clock_ticks::precise_time_s() - start;
            println!("Frame {}: rendering took {}s", i, time);

            let img = self.render_target.get_render();
            let out_file = match self.config.out_path.extension() {
                Some(_) => self.config.out_path.clone(),
                None => self.config.out_path.join(PathBuf::from(format!("frame{:05}.png", i))),
            };
            match image::save_buffer(&out_file.as_path(), &img[..], image_dim.0 as u32,
                                     image_dim.1 as u32, image::RGB(8))
            {
                Ok(_) => {},
                Err(e) => println!("Error saving image, {}", e),
            };
            self.render_target.clear();
            println!("Frame {}: rendered to '{}'\n--------------------", i, out_file.display());
        }
        let time = clock_ticks::precise_time_s() - scene_start;
        println!("Rendering entire sequence took {}s", time);
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
