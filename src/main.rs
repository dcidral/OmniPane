mod vstream;

use std::env;
use crate::vstream::VideoStreamer;

fn main() {
    println!("Starting video streaming...");

    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} \"URL\"", args[0]);
        return;
    }

    let url = args.remove(1);

    match VideoStreamer::new(url) {
        Ok(mut streamer) => {
            if let Err(e) = streamer.start_stream() {
                eprintln!("Error during streaming: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to initialize streamer: {}", e),
    }
}
