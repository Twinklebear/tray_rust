#![allow(dead_code)]
#![feature(float_extras, vec_resize)]

//! # tray\_rust - A Toy Ray Tracer in Rust
//! 
//! tray\_rust is a toy physically based ray tracer built off of the techniques
//! discussed in [Physically Based Rendering](http://pbrt.org/). It began life as a port of
//! [tray](https://github.com/Twinklebear/tray) to [Rust](http://www.rust-lang.org) to check out the language.
//! The renderer is currently capable of path tracing, supports triangle meshes (MTL support coming soon),
//! and various physically based material models (including measured data from the
//! [MERL BRDF Database](http://www.merl.com/brdf/)).
//! 
//! ## Building
//! 
//! Unfortunately the project does currently require Rust nightly as I make use of some unstable features
//! (eg. Vec::resize). I hope to get things running on stable Rust once these features get stabilized.
//! 
//! [![Build Status](https://travis-ci.org/Twinklebear/tray_rust.svg?branch=master)](https://travis-ci.org/Twinklebear/tray_rust)
//! 
//! ## Running
//! 
//! Currently the scene data is hardcoded in `src/scene.rs`, in the future I plan to add support for some
//! kind of JSON based scene file format. I strongly recommend running the release build as the debug version
//! will be very very slow. Running and passing `--help` or `-h` will print out some options, currently
//! the image resolution and samples per pixel are also hardcoded, though these are in `src/main.rs`.
//! 
//! ## Building Your Own Scenes
//!
//! Start at the documentation for the scene module.
//! 
//! ## TODO
//!
//! - More material models (eg. more microfacet models, rough glass, etc.)
//! - Textures
//! - Support for using an OBJ's associated MTL files
//! - Bump mapping
//! - [Subsurface scattering?](http://en.wikipedia.org/wiki/Subsurface_scattering)
//! - [Vertex Connection and Merging?](http://iliyan.com/publications/VertexMerging)
//! 
//! ## Sample Renders
//!
//! In the samples the the Buddha, Dragon, Bunny and Lucy statue are from
//! [The Stanford Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/).
//! The Rust logo model was made by
//! [Nylithius on BlenderArtists](http://blenderartists.org/forum/showthread.php?362836-Rust-language-3D-logo).
//! The Utah teapot used is from [Morgan McGuire's page](http://graphics.cs.williams.edu/data/meshes.xml) and
//! the monkey head is Blender's Suzanne. I've made minor tweaks to some of the models so for convenience
//! you can find versions that can be easily loaded into the sample scenes [here](https://drive.google.com/folderview?id=0B-l_lLEMo1YeflUzUndCd01hOHhRNUhrQUowM3hVd2pCc3JrSXRiS3FQSzRYLWtGcGM0eGc&usp=sharing), though the
//! cube model for the Cornell box scene is included.
//! The materials on the Rust logo, Buddha, Dragon and Lucy are from the
//! [MERL BRDF Database](http://www.merl.com/brdf/).
//!
//! Render times are formatted as hh:mm:ss and were measured using 144 threads on a machine with four
//! [Xeon E7-8890 v3](http://ark.intel.com/products/84685/Intel-Xeon-Processor-E7-8890-v3-45M-Cache-2_50-GHz)
//! CPUs. The machine is an early/engineering sample from Intel so your results may differ, but massive thanks to
//! Intel for the hardware! Some older images renders are shown as well without timing since they were
//! run on a different machine.
//!
//! <a href="http://i.imgur.com/Jgji1Y0.jpg">
//! <img src="http://i.imgur.com/Jgji1Y0.png" alt="Model gallery"
//!     style="display:block; max-width:100%; height:auto">
//! </a>
//! 
//! 1920x1080, 4096 samples/pixel. Rendering: 00:43:36.45.
//! 
//! <a href="http://i.imgur.com/Z34BleZ.jpg">
//! <img src="http://i.imgur.com/Z34BleZ.png" alt="Rust Logo with friends, disk"
//!     style="display:block; max-width:100%; height:auto">
//! </a>
//! 
//! 1920x1080, 4096 samples/pixel. Rendering: 00:49:33.514.
//! 
//! <a href="http://i.imgur.com/CUvJwum.jpg">
//! <img src="http://i.imgur.com/CUvJwum.png" alt="Cornell Box"
//!     style="display:block; max-width:100%; height:auto">
//! </a>
//! 
//! 800x600, 4096 samples/pixel. Rendering: 00:10:05.409.
//!

extern crate enum_set as enum_set;
extern crate rand;
extern crate byteorder;
extern crate serde_json;

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
pub mod partition;

