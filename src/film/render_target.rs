//! Defines the render target for tray, where our image will be written too
//! during rendering
//! TODO: Reconstruction filters

use std::vec::Vec;
use std::{iter, cmp, f32};

use film::Colorf;
use film::filter::Filter;

const FILTER_TABLE_SIZE: usize = 16;

/// RenderTarget is a RGBF render target to write our image too while rendering
pub struct RenderTarget {
    width: usize,
    height: usize,
    pixels: Vec<Colorf>,
    filter: Box<Filter>,
    filter_table: Vec<f32>,
}

impl RenderTarget {
    /// Create a render target with `width * height` pixels
    pub fn new(width: usize, height: usize, filter: Box<Filter>) -> RenderTarget {
        let mut filter_table: Vec<f32> = iter::repeat(0.0).take(FILTER_TABLE_SIZE * FILTER_TABLE_SIZE)
            .collect();
        for y in 0..FILTER_TABLE_SIZE {
            let fy = (y as f32 + 0.5) * filter.height() / FILTER_TABLE_SIZE as f32;
            for x in 0..FILTER_TABLE_SIZE {
                let fx = (x as f32 + 0.5) * filter.width() / FILTER_TABLE_SIZE as f32;
                filter_table[y * FILTER_TABLE_SIZE + x] = filter.weight(fx, fy);
            }
        }
        RenderTarget { width: width, height: height,
            pixels: iter::repeat(Colorf::broadcast(0.0)).take(width * height).collect(),
            filter: filter,
            filter_table: filter_table,
        }
    }
    /// Write a color value to the image at `(x, y)`
    pub fn write(&mut self, x: f32, y: f32, c: &Colorf) {
        // Compute the discrete pixel coordinates which the sample hits, no filtering for now
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
}

