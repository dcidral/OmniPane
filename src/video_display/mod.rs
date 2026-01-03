use opencv::core::Mat;
use std::fmt;
use std::time::Instant;

pub(crate) mod display;
pub(crate) mod image_manipulation;
pub(crate) mod video_channel;

struct ImageFrame {
    image: Mat,
    instant: Instant,
}

#[derive(Debug)]
pub enum VideoStreamError {
    OpenCv(opencv::Error),
    CreateWindowError(opencv::Error),
}

impl fmt::Display for VideoStreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VideoStreamError::OpenCv(e) => write!(f, "OpenCV error: {}", e),
            VideoStreamError::CreateWindowError(e) => write!(f, "Create Window error: {}", e),
        }
    }
}

impl std::error::Error for VideoStreamError {}

impl From<opencv::Error> for VideoStreamError {
    fn from(err: opencv::Error) -> Self {
        VideoStreamError::OpenCv(err)
    }
}

pub type VideoResult<T> = Result<T, VideoStreamError>;