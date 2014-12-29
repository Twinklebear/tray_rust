extern crate tray_rust;

use tray_rust::linalg;
use tray_rust::film;
use tray_rust::geometry;
use tray_rust::geometry::Geometry;

fn main() {
    let width = 800u;
    let height = 600u;
    let mut rt = film::RenderTarget::new(width, height);
    let camera = film::Camera::new(linalg::Transform::look_at(
        &linalg::Point::new(0.0, 0.0, -10.0), &linalg::Point::new(0.0, 0.0, 0.0),
        &linalg::Vector::new(0.0, 1.0, 0.0)), 40.0, rt.dimensions());
    let sphere = geometry::Sphere::new(1.5);
    {
        let instance = geometry::Instance::new(&sphere,
            linalg::Transform::translate(&linalg::Vector::new(0.0, 2.0, 0.0)));
        for y in range(0, height) {
            for x in range(0, width) {
                let px = (x as f32 + 0.5, y as f32 + 0.5);
                let mut ray = camera.generate_ray(px);
                match instance.intersect(&mut ray) {
                    Some(_) => rt.write(px.0, px.1, &film::Colorf::broadcast(1.0)),
                    None => {},
                }
            }
        }
    }
    film::write_ppm("out.ppm", width, height, rt.get_render().as_slice());
}

