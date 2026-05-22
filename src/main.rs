mod core;
mod overlay_text_providers;
mod video_display;
mod wrappers;

use crate::core::OmniPane;
use crate::overlay_text_providers::{
    OverlayTextProvider, TemperatureOverlayTextProvider, TimeOverlayTextProvider,
};
use crate::wrappers::ImageBuffer;
use opencv::core::{Mat, UMat};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};

struct Arguments {
    pub channel_urls: Vec<String>,
    pub overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    pub benchmark: bool,
    pub use_gpu: bool,
}

fn main() {
    println!("Starting video streaming...");

    if let Some(args) = read_arguments() {
        if args.use_gpu {
            opencv::core::set_use_opencl(true).unwrap();

            if opencv::core::have_opencl().unwrap() && opencv::core::use_opencl().unwrap() {
                println!("GPU acceleration with opencl enabled.");
            } else {
                panic!("Unable to use GPU acceleration. OpenCL not available.")
            }
            run_omni_pane::<UMat>(args);
        } else {
            run_omni_pane::<Mat>(args);
        }
    } else {
        panic!("No video stream specified!");
    }
}

fn run_omni_pane<T: ImageBuffer>(args: Arguments) {
    let mut omni_pane = OmniPane::<T>::new(args.channel_urls, args.overlay_providers);

    if !args.benchmark {
        camera_switcher(
            omni_pane.current_camera_index.clone(),
            omni_pane.get_n_channels(),
            omni_pane.running.clone(),
        );
    }

    omni_pane.start();

    // TODO: improve services exit sync
    omni_pane.running.store(false, Ordering::Relaxed);
}

fn read_arguments() -> Option<Arguments> {
    let mut args: VecDeque<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} \"URL\"", args[0]);
        return None;
    }

    // executable's path
    let _ = args.pop_front();

    let mut arguments = Arguments {
        channel_urls: vec![],
        overlay_providers: vec![],
        benchmark: false,
        use_gpu: false,
    };

    while !args.is_empty() {
        let parameter = args.pop_front().unwrap();
        if parameter.eq_ignore_ascii_case("--overlay:time") {
            arguments
                .overlay_providers
                .push(Box::new(TimeOverlayTextProvider::new()));
        } else if parameter.starts_with("--overlay:temperature=") {
            // TODO: make a better argument parsing mechanism
            let sensor_id = parameter.split("--overlay:temperature=").last().unwrap();
            arguments
                .overlay_providers
                .push(Box::new(TemperatureOverlayTextProvider::new(sensor_id)));
        } else if parameter.eq_ignore_ascii_case("--gpu") {
            arguments.use_gpu = true;
        } else if parameter.eq_ignore_ascii_case("--benchmark") {
            arguments.benchmark = true;
        } else {
            arguments.channel_urls.push(parameter);
        }
    }

    if !arguments.channel_urls.is_empty() {
        Some(arguments)
    } else {
        None
    }
}

// TODO: create a proper channel selector mechanism
fn camera_switcher(camera_index: Arc<AtomicU8>, list_size: u8, running: Arc<AtomicBool>) {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(10));
            let mut current_index = camera_index.load(Ordering::Relaxed);
            current_index = (current_index + 1) % list_size;
            println!("Changing camera to index {}", current_index);
            camera_index.store(current_index, Ordering::Relaxed);
        }
    });
}
