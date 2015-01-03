extern crate tray_rust;

use std::num::Float;
use std::vec::Vec;
use tray_rust::linalg;
use tray_rust::film;
use tray_rust::geometry;
use tray_rust::geometry::Geometry;
use tray_rust::sampler;
use tray_rust::sampler::morton;
use tray_rust::sampler::Sampler;

fn main() {
    let width = 800u;
    let height = 600u;
    let num_blocks = ((width as f32 / 8.0), (height as f32 / 8.0));
    if num_blocks.0 != Float::floor(num_blocks.0) || num_blocks.1 != Float::floor(num_blocks.1) {
        panic!("Warning! Block dimensions don't evenly divide image");
    }
    let block_count = (num_blocks.0 as u32, num_blocks.1 as u32);
    let mut blocks: Vec<(u32, u32)> = range(0, block_count.0 * block_count.1)
        .map(|i| (i % block_count.0, i / block_count.0)).collect();
    blocks.sort_by(|a, b| morton::morton2(a).cmp(&morton::morton2(b)));

    let mut rt = film::RenderTarget::new(width, height);
    let camera = film::Camera::new(linalg::Transform::look_at(
        &linalg::Point::new(0.0, 0.0, -10.0), &linalg::Point::new(0.0, 0.0, 0.0),
        &linalg::Vector::new(0.0, 1.0, 0.0)), 40.0, rt.dimensions());
    let sphere = geometry::Sphere::new(1.5);
    let instance = geometry::Instance::new(&sphere,
        linalg::Transform::translate(&linalg::Vector::new(0.0, 2.0, 0.0)));

    let mut sampler = sampler::Uniform::new((8, 8));
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    for b in blocks.iter() {
        sampler.select_block(b);
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

