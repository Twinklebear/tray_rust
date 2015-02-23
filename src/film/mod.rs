//! The film module provides color types and a render target that the image
//! is written too. Functions are also provided for saving PPM and BMP images

use std::old_io::{File, Open, Write, BufferedWriter};

pub use self::color::Colorf;
pub use self::render_target::RenderTarget;
pub use self::camera::Camera;

pub mod color;
pub mod render_target;
pub mod camera;

/// Write the sequence of bytes as a PPM image file with the desired name
pub fn write_ppm(name: &str, w: usize, h: usize, img: &Vec<u8>) {
    let file = match File::open_mode(&Path::new(name), Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open {}: {}", name, e),
    };
    let mut writer = BufferedWriter::new(file);
    if let Err(e) = write!(&mut writer, "P6\n{} {}\n255\n", w, h) {
        panic!("Failed to write pixel data to {}: {}", name, e);
    }
    if let Err(e) = writer.write_all(&img[..]) {
        panic!("Failed to write pixel data to {}: {}", name, e);
    }
}
/// Write the sequence of bytes as a 24bpp BMP image
/// Note that the bytes should be in BGR order and have previously
/// been flipped to the correct y-order for BMP output
pub fn write_bmp(name: &str, w: u32, h: u32, img: &Vec<u8>) {
    let file = match File::open_mode(&Path::new(name), Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open {}: {}", name, e),
    };
    let mut writer = BufferedWriter::new(file);
    // Write BMP header
    if let Err(e) = write!(&mut writer, "BM") {
        panic!("Failed to write image {}: {}", name, e);
    }
    let padding = (w * 3) % 4;
    let img_size = h * (3 * w + padding);
    // We're just using the BMPINFOHEADER so our total header size is 54, 14 byte
    // file header + 40 byte BMPINFOHEADER
    let file_size = 54 + img_size;
    if let Err(e) = writer.write_le_u32(file_size) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // 4 bytes we don't care about, just put some random num
    if let Err(e) = writer.write_le_u32(0) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // We're just using the BMPINFOHEADER so the offset to pixel data is always 54
    if let Err(e) = writer.write_le_u32(54) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Same reason as above the BMPINFOHEADER is 40 bytes
    if let Err(e) = writer.write_le_u32(40) {
        panic!("Failed to write image {}: {}", name, e);
    }
    if let Err(e) = writer.write_le_i32(w as i32) {
        panic!("Failed to write image {}: {}", name, e);
    }
    if let Err(e) = writer.write_le_i32(h as i32) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Color planes is unused, just set to 1
    if let Err(e) = writer.write_le_u16(1) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Always writing 24bpp BMPs
    if let Err(e) = writer.write_le_u16(24) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Uncompressed BMPs
    if let Err(e) = writer.write_le_u32(0) {
        panic!("Failed to write image {}: {}", name, e);
    }
    if let Err(e) = writer.write_le_u32(img_size) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Resolution just seems to be ignored? set to 2835 like Wikipedia
    if let Err(e) = writer.write_le_i32(2835) {
        panic!("Failed to write image {}: {}", name, e);
    }
    if let Err(e) = writer.write_le_i32(2835) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // No color palette or important colors
    if let Err(e) = writer.write_le_u32(0) {
        panic!("Failed to write image {}: {}", name, e);
    }
    if let Err(e) = writer.write_le_u32(0) {
        panic!("Failed to write image {}: {}", name, e);
    }
    // Finally we can write the image, since BMP is upside-down we need
    // to flip the rows though. We also handle padding of rows here
    for r in 0..h {
        let begin = 3 * w * (h - r - 1);
        let end = begin + 3 * w;
        if let Err(e) = writer.write_all(&img[begin as usize..end as usize]) {
            panic!("Failed to write image {}: {}", name, e);
        }
        // Write any required padding, it's just padding so just throw some junk there
        if padding > 0 {
            if let Err(e) = writer.write_all(&img[0..padding as usize]) {
                panic!("Failed to write image {}: {}", name, e);
            }
        }
    }
}

