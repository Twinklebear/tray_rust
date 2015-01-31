//! Defines the render target for tray, where our image will be written too
//! during rendering
//! TODO: Reconstruction filters

use std::vec::Vec;
use std::iter;
use std::cmp;
use std::num::Float;

use linalg;
use film::Colorf;

/// RenderTarget is a RGBF render target to write our image too while rendering
#[derive(Clone)]
pub struct RenderTarget {
    width: usize,
    height: usize,
    pixels: Vec<Colorf>,
}

impl RenderTarget {
    /// Create a render target with `width * height` pixels
    pub fn new(width: usize, height: usize) -> RenderTarget {
        RenderTarget { width: width, height: height,
            pixels: iter::repeat(Colorf::broadcast(0.0)).take(width * height).collect(),
        }
    }
    /// Write a color value to the image at `(x, y)`
    pub fn write(&mut self, x: f32, y: f32, c: &Colorf) {
        // Compute the discrete pixel coordinates which the sample hits, no filtering for now
        let img_x = x - 0.5;
        let img_y = y - 0.5;
        // TODO: We're just pretending to be a single pixel box filter for now
        let x_range = (Float::max(Float::ceil(img_x - 0.5), 0.0) as usize,
                       Float::min(Float::floor(img_x + 0.5), self.width as f32 - 1.0) as usize);
        let y_range = (Float::max(Float::ceil(img_y - 0.5), 0.0) as usize,
                       Float::min(Float::floor(img_y + 0.5), self.height as f32 - 1.0) as usize);
        let ix = x_range.0;
        let iy = y_range.0;
        // TODO: Can't currently overload the += operator
        self.pixels[iy * self.width + ix] = self.pixels[iy * self.width + ix] + *c;
        // Set the filter weight, currently just a box filter with single pixel extent
        self.pixels[iy * self.width + ix].a += 1.0;
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
                    for i in 0..3us {
                        render[y * self.width * 3 + x * 3 + i] = (cn[i] * 255.0) as u8;
                    }
                }
            }
        }
        render
    }
}

