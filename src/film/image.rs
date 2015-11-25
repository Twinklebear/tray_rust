//! Provides a simple RGBA_F32 image, used by the distributed master to store results
//! from the worker processes

use std::iter;

use film::Colorf;

pub struct Image {
    dim: (usize, usize),
    pixels: Vec<Colorf>,
}

impl Image {
    pub fn new(dimensions: (usize, usize)) -> Image {
        let pixels = iter::repeat(Colorf::broadcast(0.0)).take(dimensions.0 * dimensions.1).collect();
        Image { dim: dimensions, pixels: pixels }
    }
    /// Add the floating point RGBA_F32 pixels to the image. It is assumed that `pixels` contains
    /// a `dim.0` by `dim.1` pixel image.
    pub fn add_pixels(&mut self, pixels: &Vec<f32>) {
        for y in 0..self.dim.1 {
            for x in 0..self.dim.0 {
                let mut c = &mut self.pixels[y * self.dim.0 + x];
                let px = y * self.dim.0 * 4 + x * 4;
                for i in 0..4 {
                    c[i] = pixels[px + i];
                }
            }
        }
    }
    /// Convert the Image to sRGB8 format and return it
    pub fn get_srgb8(&self) -> Vec<u8> {
        let mut render: Vec<u8> = iter::repeat(0u8).take(self.dim.0 * self.dim.1 * 3).collect();
        for y in 0..self.dim.1 {
            for x in 0..self.dim.0 {
                let c = &self.pixels[y * self.dim.0 + x];
                if c.a > 0.0 {
                    let cn = (*c / c.a).clamp().to_srgb();
                    let px = y  * self.dim.0 * 3 + x * 3;
                    for i in 0..3 {
                        render[px + i] = (cn[i] * 255.0) as u8;
                    }
                }
            }
        }
        render
    }
    pub fn dimensions(&self) -> (usize, usize) {
        self.dim
    }
}

