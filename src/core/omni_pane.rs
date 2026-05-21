use crate::overlay_text_providers::OverlayTextProvider;
use crate::video_display::display::DisplayWindow;
use crate::video_display::image_manipulation;
use crate::video_display::video_channel::{VideoChannel, VideoChannelSettings};
use opencv::core::Mat;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use opencv::videoio::VideoCapture;

pub struct OmniPane {
    channels: Vec<VideoChannel>,
    overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    pub current_camera_index: Arc<AtomicU8>,
    pub running: Arc<AtomicBool>,
}

impl OmniPane {
    pub fn new(
        channel_urls: Vec<String>,
        overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    ) -> Self {
        let mut channels: Vec<VideoChannel> = Vec::new();

        for url in channel_urls {
            // TODO: error handling
            let camera = VideoCapture::from_file(url.as_str(), opencv::videoio::CAP_ANY).unwrap();
            channels.push(VideoChannel::new(camera, VideoChannelSettings::default()));
            println!("Added camera for url {}", url.as_str());
        }

        OmniPane {
            channels,
            overlay_providers,
            current_camera_index: Arc::new(AtomicU8::new(0)),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start(&mut self) {
        for text_provider in &mut self.overlay_providers {
            text_provider.start_service(self.running.clone());
        }

        // TODO: error handling
        let main_display = DisplayWindow::new_default().unwrap();

        while self.running.load(Ordering::Relaxed) {
            let camera_index = self.get_safe_camera_index();
            let camera_stream = &mut self.channels[camera_index as usize];

            // TODO: error handling
            let capture_start_time = Instant::now();
            let mut image = camera_stream.create_frame_image().unwrap();
            let mut duration = camera_stream.settings.get_frame_duration();

            self.draw_overlays(&mut image);

            // TODO: error handling
            main_display.display_frame(&image).unwrap();

            // TODO: error handling
            if capture_start_time.elapsed() < duration {
                duration -= capture_start_time.elapsed();
            } else {
                duration = Duration::from_millis(0);
            }
            if main_display.stop_key_pressed(duration).unwrap() {
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

    pub fn get_n_channels(&self) -> u8 {
        self.channels.len().clone() as u8
    }
}
