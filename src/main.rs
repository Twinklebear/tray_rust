extern crate image;
extern crate tray_rust;

use std::vec::Vec;
use std::sync::{Arc, TaskPool};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::duration::Duration;
use std::rand::StdRng;

use tray_rust::film;
use tray_rust::geometry::Geometry;
use tray_rust::sampler;
use tray_rust::sampler::{Sampler};
use tray_rust::scene;
use tray_rust::integrator::Integrator;

static WIDTH: usize = 800;
static HEIGHT: usize = 600;

/// Threads are each sent a sender end of the channel that is
/// read from by the render target thread which then saves the
/// values recieved to the render target
fn thread_work(tx: Sender<(f32, f32, film::Colorf)>, queue: Arc<sampler::BlockQueue>,
               scene: Arc<scene::Scene>) {
    let mut sampler = sampler::LowDiscrepancy::new(queue.block_dim(), 32);
    let mut samples = Vec::with_capacity(sampler.max_spp());
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
            for s in sample_pos.iter() {
                let mut ray = scene.camera.generate_ray(s);
                if let Some(hit) = scene.intersect(&mut ray) {
                    let c = scene.integrator.illumination(&*scene, &ray, &hit);
                    samples.push((s.0, s.1, c));
                }
            }
            for s in samples.iter() {
                if let Err(e) = tx.send(*s) {
                    println!("Worker thread exiting with send error {:?}", e);
                    return;
                }
            }
            samples.clear();
        }
    }
}

/// Spawn `n` worker threads to render the scene in parallel. Returns the receive end
/// of the channel where the threads will write their samples so that the receiver
/// can write these samples to the render target
fn spawn_workers(pool: &TaskPool, n: usize, scene: Arc<scene::Scene>) -> Receiver<(f32, f32, film::Colorf)> {
    let (tx, rx) = mpsc::channel();
    let block_queue = Arc::new(sampler::BlockQueue::new((WIDTH as u32, HEIGHT as u32), (8, 8)));
    // TODO: the .. operator precedence is very low so we need this paren here at the moment
    // once (hopefully) it's raised we can remove the parens
    for _ in (0..n) {
        let q = block_queue.clone();
        let t = tx.clone();
        let s = scene.clone();
        pool.execute(move || {
            thread_work(t, q, s);
        });
    }
    rx
}

/// Render in the scene in parallel to the render target
fn render_parallel(rt: &mut film::RenderTarget){
    let scene = Arc::new(scene::Scene::new(WIDTH, HEIGHT));
    let n = 8;
    let pool = TaskPool::new(n);
    let rx = spawn_workers(&pool, n, scene);
    for m in rx.iter() {
        rt.write(m.0, m.1, &m.2);
    }
}

fn main() {
    let mut rt = film::RenderTarget::new(WIDTH, HEIGHT);
    let d = Duration::span(|| render_parallel(&mut rt));
    println!("Rendering took {}ms", d.num_milliseconds());
    let img = rt.get_render();
    image::save_buffer(&Path::new("out.png"), &img[], WIDTH as u32, HEIGHT as u32, image::RGB(8));
}

