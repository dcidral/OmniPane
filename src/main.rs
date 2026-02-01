mod core;
mod overlay_text_providers;
mod video_display;

use crate::core::OmniPane;
use crate::overlay_text_providers::{
    OverlayTextProvider, TemperatureOverlayTextProvider, TimeOverlayTextProvider,
};
use crate::video_display::video_channel::{VideoChannel, VideoChannelSettings};
use opencv::videoio::VideoCapture;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};

fn main() {
    println!("Starting video streaming...");

    if let Some((url_list, mut list_of_overlay_providers)) = read_arguments() {
        let mut channels: Vec<VideoChannel> = Vec::new();

        for url in url_list {
            // TODO: error handling
            let camera = VideoCapture::from_file(url.as_str(), opencv::videoio::CAP_ANY).unwrap();
            channels.push(VideoChannel::new(camera, VideoChannelSettings::default()));
            println!("Added camera for url {}", url.as_str());
        }

        // TODO: create a proper service stop mechanism
        let running = Arc::new(AtomicBool::new(true));

        for text_provider in &mut list_of_overlay_providers {
            text_provider.start_service(running.clone());
        }

        let n_channels = channels.len() as u8;
        let mut streamer = OmniPane::new(channels, list_of_overlay_providers);
        let camera_index = streamer.current_camera_index.clone();

        camera_switcher(camera_index, n_channels, running.clone());

        streamer.start_display(running.clone());

        // TODO: improve services exit sync
        running.store(false, Ordering::Relaxed);
    } else {
        panic!("No video stream specified!");
    }
}

fn read_arguments() -> Option<(Vec<String>, Vec<Box<dyn OverlayTextProvider>>)> {
    let mut args: VecDeque<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} \"URL\"", args[0]);
        return None;
    }

    // executable's path
    let _ = args.pop_front();

    let mut url_list: Vec<String> = vec![];
    let mut list_of_overlay_providers: Vec<Box<dyn OverlayTextProvider>> = Vec::new();

    while !args.is_empty() {
        let parameter = args.pop_front().unwrap();
        if parameter.eq_ignore_ascii_case("--overlay:time") {
            list_of_overlay_providers.push(Box::new(TimeOverlayTextProvider::new()));
        } else if parameter.starts_with("--overlay:temperature=") {
            // TODO: make a better argument parsing mechanism
            let sensor_id = parameter.split("--overlay:temperature=").last().unwrap();
            list_of_overlay_providers
                .push(Box::new(TemperatureOverlayTextProvider::new(sensor_id)));
        } else {
            url_list.push(parameter);
        }
    }

    if !url_list.is_empty() {
        Some((url_list, list_of_overlay_providers))
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
