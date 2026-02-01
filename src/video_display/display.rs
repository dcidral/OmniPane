pub(crate) use crate::video_display::VideoResult;
pub(crate) use crate::video_display::VideoStreamError;
use opencv::core::Mat;
use opencv::highgui;
use std::cmp::max;
use std::time::Duration;

pub(crate) struct DisplayWindow {
    window_name: String,
}

impl DisplayWindow {
    pub fn new(window_name: String) -> VideoResult<Self> {
        let create_window =
            highgui::named_window(window_name.as_str(), highgui::WND_PROP_FULLSCREEN);
        if let Err(e) = create_window {
            return Err(VideoStreamError::CreateWindowError(e));
        }

        if let Err(e) = highgui::set_window_property(
            window_name.as_str(),
            highgui::WND_PROP_FULLSCREEN,
            highgui::WINDOW_FULLSCREEN as f64,
        ) {
            return Err(VideoStreamError::CreateWindowError(e));
        }

        Ok(Self { window_name })
    }

    pub fn new_default() -> VideoResult<Self> {
        Self::new("Main Camera".to_string())
    }

    pub fn display_frame(&self, image: &Mat) -> VideoResult<()> {
        highgui::imshow(self.window_name.as_str(), &image)?;
        Ok(())
    }

    pub fn stop_key_pressed(&self, duration: Duration) -> VideoResult<bool> {
        let frame_duration = max(duration.as_millis() as i32, 1);
        if highgui::wait_key(frame_duration)? == 'q' as i32 {
            return Ok(true);
        }
        Ok(false)
    }
}
