extern crate tray_rust;

use tray_rust::linalg;

fn main() {
    let v = linalg::Vector::broadcast(1f32);
    println!("Hello, v = {}", v);
}

