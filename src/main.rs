extern crate tray_rust;

use std::vec::Vec;
use std::sync::{Arc, TaskPool};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::duration::Duration;

use tray_rust::film;
use tray_rust::geometry::Geometry;
use tray_rust::sampler;
use tray_rust::sampler::{Sampler};
use tray_rust::scene;

static WIDTH: uint = 800;
static HEIGHT: uint = 600;

/// Trial of how we might do the render target stuff.
/// Threads are each sent a send end of the channel that is
/// read from by the render target thread which then saves the
/// values recieved to the framebuffer
fn thread_work(tx: Sender<(f32, f32, film::Colorf)>, queue: Arc<sampler::BlockQueue>,
               scene: Arc<scene::Scene>) {
    let mut sampler = sampler::Uniform::new(queue.block_dim());
    let mut samples = Vec::with_capacity(sampler.max_spp());
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    for b in queue.iter() {
        sampler.select_block(&b);
        while sampler.has_samples() {
            sampler.get_samples(&mut sample_pos);
            for px in sample_pos.iter() {
                let mut ray = scene.camera.generate_ray(px);
                match scene.instance.intersect(&mut ray) {
                    Some(_) => samples.push((px.0, px.1, film::Colorf::broadcast(1.0))),
                    None => samples.push((px.0, px.1, film::Colorf::new(0.0, 0.0, 1.0))),
                }
            }
            for s in samples.iter() {
                match tx.send(*s) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Worker thread exiting with send error {}", e);
                        return;
                    },
                }
            }
            samples.clear();
        }
    }
}

/// Hand the workers their own send endpoints to communicate results back
/// to the main thread but drop the sender on the main thread so once
/// all threads finish the channel closes
fn spawn_workers(pool: &TaskPool, n: uint, scene: Arc<scene::Scene>) -> Receiver<(f32, f32, film::Colorf)> {
    let (tx, rx) = mpsc::channel();
    let block_queue = Arc::new(sampler::BlockQueue::new((WIDTH as u32, HEIGHT as u32), (8, 8)));
    for _ in range(0, n) {
        let q = block_queue.clone();
        let t = tx.clone();
        let s = scene.clone();
        pool.execute(move || {
            thread_work(t, q, s);
        });
    }
    rx
}

fn test_parallel(){
    let mut rt = film::RenderTarget::new(WIDTH, HEIGHT);
    let scene = Arc::new(scene::Scene::new(WIDTH, HEIGHT));
    let n = 8;
    let pool = TaskPool::new(n);
    let rx = spawn_workers(&pool, n, scene);
    for m in rx.iter() {
        rt.write(m.0, m.1, &m.2);
    }
    film::write_ppm("out.ppm", WIDTH, HEIGHT, rt.get_render().as_slice());
}

fn main() {
    let d = Duration::span(move || test_parallel());
    println!("Rendering took {}ms", d.num_milliseconds());
}

