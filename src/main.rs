extern crate tray_rust;

use tray_rust::film;

fn main() {
    let mut rt = film::RenderTarget::new(2, 2);
    film::write_ppm("black_test.ppm", 2, 2, rt.get_render().as_slice());

    rt.write(0.5, 0.5, &film::Colorf::new(1.0, 0.0, 0.0));
    rt.write(1.5, 0.5, &film::Colorf::new(0.0, 1.0, 0.0));
    rt.write(0.5, 1.5, &film::Colorf::new(0.0, 0.0, 1.0));
    rt.write(1.5, 1.5, &film::Colorf::new(0.3, 0.3, 0.3));
    film::write_ppm("out.ppm", 2, 2, rt.get_render().as_slice());
}

