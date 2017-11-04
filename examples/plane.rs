extern crate tray_rust;
extern crate rand;
extern crate image;

use std::sync::Arc;
use rand::StdRng;

use tray_rust::linalg::{AnimatedTransform, Transform, Point, Vector};
use tray_rust::film::{Colorf, RenderTarget, Camera, ImageSample};
use tray_rust::film::filter::MitchellNetravali;
use tray_rust::geometry::{Rectangle, Instance};
use tray_rust::material::Matte;
use tray_rust::sampler::{BlockQueue, LowDiscrepancy, Sampler};
use tray_rust::texture;

fn main() {
    let width = 800usize;
    let height = 600usize;
    let filter =
        Box::new(MitchellNetravali::new(2.0, 2.0, 0.333333333333333333, 0.333333333333333333));
    let rt = RenderTarget::new((width, height), (20, 20), filter);
    let transform =
        AnimatedTransform::unanimated(&Transform::look_at(&Point::new(0.0, 0.0, -10.0),
                                                          &Point::new(0.0, 0.0, 0.0),
                                                          &Vector::new(0.0, 1.0, 0.0)));
    let camera = Camera::new(transform, 40.0, rt.dimensions(), 0.5, 0);
    let plane = Rectangle::new(2.0, 2.0);
    let geometry_lock = Arc::new(plane);
    // TODO: From a code usage standpoint it might be nice to have a constant version
    // of the material ctor exposed which takes the plain types and builds the textures internally
    let texture = Arc::new(texture::ConstantColor::new(Colorf::new(0.740063, 0.742313, 0.733934)));
    let roughness = Arc::new(texture::ConstantScalar::new(1.0));
    let white_wall = Matte::new(texture, roughness);
    let material_lock = Arc::new(white_wall);
    let position_transform =
        AnimatedTransform::unanimated(&Transform::translate(&Vector::new(0.0, 2.0, 0.0)));
    let instance = Instance::receiver(geometry_lock,
                                      material_lock,
                                      position_transform,
                                      "single_plane".to_string());

    let dim = rt.dimensions();
    // A block queue is how work is distributed among threads, it's a list of tiles
    // of the image that have yet to be rendered. Each thread will pull a block from
    // this queue and render it.
    let block_queue = BlockQueue::new((dim.0 as u32, dim.1 as u32), (8, 8), (0, 0));
    let block_dim = block_queue.block_dim();
    // A sample is responsible for choosing randomly placed locations within a pixel to
    // get a good sampling of the image. Using a poor quality sampler will resuly in a
    // noiser and more aliased image that converges slower. The LowDiscrepency sampler
    // is a good choice for quality.
    let mut sampler = LowDiscrepancy::new(block_dim, 2);
    let mut sample_pos = Vec::with_capacity(sampler.max_spp());
    let mut block_samples = Vec::with_capacity(sampler.max_spp() *
                                               (block_dim.0 * block_dim.1) as usize);
    let mut rng = match StdRng::new() {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to get StdRng, {}", e);
            return;
        }
    };
    // Grab a block from the queue and start working on it, submitting samples
    // to the render target thread after each pixel
    for b in block_queue.iter() {
        sampler.select_block(b);
        // While the sampler has samples left to take for this pixel, take some samples
        while sampler.has_samples() {
            // Get samples for a pixel and render them
            sampler.get_samples(&mut sample_pos, &mut rng);
            for s in &sample_pos[..] {
                let mut ray = camera.generate_ray(s, 0.0);
                if let Some(_) = instance.intersect(&mut ray) {
                    block_samples.push(ImageSample::new(s.0, s.1, Colorf::broadcast(1.0)));
                } else {
                    // For correct filtering we also MUST set a background color of some kind
                    // if we miss, otherwise the pixel weights will be wrong and we'll see object
                    // fringes and artifacts at object boundaries w/ nothing. Try removing this
                    // line and rendering again.
                    block_samples.push(ImageSample::new(s.0, s.1, Colorf::black()));
                }
            }
        }
        // We write all samples at once so we don't need to lock the render target tiles as often
        rt.write(&block_samples, sampler.get_region());
        block_samples.clear();
    }

    // Get the sRGB8 render buffer from the floating point framebuffer and save it
    let img = rt.get_render();
    match image::save_buffer("plane.png",
                             &img[..],
                             dim.0 as u32,
                             dim.1 as u32,
                             image::RGB(8)) {
        Ok(_) => {}
        Err(e) => println!("Error saving image, {}", e),
    };
}
