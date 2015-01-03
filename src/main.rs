extern crate tray_rust;

use std::vec::Vec;
use std::sync::{Arc, TaskPool};
use std::io::timer;
use std::time::duration::Duration;
use tray_rust::linalg;
use tray_rust::film;
use tray_rust::geometry;
use tray_rust::geometry::Geometry;
use tray_rust::sampler;
use tray_rust::sampler::{Sampler};

/// Trial of how we might do the render target stuff.
/// Threads are each sent a send end of the channel that is
/// read from by the render target thread which then saves the
/// values recieved to the framebuffer
fn thread_work(tx: Sender<uint>, i: uint, queue: Arc<sampler::BlockQueue>) {
    for _ in queue.iter() {
        tx.send(i);
        // Pretend to do some work
        timer::sleep(Duration::milliseconds(50));
    }
}

/// Hand the workers their own send endpoints to communicate results back
/// to the main thread but drop the sender on the main thread so once
/// all threads finish the channel closes
fn spawn_workers(pool: &TaskPool, n: uint) -> Receiver<uint> {
    let (tx, rx) = channel();
    let block_queue = Arc::new(sampler::BlockQueue::new((10, 10), (2, 2)));
    for i in range(0, n) {
        let t = tx.clone();
        let q = block_queue.clone();
        // Can we just directly pass thread_work here? How would we send the args?
        pool.execute(move || {
            thread_work(t, i, q);
        });
    }
    rx
}

fn thread_test() {
    let n = 8;
    let pool = TaskPool::new(n);
    let rx = spawn_workers(&pool, n);
    for m in rx.iter() {
        println!("Message: {}", m);
    }
}

fn main() {
    thread_test();
    let width = 800u;
    let height = 600u;

    let mut rt = film::RenderTarget::new(width, height);
    let camera = film::Camera::new(linalg::Transform::look_at(
        &linalg::Point::new(0.0, 0.0, -10.0), &linalg::Point::new(0.0, 0.0, 0.0),
        &linalg::Vector::new(0.0, 1.0, 0.0)), 40.0, rt.dimensions());
    let sphere = geometry::Sphere::new(1.5);
    let instance = geometry::Instance::new(&sphere,
        linalg::Transform::translate(&linalg::Vector::new(0.0, 2.0, 0.0)));

    let block_queue = sampler::BlockQueue::new((width as u32, height as u32), (8, 8));
    let mut sampler = sampler::Uniform::new(block_queue.block_dim());
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    for b in block_queue.iter() {
        sampler.select_block(&b);
        while sampler.has_samples() {
            sampler.get_samples(&mut sample_pos);
            for px in sample_pos.iter() {
                let mut ray = camera.generate_ray(*px);
                match instance.intersect(&mut ray) {
                    Some(_) => rt.write(px.0, px.1, &film::Colorf::broadcast(1.0)),
                    None => rt.write(px.0, px.1, &film::Colorf::new(0.0, 0.0, 1.0)),
                }
            }
        }
    }
    film::write_ppm("out.ppm", width, height, rt.get_render().as_slice());
}

