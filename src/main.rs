//#![cfg_attr(feature = "unstable", feature(plugin))]
//#![cfg_attr(feature = "unstable", plugin(clippy))]

extern crate image;
extern crate rand;
extern crate docopt;
#[macro_use]
extern crate serde_derive;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate tray_rust;

use std::path::PathBuf;
use std::io::ErrorKind;
use std::time::SystemTime;

use docopt::Docopt;

use tray_rust::scene;
use tray_rust::exec::{self, Exec};
use tray_rust::exec::distrib;

static USAGE: &'static str = "
Usage:
    tray_rust <scenefile> [-o <path>] [-n <number>] [--start-frame <number>] [--end-frame <number>]
    tray_rust <scenefile> --master <workers>... [-o <path>] [--start-frame <number>] [--end-frame <number>]
    tray_rust --worker [-n <number>]
    tray_rust (-h | --help)


Options:
  -o <path>               Specify the output file or directory to save the image or frames. Supported formats are
                          PNG, JPG and PPM. Default is 'frame<#>.png'.
  -n <number>             Specify the number of threads to use for rendering. Defaults to the number of cores
                          on the system.
  --start-frame <number>  Specify frame to start rendering at, specifies an inclusive range [start, end]
  --end-frame <number>    Specify frame to stop rendering at, specifies an inclusive range [start, end]
  --master                Start a master process to manage the worker nodes in <workers>... for distributed
                          rendering. The master collects results from workers and saves the image(s).
  <workers>...            Specify the list of worker nodes the master will connect too.
  --worker                Start a worker process that will listen for a master process to contact it and
                          instruct on what to start rendering. The worker will report its results back to
                          the master.
  -h, --help              Show this message.
";

#[derive(Deserialize, Debug)]
struct Args {
    arg_scenefile: String,
    flag_o: Option<String>,
    flag_n: Option<u32>,
    flag_start_frame: Option<usize>,
    flag_end_frame: Option<usize>,
    flag_master: Option<bool>,
    arg_workers: Vec<String>,
    flag_worker: Option<bool>,
}

fn single_node_render(args: Args) {
    let num_threads = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };
    let out_path = match args.flag_o {
        Some(ref f) => {
            let p = PathBuf::from(f);
            // If we're writing to a directory make sure it exists
            if p.extension() == None {
                if let Err(e) = std::fs::create_dir(p.as_path()) {
                    if e.kind() != ErrorKind::AlreadyExists {
                        panic!("Failed to create output directory");
                    }
                }
            }
            p
        },
        None => PathBuf::from("./"),
    };

    let (mut scene, mut rt, spp, mut frame_info) = scene::Scene::load_file(&args.arg_scenefile[..]);
    let dim = rt.dimensions();

    frame_info.start = match args.flag_start_frame {
        Some(x) => x,
        _ => frame_info.start,
    };
    frame_info.end = match args.flag_end_frame {
        Some(x) => x,
        _ => frame_info.end,
    };
    let scene_start = SystemTime::now();
    let mut config = exec::Config::new(out_path, args.arg_scenefile, spp, num_threads, frame_info, (0, 0));
    let mut exec = exec::MultiThreaded::new(num_threads);
    for i in frame_info.start..frame_info.end + 1 {
        config.current_frame = i;
        exec.render(&mut scene, &mut rt, &config);

        let img = rt.get_render();
        let out_file = match config.out_path.extension() {
            Some(_) => config.out_path.clone(),
            None => config.out_path.join(PathBuf::from(format!("frame{:05}.png", i))),
        };
        match image::save_buffer(&out_file.as_path(), &img[..], dim.0 as u32, dim.1 as u32, image::RGB(8)) {
            Ok(_) => {},
            Err(e) => println!("Error saving image, {}", e),
        };
        rt.clear();
        println!("Frame {}: rendered to '{}'\n--------------------", i, out_file.display());
    }
    let time = scene_start.elapsed().expect("Failed to get render time?");
    println!("Rendering entire sequence took {:4}s", time.as_secs() as f64 + time.subsec_nanos() as f64 * 1e-9)
}

fn master_node(args: Args) {
    let out_path = match args.flag_o {
        Some(ref f) => {
            let p = PathBuf::from(f);
            // If we're writing to a directory make sure it exists
            if p.extension() == None {
                if let Err(e) = std::fs::create_dir(p.as_path()) {
                    if e.kind() != ErrorKind::AlreadyExists {
                        panic!("Failed to create output directory");
                    }
                }
            }
            p
        },
        None => PathBuf::from("./"),
    };

    let (_, rt, spp, mut frame_info) = scene::Scene::load_file(&args.arg_scenefile[..]);

    frame_info.start = match args.flag_start_frame {
        Some(x) => x,
        _ => frame_info.start,
    };
    frame_info.end = match args.flag_end_frame {
        Some(x) => x,
        _ => frame_info.end,
    };
    let scene_start = SystemTime::now();
    let config = exec::Config::new(out_path, args.arg_scenefile, spp, 0, frame_info, (0, 0));
    // Connect to all the workers and prepare to send/receive data from/to them
    let (mut master, mut event_loop) = distrib::Master::start_workers(args.arg_workers, config, rt.dimensions());
    // Start the event loop to wait for and read results from each worker. No
    event_loop.run(&mut master).unwrap();
    let time = scene_start.elapsed().expect("Failed to get render time?");
    println!("Rendering entire sequence took {:4}s", time.as_secs() as f64 + time.subsec_nanos() as f64 * 1e-9)
}

fn worker_node(args: Args) {
    let num_threads = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };
    let mut exec = exec::MultiThreaded::new(num_threads);
    // Get our instructions of what to render from the master
    let mut worker = distrib::Worker::listen_for_master(num_threads);
    let scene_start = SystemTime::now();
    for i in worker.config.frame_info.start..worker.config.frame_info.end + 1 {
        worker.config.current_frame = i;
        exec.render(&mut worker.scene, &mut worker.render_target, &worker.config);
        worker.send_results();
        worker.render_target.clear();
        println!("--------------------");
    }
    let time = scene_start.elapsed().expect("Failed to get render time?");
    println!("Rendering entire sequence took {:4}s", time.as_secs() as f64 + time.subsec_nanos() as f64 * 1e-9)
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    if Some(true) == args.flag_master {
        master_node(args);
    } else if Some(true) == args.flag_worker {
        worker_node(args);
    } else {
        single_node_render(args);
    }
}

