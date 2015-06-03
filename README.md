tray\_rust - A Toy Ray Tracer in Rust
===
tray\_rust is a toy physically based ray tracer built off of the techniques
discussed in [Physically Based Rendering](http://pbrt.org/). It began life as a port of
[tray](https://github.com/Twinklebear/tray) to [Rust](http://www.rust-lang.org) to check out the language.
The renderer is currently capable of path tracing, supports triangle meshes (MTL support coming soon),
and various physically based material models (including measured data from the
[MERL BRDF Database](http://www.merl.com/brdf/)).

Building
---
Unfortunately the project does currently require Rust nightly as I make use of some unstable features
(eg. Vec::resize). I hope to get things running on stable Rust once these features get stabilized.

[![Build Status](https://travis-ci.org/Twinklebear/tray_rust.svg?branch=master)](https://travis-ci.org/Twinklebear/tray_rust)

Running
---
Currently the scene data is hardcoded in `src/scene.rs`, in the future I plan to add support for some
kind of JSON based scene file format. I strongly recommend running the release build as the debug version
will be very very slow. Running and passing `--help` or `-h` will print out some options, currently
the image resolution and samples per pixel are also hardcoded, though these are in `src/main.rs`.

Documentation
---
Documentation can be found on the [project site](http://www.willusher.io/tray_rust/tray_rust/).

TODO
---
- Area light support
- More material models (eg. more microfacet models, rough glass, etc.)
- Textures
- Support for using an OBJ's associated MTL files
- Bump mapping
- [Subsurface scattering?](http://en.wikipedia.org/wiki/Subsurface_scattering)
- [Vertex Connection and Merging?](http://iliyan.com/publications/VertexMerging)

Sample Renders
---
I used tray\_rust to render the Rust logo with some friends from the computer graphics community,
the Buddha and Dragon from [The Stanford Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/).
The Rust logo model was made by
[Nylithius on BlenderArtists](http://blenderartists.org/forum/showthread.php?362836-Rust-language-3D-logo).
The Rust logo model is 28,844 triangles, the Buddha is 1,087,474 and the Dragon is 871,306. These timings
are also before I've fixed my BVH build so they could probably be improved somewhat.
Render times are formatted as hh:mm:ss and were measured using 32 threads on a machine with dual
[Xeon E5-2680's @ 2.7GHz](http://ark.intel.com/products/64583/Intel-Xeon-Processor-E5-2680-20M-Cache-2_70-GHz-8_00-GTs-Intel-QPI).
The materials on the Rust logo, Buddha and Dragon are from the [MERL BRDF Database](http://www.merl.com/brdf/).

[![Rust Logo with friends](http://i.imgur.com/9QU6fOU.png)](http://i.imgur.com/9QU6fOU.png)

1920x1080, 2048 samples/pixel. Rendering took 01:13:52.13.

[![Rust Logo](http://i.imgur.com/JouSgr5.png)](http://i.imgur.com/JouSgr5.png)

800x600, 1024 samples/pixel. Rendering took 00:09:00.208.

[![Smallpt](http://i.imgur.com/fUEv6Au.png)](http://i.imgur.com/fUEv6Au.png)

800x600, 1024 samples/pixel. Rendering took: 00:03:15.86.

