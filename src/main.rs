mod camera_display;
mod overlay_text_providers;

use crate::camera_display::VideoStreamer;
use crate::overlay_text_providers::{OverlayTextProvider, TimeOverlayTextProvider};
use std::collections::VecDeque;
use std::env;

fn main() {
    println!("Starting video streaming...");

    let mut args: VecDeque<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} \"URL\"", args[0]);
        return;
    }

    // executable's path
    let _ = args.pop_front();

    let mut url: Option<String> = None;
    let mut list_of_overlay_providers: Vec<Box<dyn OverlayTextProvider>> = Vec::new();

    while !args.is_empty() {
        let parameter = args.pop_front().unwrap();
        if parameter.eq_ignore_ascii_case("--overlay:time") {
            list_of_overlay_providers.push(Box::new(TimeOverlayTextProvider::new()));
        } else {
            url = Some(parameter);
        }
    }
    
    match url {
        None => {
            panic!("No video stream specified!");
        }
        Some(url) => match VideoStreamer::new(url, list_of_overlay_providers) {
            Ok(mut streamer) => {
                if let Err(e) = streamer.start_stream() {
                    eprintln!("Error during streaming: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to initialize streamer: {}", e),
        },
    }
}
