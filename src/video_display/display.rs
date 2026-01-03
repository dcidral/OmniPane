use opencv::core::Mat;
use opencv::highgui;
pub(crate) use crate::video_display::VideoResult;
pub(crate) use crate::video_display::VideoStreamError;

const WINDOW_NAME: &str = "Main Camera";

pub(crate) struct DisplayWindow {
    frame_duration: i32,
}

impl DisplayWindow {
    pub fn new(frame_duration: i32) -> VideoResult<Self> {
        let create_window = highgui::named_window(WINDOW_NAME, highgui::WND_PROP_FULLSCREEN);
        if let Err(e) = create_window {
            return Err(VideoStreamError::CreateWindowError(e));
        }

        if let Err(e) = highgui::set_window_property(
            WINDOW_NAME,
            highgui::WND_PROP_FULLSCREEN,
            highgui::WINDOW_FULLSCREEN as f64,
        ) {
            return Err(VideoStreamError::CreateWindowError(e));
        }

        Ok(Self { frame_duration })
    }

    pub fn display_frame(&self, image: &Mat) -> VideoResult<()> {
        highgui::imshow(WINDOW_NAME, &image)?;
        Ok(())
    }

    pub fn stop_key_pressed(&self) -> VideoResult<bool> {
        if highgui::wait_key(self.frame_duration)? == 'q' as i32 {
            return Ok(true)
        }
        Ok(false)
    }
}
