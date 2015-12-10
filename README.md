tray\_rust - A Toy Ray Tracer in Rust
===
tray\_rust is a toy physically based ray tracer built off of the techniques
discussed in [Physically Based Rendering](http://pbrt.org/). It began life as a port of
[tray](https://github.com/Twinklebear/tray) to [Rust](http://www.rust-lang.org) to check out the language
but has surpassed it in a few ways.
The renderer is currently capable of path tracing, supports triangle meshes (MTL support coming soon),
and various physically based material models (including measured data from the
[MERL BRDF Database](http://www.merl.com/brdf/)). tray\_rust also supports rigid body animation along
B-spline paths and distributed rendering.

[![Build Status](https://travis-ci.org/Twinklebear/tray_rust.svg?branch=master)](https://travis-ci.org/Twinklebear/tray_rust)

Running
---
Running and passing `--help` or `-h` will print out options you can pass to the renderer which are documented in the help.
For the more complicated use cases I hope to do some write ups and guides on how to use them (e.g. distributed rendering,
animation) but this may take a while. I strongly recommend running the release build as the debug version will be very slow.

Building Your Own Scenes
---
Start at the documentation for the [scene module](http://www.willusher.io/tray_rust/tray_rust/scene/index.html),
there are also a few example [scenes](scenes/) included but not all the models are provided. From a clean `git clone` you
should be able to run [cornell\_box.json](scenes/cornell_box.json) and [smallpt.json](scenes/smallpt.json). I plan to add some
more simple scenes that show usage of other features like animation to provide examples. The rigid body animation
feature is relatively new though so I haven't had time to document it properly yet.

Documentation
---
Documentation can be found on the [project site](http://www.willusher.io/tray_rust/tray_rust/).

TODO
---
- More material models (eg. more microfacet models, rough glass, etc.)
- Textures
- Support for using an OBJ's associated MTL files
- Bump mapping
- [Subsurface scattering?](http://en.wikipedia.org/wiki/Subsurface_scattering)
- [Vertex Connection and Merging?](http://iliyan.com/publications/VertexMerging)

Sample Renders
---
In the samples the the Buddha, Dragon, Bunny and Lucy statue are from
[The Stanford Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/).
The Rust logo model was made by
[Nylithius on BlenderArtists](http://blenderartists.org/forum/showthread.php?362836-Rust-language-3D-logo).
The Utah teapot used is from [Morgan McGuire's page](http://graphics.cs.williams.edu/data/meshes.xml) and
the monkey head is Blender's Suzanne. I've made minor tweaks to some of the models so for convenience
you can find versions that can be easily loaded into the sample scenes [here](https://drive.google.com/folderview?id=0B-l_lLEMo1YeflUzUndCd01hOHhRNUhrQUowM3hVd2pCc3JrSXRiS3FQSzRYLWtGcGM0eGc&usp=sharing), though the
cube model for the Cornell box scene is included.
The materials on the Rust logo, Buddha, Dragon and Lucy are from the
[MERL BRDF Database](http://www.merl.com/brdf/).

Render times are formatted as hh:mm:ss and were measured using 144 threads on a machine with four
[Xeon E7-8890 v3](http://ark.intel.com/products/84685/Intel-Xeon-Processor-E7-8890-v3-45M-Cache-2_50-GHz)
CPUs. The machine is an early/engineering sample from Intel so your results may differ, but massive thanks to
Intel for the hardware! Some older renders are shown as well without timing since they were
run on a different machine.

[![Model gallery](http://i.imgur.com/X5y8oIq.png)](http://i.imgur.com/X5y8oIq.png)

1920x1080, 4096 samples/pixel. Rendering: 00:43:36.45.

[![Rust Logo with friends, disk](http://i.imgur.com/E1ylrZW.png)](http://i.imgur.com/E1ylrZW.png)

1920x1080, 4096 samples/pixel. Rendering: 00:49:33.514.

[![Cornell Box](http://i.imgur.com/CUvJwum.png)](http://i.imgur.com/CUvJwum.png)

800x600, 4096 samples/pixel. Rendering: 00:10:05.409.

[![Suzanne Box](http://i.imgur.com/RbCNrxX.png)](http://i.imgur.com/RbCNrxX.png)

800x600, 4096 samples/pixel. Rendering: 00:11:23.67.

[![Ajax Bust](http://i.imgur.com/2lcANyn.png)](http://i.imgur.com/2lcANyn.png)

