#![allow(dead_code)]
#![feature(std_misc)]
#![feature(core)]
#![feature(collections)]

extern crate enum_set as enum_set;
extern crate rand;

pub mod linalg;
pub mod film;
pub mod geometry;
pub mod sampler;
pub mod integrator;
pub mod scene;
pub mod bxdf;
pub mod material;
pub mod light;
pub mod mc;

