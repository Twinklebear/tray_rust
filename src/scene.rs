//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.
//!
//! # Scene JSON Files
//! The scene file format has four required sections: a camera, an integrator,
//! a list of materials and a list of objects and lights. The root object in the
//! JSON file should contain one of each of these.
//!
//! ```json
//! {
//!     "camera": {...},
//!     "integrator": {...},
//!     "materials": [...],
//!     "objects": [...]
//! }
//! ```
//!
//! For more information on each object see the corresponding modules:
//!
//! - Camera: See film/camera
//! - Integrator: See integrator
//! - Materials: See materials
//! - Objects: See geometry
//!

use std::io::prelude::*;
use std::fs::File;
use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use serde_json::{self, Value};

use linalg::{Transform, Point, Vector, Ray, Keyframe, AnimatedTransform};
use film::{filter, Camera, Colorf, RenderTarget, FrameInfo, AnimatedColor, ColorKeyframe};
use geometry::{Sphere, Instance, Intersection, BVH, Mesh, Disk, Rectangle,
               BoundableGeom, SampleableGeom};
use material::{Material, Matte, Glass, Metal, Merl, Plastic, SpecularMetal};
use integrator::{self, Integrator};

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub cameras: Vec<Camera>,
    active_camera: usize,
    pub bvh: BVH<Instance>,
    pub integrator: Box<Integrator + Send + Sync>,
}

impl Scene {
    pub fn load_file(file: &str) -> (Scene, RenderTarget, usize, FrameInfo) {
        let mut f = match File::open(file) {
            Ok(f) => f,
            Err(e) => panic!("Failed to open scene file: {}", e),
        };
        let mut content = String::new();
        match f.read_to_string(&mut content) {
            Err(e) => panic!("Failed to read scene file: {}", e),
            _ => {}
        }
        // Why not use expect here?
        let data: Value = match serde_json::from_str(&content[..]) {
            Ok(d) => d,
            Err(e) => panic!("JSON parsing error: {}", e),
        };
        assert!(data.is_object(), "Expected a root JSON object. See example scenes");
        let path = match Path::new(file).parent() {
            Some(p) => p,
            None => Path::new(file),
        };

        let (rt, spp, frame_info) = load_film(data.find("film").expect("The scene must specify a film to write to"));
        let cameras = load_cameras(&data, rt.dimensions());
        let integrator = load_integrator(data.find("integrator")
                                         .expect("The scene must specify the integrator to render with"));
        let materials = load_materials(&path, data.find("materials")
                                       .expect("The scene must specify an array of materials"));
        // mesh cache is a map of file_name -> (map of mesh name -> mesh)
        let mut mesh_cache = HashMap::new();
        let instances = load_objects(&path, &materials, &mut mesh_cache,
                                     data.find("objects").expect("The scene must specify a list of objects"));

        assert!(!instances.is_empty(), "Aborting: the scene does not have any objects!");
        let scene = Scene {
            cameras: cameras,
            active_camera: 0,
            // TODO: Read time parameters from the scene file, update BVH every few frames
            bvh: BVH::new(4, instances, 0.0, frame_info.time),
            integrator: integrator,
        };
        (scene, rt, spp, frame_info)
    }
    /// Test the ray for intersections against the objects in the scene.
    /// Returns Some(Intersection) if an intersection was found and None if not.
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
    /// Advance the time the scene is currently displaying to the time range passed
    pub fn update_frame(&mut self, frame: usize, start: f32, end: f32) {
        if self.active_camera != self.cameras.len() - 1 && self.cameras[self.active_camera + 1].active_at == frame {
            self.active_camera += 1;
            println!("Changing to camera {}", self.active_camera);
        }
        self.cameras[self.active_camera].update_frame(start, end);
        // TODO: How often to re-build the BVH?
        let shutter_time = self.cameras[self.active_camera].shutter_time();
        println!("Frame {}: re-building bvh for {} to {}", frame, shutter_time.0, shutter_time.1);
        self.bvh.rebuild(shutter_time.0, shutter_time.1);
    }
    /// Get the active camera for the current frame
    pub fn active_camera(&self) -> &Camera {
        &self.cameras[self.active_camera]
    }
}

/// Load the film described by the JSON value passed. Returns the render target
/// along with the image dimensions and samples per pixel
fn load_film(elem: &Value) -> (RenderTarget, usize, FrameInfo) {
    let width = elem.find("width").expect("The film must specify the image width")
        .as_u64().expect("Image width must be a number") as usize;
    let height = elem.find("height").expect("The film must specify the image height")
        .as_u64().expect("Image height must be a number") as usize;
    let spp = elem.find("samples").expect("The film must specify the number of samples per pixel")
        .as_u64().expect("Samples per pixel must be a number") as usize;
    let start_frame = elem.find("start_frame").expect("The film must specify the starting frame")
        .as_u64().expect("Start frame must be a number") as usize;
    let end_frame = elem.find("end_frame").expect("The film must specify the frame to end on")
        .as_u64().expect("End frame must be a number") as usize;
    if end_frame < start_frame {
        panic!("End frame must be greater or equal to the starting frame");
    }
    let frames = elem.find("frames").expect("The film must specify the total number of frames")
        .as_u64().expect("Frames must be a number") as usize;
    let scene_time = elem.find("scene_time").expect("The film must specify the overall scene time")
        .as_f64().expect("Scene time must be a number") as f32;
    let frame_info = FrameInfo::new(frames, scene_time, start_frame, end_frame);
    let filter = load_filter(elem.find("filter").expect("The film must specify a reconstruction filter"));
    (RenderTarget::new((width, height), (2, 2), filter), spp, frame_info)
}
/// Load the reconstruction filter described by the JSON value passed
fn load_filter(elem: &Value) -> Box<filter::Filter + Send + Sync> {
    let width = elem.find("width").expect("The filter must specify the filter width")
        .as_f64().expect("Filter width must be a number") as f32;
    let height = elem.find("height").expect("The filter must specify the filter height")
        .as_f64().expect("Filter height must be a number") as f32;
    let ty = elem.find("type").expect("A type is required for the filter")
        .as_string().expect("Filter type must be a string");
    if ty == "mitchell_netravali" {
        let b = elem.find("b").expect("A b parameter is required for the Mitchell-Netravali filter")
            .as_f64().expect("b must be a number") as f32;
        let c = elem.find("c").expect("A c parameter is required for the Mitchell-Netravali filter")
            .as_f64().expect("c must be a number") as f32;
        Box::new(filter::MitchellNetravali::new(width, height, b, c)) as Box<filter::Filter + Send + Sync>
    } else if ty == "gaussian" {
        let alpha = elem.find("alpha").expect("An alpha parameter is required for the Gaussian filter")
            .as_f64().expect("alpha must be a number") as f32;
        Box::new(filter::Gaussian::new(width, height, alpha)) as Box<filter::Filter + Send + Sync>
    } else {
        panic!("Unrecognized filter type {}!", ty);
    }
}

/// Load the cameras or single camera specified for this scene
fn load_cameras(elem: &Value, dim: (usize, usize)) -> Vec<Camera> {
    match elem.find("cameras") {
        Some(c) => {
            let cameras_json = match c.as_array() {
                Some(ca) => ca,
                None => panic!("cameras listing must be an array of cameras"),
            };
            let mut cameras = Vec::new();
            for cam in cameras_json {
                cameras.push(load_camera(cam, dim));
            }
            cameras.sort_by(|a, b| a.active_at.cmp(&b.active_at));
            cameras
        },
        None => vec![load_camera(elem.find("camera").expect("Error: A camera is required!"), dim)]
    }
}
/// Load the camera described by the JSON value passed.
/// Returns the camera along with the number of samples to take per pixel
/// and the scene dimensions. Panics if the camera is incorrectly specified
fn load_camera(elem: &Value, dim: (usize, usize)) -> Camera {
    let fov = elem.find("fov").expect("The camera must specify a field of view").as_f64()
        .expect("fov must be a float") as f32;
    let shutter_size = match elem.find("shutter_size") {
        Some(s) => s.as_f64().expect("Shutter size should be a float from 0 to 1") as f32,
        None => 0.5,
    };
    let active_at = match elem.find("active_at") {
        Some(s) => s.as_u64().expect("The camera activation frame 'active_at' must be an unsigned int") as usize,
        None => 0,
    };
    let transform = match elem.find("keyframes") {
        Some(t) => load_keyframes(t).expect("Invalid keyframes specified"),
        None => {
            let t = match elem.find("transform") {
                Some(t) => load_transform(t).expect("Invalid transform specified"),
                None => {
                    println!("Warning! Specifying transforms with pos, target and up vectors is deprecated!");
                    let pos = load_point(elem.find("position").expect("The camera must specify a position"))
                        .expect("position must be an array of 3 floats");
                    let target = load_point(elem.find("target").expect("The camera must specify a target"))
                        .expect("target must be an array of 3 floats");
                    let up = load_vector(elem.find("up").expect("The camera must specify an up vector"))
                        .expect("up must be an array of 3 floats");
                    Transform::look_at(&pos, &target, &up)
                }
            };
            AnimatedTransform::unanimated(&t)
        },
    };
    Camera::new(transform, fov, dim, shutter_size, active_at)
}

/// Load the integrator described by the JSON value passed.
/// Return the integrator or panics if it's incorrectly specified
fn load_integrator(elem: &Value) -> Box<Integrator + Send + Sync> {
    let ty = elem.find("type").expect("Integrator must specify a type")
        .as_string().expect("Integrator type must be a string");
    if ty == "pathtracer" {
        let min_depth = elem.find("min_depth").expect("The integrator must specify the minimum ray depth")
            .as_u64().expect("min_depth must be a number") as u32;
        let max_depth = elem.find("max_depth").expect("The integrator must specify the maximum ray depth")
            .as_u64().expect("max_depth must be a number") as u32;
        Box::new(integrator::Path::new(min_depth, max_depth))
    } else if ty == "whitted" {
        let min_depth = elem.find("min_depth").expect("The integrator must specify the minimum ray depth")
            .as_u64().expect("min_depth must be a number") as u32;
        Box::new(integrator::Whitted::new(min_depth))
    } else if ty == "normals_debug" {
        Box::new(integrator::NormalsDebug)
    } else {
        panic!("Unrecognized integrator type '{}'", ty);
    }
}

/// Generate a material loading error string
fn mat_error(mat_name: &String, msg: &str) -> String {
    format!("Error loading material '{}': {}", mat_name, msg)
}

/// Load the array of materials used in the scene, panics if a material is specified
/// incorrectly. The path to the directory containing the scene file is required to find
/// referenced material data relative to the scene file.
fn load_materials(path: &Path, elem: &Value) -> HashMap<String, Arc<Material + Send + Sync>> {
    let mut materials = HashMap::new();
    let mat_vec = elem.as_array().expect("The materials must be an array of materials used");
    for (i, m) in mat_vec.iter().enumerate() {
        let name = m.find("name").expect(&format!("Error loading material #{}: A name is required", i)[..])
            .as_string().expect(&format!("Error loading material #{}: name must be a string", i)[..])
            .to_string();
        let ty = m.find("type").expect(&mat_error(&name, "a type is required")[..])
            .as_string().expect(&mat_error(&name, "type must be a string")[..]);
        // Make sure names are unique to avoid people accidently overwriting materials
        if materials.contains_key(&name) {
            panic!("Error loading material '{}': name conflicts with an existing entry", name);
        }
        if ty == "glass" {
            let reflect = load_color(m.find("reflect")
                                     .expect(&mat_error(&name, "A reflect color is required for glass")[..]))
                .expect(&mat_error(&name, "Invalid color specified for reflect of glass")[..]);
            let transmit = load_color(m.find("transmit")
                                      .expect(&mat_error(&name, "A transmit color is required for glass")[..]))
                .expect(&mat_error(&name, "Invalid color specified for transmit of glass")[..]);
            let eta = m.find("eta")
                .expect(&mat_error(&name, "A refractive index 'eta' is required for glass")[..]).as_f64()
                .expect(&mat_error(&name, "glass eta must be a float")[..]) as f32;
            materials.insert(name, Arc::new(Glass::new(&reflect, &transmit, eta)) as Arc<Material + Send + Sync>);
        } else if ty == "matte" {
            let diffuse = load_color(m.find("diffuse")
                                     .expect(&mat_error(&name, "A diffuse color is required for matte")[..]))
                .expect(&mat_error(&name, "Invalid color specified for diffuse of matte")[..]);
            let roughness = m.find("roughness")
                .expect(&mat_error(&name, "A roughness is required for matte")[..]).as_f64()
                .expect(&mat_error(&name, "roughness must be a float")[..]) as f32;
            materials.insert(name, Arc::new(Matte::new(&diffuse, roughness)) as Arc<Material + Send + Sync>);
        } else if ty == "merl" {
            let file_path = Path::new(m.find("file")
                      .expect(&mat_error(&name, "A filename containing the MERL material data is required")[..])
                      .as_string().expect(&mat_error(&name, "The MERL file must be a string")[..]));
            if file_path.is_relative() {
                materials.insert(name, Arc::new(Merl::load_file(path.join(file_path).as_path()))
                                 as Arc<Material + Send + Sync>);
            } else {
                materials.insert(name, Arc::new(Merl::load_file(&file_path)) as Arc<Material + Send + Sync>);
            }
        } else if ty == "metal" {
            let refr_index = load_color(m.find("refractive_index")
                            .expect(&mat_error(&name, "A refractive_index color is required for metal")[..]))
                .expect(&mat_error(&name, "Invalid color specified for refractive_index of metal")[..]);
            let absorption_coef = load_color(m.find("absorption_coefficient")
                         .expect(&mat_error(&name, "An absorption_coefficient color is required for metal")[..]))
                .expect(&mat_error(&name, "Invalid color specified for absorption_coefficient of metal")[..]);
            let roughness = m.find("roughness")
                .expect(&mat_error(&name, "A roughness is required for metal")[..]).as_f64()
                .expect(&mat_error(&name, "roughness must be a float")[..]) as f32;
            materials.insert(name, Arc::new(Metal::new(&refr_index, &absorption_coef, roughness))
                             as Arc<Material + Send + Sync>);
        } else if ty == "plastic" {
            let diffuse = load_color(m.find("diffuse")
                             .expect(&mat_error(&name, "A diffuse color is required for plastic")[..]))
                .expect(&mat_error(&name, "Invalid color specified for diffuse of plastic")[..]);
            let gloss = load_color(m.find("gloss")
                             .expect(&mat_error(&name, "A gloss color is required for plastic")[..]))
                .expect(&mat_error(&name, "Invalid color specified for gloss of plastic")[..]);
            let roughness = m.find("roughness")
                .expect(&mat_error(&name, "A roughness is required for plastic")[..]).as_f64()
                .expect(&mat_error(&name, "roughness must be a float")[..]) as f32;
            materials.insert(name, Arc::new(Plastic::new(&diffuse, &gloss, roughness))
                             as Arc<Material + Send + Sync>);
        } else if ty == "specular_metal" {
            let refr_index = load_color(m.find("refractive_index")
                    .expect(&mat_error(&name, "A refractive_index color is required for specular metal")[..]))
                .expect(&mat_error(&name, "Invalid color specified for refractive_index of specular metal")[..]);
            let absorption_coef = load_color(m.find("absorption_coefficient")
                     .expect(&mat_error(&name,
                                        "An absorption_coefficient color is required for specular metal")[..]))
                .expect(&mat_error(&name,
                                   "Invalid color specified for absorption_coefficient of specular metal")[..]);
            materials.insert(name, Arc::new(SpecularMetal::new(&refr_index, &absorption_coef))
                             as Arc<Material + Send + Sync>);
        } else {
            panic!("Error parsing material '{}': unrecognized type '{}'", name, ty);
        }
    }
    materials
}

/// Loads the array of objects in the scene, assigning them materials from the materials map. Will
/// panic if an incorrectly specified object is found.
fn load_objects(path: &Path, materials: &HashMap<String, Arc<Material + Send + Sync>>,
                mesh_cache: &mut HashMap<String, HashMap<String, Arc<Mesh>>>, elem: &Value)
                -> Vec<Instance> {
    let mut instances = Vec::new();
    let objects = elem.as_array().expect("The objects must be an array of objects used");
    for o in objects {
        let name = o.find("name").expect("A name is required for an object")
            .as_string().expect("Object name must be a string").to_string();
        let ty = o.find("type").expect("A type is required for an object")
            .as_string().expect("Object type must be a string");

        let transform = match o.find("keyframes") {
            Some(t) => load_keyframes(t).expect("Invalid keyframes specified"),
            None => {
                let t = match o.find("transform") {
                    Some(t) => load_transform(t).expect("Invalid transform specified"),
                    None => panic!("No keyframes or transform specified for object {}", name),
                };
                AnimatedTransform::unanimated(&t)
            },
        };
        if ty == "emitter" {
            let emit_ty = o.find("emitter").expect("An emitter type is required for emitters")
                .as_string().expect("Emitter type must be a string");
            let emission = load_animated_color(o.find("emission")
                    .expect("An emission color is required for emitters"))
                    .expect("Emitter emission must be a color");
            if emit_ty == "point" {
                instances.push(Instance::point_light(transform, emission, name));
            } else if emit_ty == "area" {
                let mat_name = o.find("material").expect("A material is required for an object")
                    .as_string().expect("Object material name must be a string");
                let mat = materials.get(mat_name)
                    .expect("Material was not found in the material list").clone();
                let geom = load_sampleable_geometry(o.find("geometry")
                                                    .expect("Geometry is required for area lights"));

                instances.push(Instance::area_light(geom, mat, emission, transform, name));
            } else {
                panic!("Invalid emitter type specified: {}", emit_ty);
            }
        } else if ty == "receiver" {
            let mat_name = o.find("material").expect("A material is required for an object")
                    .as_string().expect("Object material name must be a string");
            let mat = materials.get(mat_name)
                .expect("Material was not found in the material list").clone();
            let geom = load_geometry(path, mesh_cache, o.find("geometry")
                                     .expect("Geometry is required for receivers"));

            instances.push(Instance::receiver(geom, mat, transform, name));
        } else if ty == "group" {
            let group_objects = o.find("objects").expect("A group must specify an array of objects in the group");
            let group_instances = load_objects(path, materials, mesh_cache, group_objects);
            for mut gi in group_instances {
                {
                    let t = gi.get_transform().clone();
                    gi.set_transform(transform.clone() * t);
                }
                instances.push(gi);
            }
        } else {
            panic!("Error parsing object '{}': unrecognized type '{}'", name, ty);
        }
    }
    instances
}

/// Load the geometry specified by the JSON value. Will re-use any already loaded meshes
/// and will place newly loaded meshees in the mesh cache.
fn load_geometry(path: &Path, meshes: &mut HashMap<String, HashMap<String, Arc<Mesh>>>, elem: &Value)
             -> Arc<BoundableGeom + Send + Sync> {
    let ty = elem.find("type").expect("A type is required for geometry")
        .as_string().expect("Geometry type must be a string");
    if ty == "sphere" {
        let r = elem.find("radius").expect("A radius is required for a sphere").as_f64()
            .expect("radius must be a number") as f32;
        Arc::new(Sphere::new(r))
    } else if ty == "disk" {
        let r = elem.find("radius").expect("A radius is required for a disk").as_f64()
            .expect("radius must be a number") as f32;
        let ir = elem.find("inner_radius").expect("An inner radius is required for a disk").as_f64()
            .expect("inner radius must be a number") as f32;
        Arc::new(Disk::new(r, ir))
    } else if ty == "plane" {
        // We just treat plane as a special case of Rectangle now
        Arc::new(Rectangle::new(2.0, 2.0))
    } else if ty == "rectangle" {
        let width = elem.find("width").expect("A width is required for a rectangle").as_f64()
            .expect("width must be a number") as f32;
        let height = elem.find("height").expect("A height is required for a rectangle").as_f64()
            .expect("height must be a number") as f32;
        Arc::new(Rectangle::new(width, height))
    } else if ty == "mesh" {
        let mut file = Path::new(elem.find("file").expect("An OBJ file is required for meshes")
            .as_string().expect("OBJ filename must be a string")).to_path_buf();
        let model = elem.find("model").expect("A model name is required for geometry")
            .as_string().expect("Model name type must be a string");

        if file.is_relative() {
            file = path.join(file);
        }
        let file_string = file.to_str().expect("Invalid file name");
        if meshes.get(file_string).is_none() {
            meshes.insert(file_string.to_string(), Mesh::load_obj(Path::new(&file)));
        }
        let file_meshes = &meshes[file_string];
        match file_meshes.get(model) {
            Some(m) => m.clone(),
            None => panic!("Requested model '{}' was not found in '{:?}'", model, file),
        }
    } else {
        panic!("Unrecognized geometry type '{}'", ty);
    }
}

/// Load the sampleable geometry specified by the JSON value. Will panic if the geometry specified
/// is not sampleable.
fn load_sampleable_geometry(elem: &Value) -> Arc<SampleableGeom + Send + Sync> {
    let ty = elem.find("type").expect("A type is required for geometry")
        .as_string().expect("Geometry type must be a string");
    if ty == "sphere" {
        let r = elem.find("radius").expect("A radius is required for a sphere").as_f64()
            .expect("radius must be a number") as f32;
        Arc::new(Sphere::new(r))
    } else if ty == "disk" {
        let r = elem.find("radius").expect("A radius is required for a disk").as_f64()
            .expect("radius must be a number") as f32;
        let ir = elem.find("inner_radius").expect("An inner radius is required for a disk").as_f64()
            .expect("inner radius must be a number") as f32;
        Arc::new(Disk::new(r, ir))
    } else if ty == "rectangle" {
        let width = elem.find("width").expect("A width is required for a rectangle").as_f64()
            .expect("width must be a number") as f32;
        let height = elem.find("height").expect("A height is required for a rectangle").as_f64()
            .expect("height must be a number") as f32;
        Arc::new(Rectangle::new(width, height))
    } else {
        panic!("Geometry of type '{}' is not sampleable and can't be used for area light geometry", ty);
    }
}

/// Load a vector from the JSON element passed. Returns None if the element
/// did not contain a valid vector (eg. [1.0, 2.0, 0.5])
fn load_vector(elem: &Value) -> Option<Vector> {
    let array = match elem.as_array() {
        Some(a) => a,
        None => return None,
    };
    if array.len() != 3 {
        return None;
    }
    let mut v = [0.0f32; 3];
    for (i, x) in array.iter().enumerate() {
        match x.as_f64() {
            Some(f) => v[i] = f as f32,
            None => return None,
        }
    }
    Some(Vector::new(v[0], v[1], v[2]))
}

/// Load a point from the JSON element passed. Returns None if the element
/// did not contain a valid point (eg. [1.0, 2.0, 0.5])
fn load_point(elem: &Value) -> Option<Point> {
    let array = match elem.as_array() {
        Some(a) => a,
        None => return None,
    };
    if array.len() != 3 {
        return None;
    }
    let mut v = [0.0f32; 3];
    for (i, x) in array.iter().enumerate() {
        match x.as_f64() {
            Some(f) => v[i] = f as f32,
            None => return None,
        }
    }
    Some(Point::new(v[0], v[1], v[2]))
}

/// Load a color from the JSON element passed. Returns None if the element
/// did not contain a valid color.
fn load_color(elem: &Value) -> Option<Colorf> {
    let array = match elem.as_array() {
        Some(a) => a,
        None => return None,
    };
    if array.len() != 3 && array.len() != 4 {
        return None;
    }
    let mut v = Vec::with_capacity(4);
    for x in array.iter() {
        match x.as_f64() {
            Some(f) => v.push(f as f32),
            None => return None,
        }
    }
    let mut c = Colorf::new(v[0], v[1], v[2]);
    if v.len() == 4 {
        c = c * v[3];
    }
    Some(c)
}

/// Load an animated color from the JSON element passed. Returns None if the
/// element did not contain a valid color
fn load_animated_color(elem: &Value) -> Option<AnimatedColor> {
    let array = match elem.as_array() {
        Some(a) => a,
        None => return None,
    };
    if array.is_empty() {
        return None;
    }
    // Check if this is actually just a single color value
    if array[0].is_number() {
       match load_color(elem) {
            Some(c) => Some(AnimatedColor::with_keyframes(vec![ColorKeyframe::new(&c, 0.0)])),
            None => None,
        }
    } else {
        let mut v = Vec::new();
        for c in array.iter() {
            let time = c.find("time").expect("A time must be specified for a color keyframe").as_f64()
                .expect("Time for color keyframe must be a number") as f32;
            let color = load_color(c.find("color").expect("A color must be specified for a color keyframe"))
                .expect("A valid color is required for a color keyframe");
            v.push(ColorKeyframe::new(&color, time));
        }
        Some(AnimatedColor::with_keyframes(v))
    }
}

/// Load a transform stack specified by the element. Will panic on invalidly specified
/// transforms and log the error.
fn load_transform(elem: &Value) -> Option<Transform> {
    let array = match elem.as_array() {
        Some(a) => a,
        None => return None,
    };
    let mut transform = Transform::identity();
    for t in array {
        let ty = t.find("type").expect("A type is required for a transform")
            .as_string().expect("Transform type must be a string");
        if ty == "translate" {
            let v = load_vector(t.find("translation").expect("A translation vector is required for translate"))
                .expect("Invalid vector specified for translation direction");

            transform = Transform::translate(&v) * transform;
        } else if ty == "scale" {
            let s = t.find("scaling").expect("A scaling value or vector is required for scale");
            let v;
            if s.is_array() {
                v = load_vector(s).expect("Invalid vector specified for scaling vector");
            } else if s.is_number() {
                v = Vector::broadcast(s.as_f64().expect("Invalid float specified for scale value") as f32);
            } else {
                panic!("Scaling value should be an array of 3 floats or a single float");
            }

            transform = Transform::scale(&v) * transform;
        } else if ty == "rotate_x" {
            let r = t.find("rotation").expect("A rotation in degrees is required for rotate_x")
                .as_f64().expect("rotation for rotate_x must be a number") as f32;

            transform = Transform::rotate_x(r) * transform;
        } else if ty == "rotate_y" {
            let r = t.find("rotation").expect("A rotation in degrees is required for rotate_y")
                .as_f64().expect("rotation for rotate_y must be a number") as f32;

            transform = Transform::rotate_y(r) * transform;
        } else if ty == "rotate_z" {
            let r = t.find("rotation").expect("A rotation in degrees is required for rotate_z")
                .as_f64().expect("rotation for rotate_z must be a number") as f32;

            transform = Transform::rotate_z(r) * transform;
        } else if ty == "rotate" {
            let r = t.find("rotation").expect("A rotation in degrees is required for rotate")
                .as_f64().expect("rotation for rotate must be a number") as f32;
            let axis = load_vector(t.find("axis").expect("An axis vector is required for rotate"))
                .expect("Invalid vector specified for rotation axis");

            transform = Transform::rotate(&axis, r) * transform;
        } else if ty == "matrix" {
            // User has specified a pre-computed matrix for the transform
            let mat = t.find("matrix").expect("The rows of the matrix are required for matrix transform")
                .as_array().expect("The rows should be an array");
            let mut rows = Vec::with_capacity(16);
            for r in mat {
                let row = r.as_array().expect("Each row of the matrix transform must be an array, specifying the row");
                if row.len() != 4 {
                    panic!("Each row of the transformation matrix must contain 4 elements");
                }
                for e in row {
                    rows.push(e.as_f64().expect("Each element of a matrix row must be a float") as f32);
                }
            }

            transform = Transform::from_mat(&rows.iter().collect()) * transform;
        } else {
            println!("Unrecognized transform type '{}'", ty);
            return None;
        }
    }
    Some(transform)
}

/// Load a list of keyframes specified by the element. Will panic on invalidly
/// specified keyframes or transforms and log the error
fn load_keyframes(elem: &Value) -> Option<AnimatedTransform> {
    let points = match elem.find("control_points")
        .expect("Control points are required for bspline keyframes").as_array() {
            Some(a) => a,
            None => return None,
        };
    let knots_json = match elem.find("knots").expect("knots are required for bspline keyframes").as_array() {
        Some(a) => a,
        None => return None,
    };
    let mut keyframes = Vec::new();
    for t in points {
        let transform = load_transform(t.find("transform").expect("A transform is required for a keyframe"))
            .expect("Invalid transform for keyframe");
        keyframes.push(Keyframe::new(&transform));
    }
    let mut knots = Vec::new();
    for k in knots_json {
        knots.push(k.as_f64().expect("Knots must be numbers") as f32);
    }
    let degree = match elem.find("degree") {
        Some(d) => d.as_u64().expect("Curve degree must be a positive integer") as usize,
        None => 3,
    };
    Some(AnimatedTransform::with_keyframes(keyframes, knots, degree))
}

