//! The film module provides color types and a render target that the image
//! is written too. Functions are also provided for saving PPM and BMP images
//! while I wait to migrate over to the Piston image library due to some
//! compile issues TODO: https://github.com/PistonDevelopers/image

use std::io::{File, Open, Write, BufferedWriter};

pub use self::color::{Colorf, Color24};

pub mod color;

/// Write the sequence of bytes as a PPM image file with the desired name
pub fn write_ppm(name: &str, w: u32, h: u32, img: &[u8]) {
    let file = match File::open_mode(&Path::new(name), Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open {}: {}", name, e),
    };
    let mut writer = BufferedWriter::new(file);
    match write!(&mut writer, "P6\n{} {}\n255\n", w, h) {
        Err(e) => panic!("Failed to write pixel data to {}: {}", name, e),
        _ => {},
    }
    match writer.write(img) {
        Err(e) => panic!("Failed to write pixel data to {}: {}", name, e),
        _ => {},
    }
}

