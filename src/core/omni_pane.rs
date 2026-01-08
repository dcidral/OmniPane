use crate::overlay_text_providers::OverlayTextProvider;
use crate::video_display::display::DisplayWindow;
use crate::video_display::image_manipulation;
use crate::video_display::video_channel;
use crate::video_display::video_channel::VideoChannel;
use opencv::core::Mat;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;

pub struct OmniPane {
    channels: Vec<VideoChannel>,
    overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    pub current_camera_index: Arc<AtomicU8>,
}

impl OmniPane {
    pub fn new(
        channels: Vec<VideoChannel>,
        overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    ) -> Self {
        OmniPane {
            channels,
            overlay_providers,
            current_camera_index: Arc::new(AtomicU8::new(0)),
        }
    }

    pub fn start_display(&mut self, is_running: Arc<AtomicBool>) {
        // TODO: error handling
        let main_display = DisplayWindow::new(video_channel::FRAME_DURATION_MS as i32).unwrap();

        while is_running.load(Ordering::Relaxed) {
            let camera_index = self.get_safe_camera_index();
            let camera_stream = &mut self.channels[camera_index as usize];

            // TODO: error handling
            let mut image = camera_stream.create_frame_image().unwrap();

            self.draw_overlays(&mut image);

            // TODO: error handling
            main_display.display_frame(&image).unwrap();

            // TODO: error handling
            if main_display.stop_key_pressed().unwrap() {
                // TODO: exit all services
                break;
            }
        }
    }

    fn draw_overlays(&mut self, mut image: &mut Mat) {
        let mut line_index: u8 = 0;
        for overlay_provider in &self.overlay_providers {
            let text = overlay_provider.get_text();
            image_manipulation::write_text(
                &mut image,
                line_index,
                &text,
                image_manipulation::TextPosition::BottomRight,
            );
            if line_index == u8::MAX {
                break;
            }
            line_index += 1;
        }
    }

    fn get_safe_camera_index(&self) -> u8 {
        let mut current_index = self.current_camera_index.load(Ordering::Relaxed);
        if current_index >= self.channels.len() as u8 {
            println!("Wrong camera index {}", current_index);
            current_index = 0;
        }
        current_index
    }
}

// fn log_if_err<T>(result: opencv::Result<T>, label: Option<&str>) {
//     if let Err(e) = result {
//         eprintln!("{} error: {}", label.unwrap_or("Unknown"), e);
//     }
// }
