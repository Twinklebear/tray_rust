extern crate image;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize;
extern crate num_cpus;
extern crate scoped_threadpool;
extern crate clock_ticks;
extern crate tray_rust;

use std::path::PathBuf;
use std::io::ErrorKind;

use docopt::Docopt;

use tray_rust::{scene, exec};
use tray_rust::exec::Exec;

static USAGE: &'static str = "
Usage: tray_rust <scenefile> [options]

Options:
  -o <file>               Specify the output file or directory to save the image or frames. Supported formats are
                          PNG, JPG and PPM. Default is 'frame<#>.png'.
  -n <number>             Specify the number of threads to use for rendering. Defaults to the number of cores
                          on the system.
  --start-frame <number>  Specify frame to start rendering at, specifies an inclusive range [start, end]
  --end-frame <number>    Specify frame to stop rendering at, specifies an inclusive range [start, end]
  -h, --help              Show this message.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_scenefile: String,
    flag_o: Option<String>,
    flag_n: Option<u32>,
    flag_start_frame: Option<usize>,
    flag_end_frame: Option<usize>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    let num_threads = match args.flag_n {
        Some(n) => n,
        None => num_cpus::get() as u32,
    };
    let out_path = match &args.flag_o {
        &Some(ref f) => {
            let p = PathBuf::from(f);
            // If we're writing to a directory make sure it exists
            if p.extension() == None {
                match std::fs::create_dir(p.as_path()) {
                    Err(e) => {
                        if e.kind() != ErrorKind::AlreadyExists {
                            panic!("Failed to create output directory");
                        }
                    },
                    Ok(_) => {},
                }
            }
            p
        },
        &None => PathBuf::from("./"),
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

    let scene_start = clock_ticks::precise_time_s();
    let mut config = exec::Config::new(out_path, spp, num_threads, frame_info, (0, 0));
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
    let time = clock_ticks::precise_time_s() - scene_start;
    println!("Rendering entire sequence took {}s", time);
}

