//! Defines the render target for tray, where our image will be written too
//! during rendering
//! TODO: Reconstruction filters

use std::vec::Vec;
use std::iter;
use linalg;
use film::Colorf;

/// RenderTarget is a RGBF render target to write our image too while rendering
#[derive(Clone)]
pub struct RenderTarget {
    width: uint,
    height: uint,
    pixels: Vec<Colorf>,
}

impl RenderTarget {
    /// Create a render target with `width * height` pixels
    pub fn new(width: uint, height: uint) -> RenderTarget {
        RenderTarget { width: width, height: height,
            pixels: iter::repeat(Colorf::broadcast(0f32)).take(width * height).collect(),
        }
    }
    /// Write a color value to the image at `(x, y)`
    pub fn write(&mut self, x: f32, y: f32, c: &Colorf) {
        // Compute the discrete pixel coordinates which the sample hits, no filtering for now
        let ix = linalg::clamp(x - 0.5, 0f32, (self.width - 1) as f32) as uint;
        let iy = linalg::clamp(y - 0.5, 0f32, (self.height - 1) as f32) as uint;
        self.pixels[iy * self.width + ix] = *c;
        // Set the filter weight, currently just a box filter with single pixel extent
        self.pixels[iy * self.width + ix].a += 1.0;
    }
    /// Get the dimensions of the render target
    pub fn dimensions(&self) -> (uint, uint) {
        (self.width, self.height)
    }
    /// Convert the floating point color buffer to 24bpp sRGB for output to an image
    pub fn get_render(&self) -> Vec<u8> {
        let mut render: Vec<u8> = iter::repeat(0u8).take(self.width * self.height * 3).collect();
        for y in range(0u, self.height) {
            for x in range(0u, self.width) {
                let c = &self.pixels[y * self.width + x];
                if c.a != 0.0 {
                    let cn = (*c / c.a).clamp().to_srgb();
                    for i in range(0u, 3) {
                        render[y * self.width * 3 + x * 3 + i] = (cn[i] * 255.0) as u8;
                    }
                }
            }
        }
        render
    }
}

