//! Defines the render target for tray, where our image will be written too
//! during rendering
//! TODO: Reconstruction filters

use std::vec::Vec;
use std::{iter, cmp, f32};
use std::sync::Mutex;

use film::Colorf;
use film::filter::Filter;
use sampler::Region;

const FILTER_TABLE_SIZE: usize = 16;

/// A struct containing results of an image sample where a ray was fired through
/// continuous pixel coordinates [x, y] and color `color` was computed
pub struct ImageSample {
    pub x: f32,
    pub y: f32,
    pub color: Colorf,
}

impl ImageSample {
    pub fn new(x: f32, y: f32, color: Colorf) -> ImageSample {
        ImageSample { x: x, y: y, color: color }
    }
}

/// RenderTarget is a RGBF render target to write our image too while rendering
pub struct RenderTarget {
    width: usize,
    height: usize,
    pixels: Vec<Colorf>,
    pixels_locked: Vec<Mutex<[Colorf; 8 * 8]>>,
    filter: Box<Filter + Send + Sync>,
    filter_table: Vec<f32>,
}

impl RenderTarget {
    /// Create a render target with `width * height` pixels
    pub fn new(width: usize, height: usize, filter: Box<Filter + Send + Sync>) -> RenderTarget {
        if width % 8 != 0 || height % 8 != 0 {
            panic!("Image with dimension {:?} not evenly divided by blocks of {:?}", (width, height), (8, 8));
        }
        let mut filter_table: Vec<f32> = iter::repeat(0.0).take(FILTER_TABLE_SIZE * FILTER_TABLE_SIZE)
            .collect();
        for y in 0..FILTER_TABLE_SIZE {
            let fy = (y as f32 + 0.5) * filter.height() / FILTER_TABLE_SIZE as f32;
            for x in 0..FILTER_TABLE_SIZE {
                let fx = (x as f32 + 0.5) * filter.width() / FILTER_TABLE_SIZE as f32;
                filter_table[y * FILTER_TABLE_SIZE + x] = filter.weight(fx, fy);
            }
        }
        let x_blocks = width / 8;
        let y_blocks = height / 8;
        println!("x_blocks = {}, y_blocks = {}", x_blocks, y_blocks);
        println!("width / 0.5 = {}, height / 0.5 = {}", f32::floor(filter.width() / 0.5),
            f32::floor(filter.height() / 0.5));
        let mut pixels_locked = Vec::with_capacity(x_blocks * y_blocks);
        for _ in 0..x_blocks * y_blocks {
            pixels_locked.push(Mutex::new([Colorf::broadcast(0.0); 8 * 8]));
        }
        RenderTarget { width: width, height: height,
            pixels: iter::repeat(Colorf::broadcast(0.0)).take(width * height).collect(),
            pixels_locked: pixels_locked,
            filter: filter,
            filter_table: filter_table,
        }
    }
    /// Write a color value to the image at `(x, y)`
    pub fn write(&mut self, x: f32, y: f32, c: &Colorf) {
        // Compute the discrete pixel coordinates which the sample hits
        let img_x = x - 0.5;
        let img_y = y - 0.5;
        let x_range = (f32::max(f32::ceil(img_x - self.filter.width()), 0.0) as usize,
                       f32::min(f32::floor(img_x + self.filter.width()),
                           self.width as f32 - 1.0) as usize);
        let y_range = (f32::max(f32::ceil(img_y - self.filter.height()), 0.0) as usize,
                       f32::min(f32::floor(img_y + self.filter.height()),
                           self.height as f32 - 1.0) as usize);
        if (x_range.1 as isize) - (x_range.0 as isize) < 0
            || (y_range.1 as isize) - (y_range.0 as isize) < 0 {
                return;
        }
        for iy in y_range.0..(y_range.1 + 1) {
            let fy = f32::abs(iy as f32 - img_y) * self.filter.inv_height() * FILTER_TABLE_SIZE as f32;
            let fy_idx = cmp::min(fy as usize, FILTER_TABLE_SIZE - 1);
            for ix in x_range.0..(x_range.1 + 1) {
                let fx = f32::abs(ix as f32 - img_x) * self.filter.inv_width() * FILTER_TABLE_SIZE as f32;
                let fx_idx = cmp::min(fx as usize, FILTER_TABLE_SIZE - 1);
                let weight = self.filter_table[fy_idx * FILTER_TABLE_SIZE + fx_idx];
                // TODO: Can't currently overload the += operator
                // Coming soon though, see RFC #953 https://github.com/rust-lang/rfcs/pull/953
                self.pixels[iy * self.width + ix].r += weight * c.r;
                self.pixels[iy * self.width + ix].g += weight * c.g;
                self.pixels[iy * self.width + ix].b += weight * c.b;
                self.pixels[iy * self.width + ix].a += weight;
            }
        }
    }
    /// Write all the color values to the image
    /// TODO: This code can definitely be optimized and cleaned up
    pub fn write_block(&self, samples: &Vec<ImageSample>, region: &Region) {
        // Determine which blocks we touch with our set of samples
        let filter_pixels = (f32::floor(self.filter.width() / 0.5) as i32,
                             f32::floor(self.filter.height() / 0.5) as i32);
        let x_range = (cmp::max(region.start.0 as i32 - filter_pixels.0, 0),
                       cmp::min(region.end.0 as i32 + filter_pixels.0, self.width as i32 - 1));
        let y_range = (cmp::max(region.start.1 as i32 - filter_pixels.1, 0),
                       cmp::min(region.end.1 as i32 + filter_pixels.1, self.height as i32 - 1));
        /*
        println!("Writing starting at {:?} with filter pixels = {:?}\n\tx_range = {:?}, y_range = {:?}",
                 region.start, filter_pixels, x_range, y_range);
         */
        let block_x_range = (x_range.0 / 8, x_range.1 / 8);
        let block_y_range = (y_range.0 / 8, y_range.1 / 8);
        //println!("\tBlock x range {:?}, y range {:?}", block_x_range, block_y_range);
        if x_range.1 - x_range.0 < 0 || y_range.1 - y_range.0 < 0 {
                return;
        }
        let blocks_per_row = self.width as i32 / 8;
        for y in block_y_range.0..block_y_range.1 + 1 {
            for x in block_x_range.0..block_x_range.1 + 1 {
                //println!("\tBlock [{}, {}] is index {}", x, y, y * blocks_per_row + x);
                let x_write_range = (cmp::max(x_range.0, x * 8), cmp::min(x_range.1 + 1, x * 8 + 8));
                let y_write_range = (cmp::max(y_range.0, y * 8), cmp::min(y_range.1 + 1, y * 8 + 8));
                //println!("\tWriting to pixels x range: {:?}, y range {:?}", x_write_range, y_write_range);
                let block_samples = samples.iter().filter(|s| {
                    s.x >= (x_write_range.0 - filter_pixels.0) as f32
                        && s.x < (x_write_range.1 + filter_pixels.0) as f32
                        && s.y >= (y_write_range.0 - filter_pixels.1) as f32
                        && s.y < (y_write_range.1 + filter_pixels.1) as f32
                });
                // Acquire lock for the block and write the samples
                // TODO: Move more work out of critical section
                let block_idx = (y * blocks_per_row + x) as usize;
                let mut pixels = self.pixels_locked[block_idx].lock().unwrap();
                for c in block_samples {
                    let img_x = c.x - 0.5;
                    let img_y = c.y - 0.5;
                    for iy in y_write_range.0..y_write_range.1 {
                        let fy = f32::abs(iy as f32 - img_y) * self.filter.inv_height()
                            * FILTER_TABLE_SIZE as f32;
                        let fy_idx = cmp::min(fy as usize, FILTER_TABLE_SIZE - 1);
                        for ix in x_write_range.0..x_write_range.1 {
                            let fx = f32::abs(ix as f32 - img_x) * self.filter.inv_width()
                                * FILTER_TABLE_SIZE as f32;
                            let fx_idx = cmp::min(fx as usize, FILTER_TABLE_SIZE - 1);
                            let weight = self.filter_table[fy_idx * FILTER_TABLE_SIZE + fx_idx];
                            // TODO: Can't currently overload the += operator
                            // Coming soon though, see RFC #953 https://github.com/rust-lang/rfcs/pull/953
                            let px = ((iy - y * 8) * 8 + ix - x * 8) as usize;
                            //println!("\tWriting pixel [{}, {}], block pix index = {}", ix, iy, px);
                            pixels[px].r += weight * c.color.r;
                            pixels[px].g += weight * c.color.g;
                            pixels[px].b += weight * c.color.b;
                            pixels[px].a += weight;
                        }
                    }
                }
            }
        }
    }
    /// Clear the render target to black
    pub fn clear(&mut self) {
        for p in self.pixels.iter_mut() {
            *p = Colorf::broadcast(0.0);
        }
    }
    /// Get the dimensions of the render target
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    /// Convert the floating point color buffer to 24bpp sRGB for output to an image
    pub fn get_render(&self) -> Vec<u8> {
        let mut render: Vec<u8> = iter::repeat(0u8).take(self.width * self.height * 3).collect();
        for y in 0..self.height {
            for x in 0..self.width {
                let c = &self.pixels[y * self.width + x];
                if c.a != 0.0 {
                    let cn = (*c / c.a).clamp().to_srgb();
                    for i in 0..3 {
                        render[y * self.width * 3 + x * 3 + i] = (cn[i] * 255.0) as u8;

                    }
                }
            }
        }
        render
    }
    /// Get the lock-block rendered image (TODO this is just for debugging)
    pub fn get_block_render(&self) -> Vec<u8> {
        let mut render: Vec<u8> = iter::repeat(0u8).take(self.width * self.height * 3).collect();
        let x_blocks = self.width / 8;
        let y_blocks = self.height / 8;
        for by in 0..y_blocks {
            for bx in 0..x_blocks {
                let block_idx = (by * x_blocks + bx) as usize;
                let pixels = self.pixels_locked[block_idx].lock().unwrap();
                for y in 0..8 {
                    for x in 0..8 {
                        let c = &pixels[y * 8 + x];
                        if c.a > 0.0 {
                            let cn = (*c / c.a).clamp().to_srgb();
                            let px = (y + by * 8) * self.width * 3 + (x + bx * 8) * 3;
                            for i in 0..3 {
                                render[px + i] = (cn[i] * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
        render
    }
}

