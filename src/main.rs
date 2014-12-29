extern crate tray_rust;

use tray_rust::film;

fn main() {
    let img = vec![255u8, 0u8, 0u8, 0u8, 255u8, 0u8,
                   0u8, 0u8, 255u8, 255u8, 255u8, 255u8];
    film::write_ppm("out.ppm", 2, 2, img.as_slice());
}

