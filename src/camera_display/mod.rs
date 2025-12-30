use crate::overlay_text_providers::OverlayTextProvider;
use opencv::core::{Mat, MatTraitConst, Point, Vector};
use opencv::imgproc::put_text;
use opencv::videoio::{VideoCapture, VideoCaptureTrait};
use opencv::{highgui, imgproc};
use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};

const WINDOW_NAME: &str = "Main Camera";

const TIME_DIFF_MS: u16 = 500;
const FRAME_DURATION_MS: u16 = 1000 / 30; // 30 frames per second
const N_FRAMES_MOVEMENT_CHECK: u16 = 5; // Number of frames between each movement check

const IMG_DIFF_THRESHOLD: f64 = 10.0;
const MIN_CONTOUR_AREA: f64 = 10000.0;

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

type VideoResult<T> = Result<T, VideoStreamError>;

struct ImageFrame {
    image: Mat,
    instant: Instant,
}

pub struct VideoStreamer {
    camera: VideoCapture,
    frame_buffer: VecDeque<ImageFrame>,
    overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
}

fn log_if_err<T>(result: opencv::Result<T>, label: Option<&str>) {
    if let Err(e) = result {
        eprintln!("{} error: {}", label.unwrap_or("Unknown"), e);
    }
}

impl VideoStreamer {
    pub fn new(
        url: String,
        overlay_providers: Vec<Box<dyn OverlayTextProvider>>,
    ) -> VideoResult<Self> {
        let camera = VideoCapture::from_file(url.as_str(), opencv::videoio::CAP_ANY)?;

        Ok(Self {
            camera,
            frame_buffer: VecDeque::new(),
            overlay_providers,
        })
    }

    pub fn start_stream(&mut self) -> VideoResult<()> {
        let create_window = highgui::named_window(WINDOW_NAME, highgui::WND_PROP_FULLSCREEN);
        if let Err(e) = create_window {
            return Err(VideoStreamError::CreateWindowError(e));
        }

        log_if_err(
            highgui::set_window_property(
                WINDOW_NAME,
                highgui::WND_PROP_FULLSCREEN,
                highgui::WINDOW_FULLSCREEN as f64,
            ),
            Some("Set window property"),
        );

        let mut contours: Vector<Vector<Point>> = Vector::<Vector<Point>>::default();
        let mut frame_index: u32 = 0;
        loop {
            let mut image = Mat::default();
            self.camera.read(&mut image)?;

            let update_movement = frame_index % N_FRAMES_MOVEMENT_CHECK as u32 == 0;

            if update_movement {
                println!("Updating movement. Frame index={}", frame_index);

                let background = self.get_background_image();

                self.frame_buffer.push_back(ImageFrame {
                    image: to_gray_image(&image)?,
                    instant: Instant::now(),
                });

                let current_frame = self.frame_buffer.back().unwrap();
                if let Some(background_image) = background {
                    let img_diff = get_image_diff(&current_frame.image, &background_image.image)?;

                    println!("Background image found at {:?}", background_image.instant);
                    contours = get_movement_contours(&img_diff)?;
                } else {
                    println!("No background image found for {:?}", current_frame.instant);
                }
            }

            draw_contours(&contours, &mut image)?;

            self.draw_overlays(&mut image);

            highgui::imshow(WINDOW_NAME, &image)?;
            if highgui::wait_key(FRAME_DURATION_MS as i32)? == 'q' as i32 {
                break;
            }
            frame_index = frame_index.wrapping_add(1);
        }
        Ok(())
    }

    fn draw_overlays(&mut self, mut image: &mut Mat) {
        let mut i: u8 = 0;
        for overlay_provider in &self.overlay_providers {
            let text = overlay_provider.get_text();
            write_text(&mut image, i, &text);
            if i == u8::MAX {
                break;
            }
            i += 1;
        }
    }

    fn get_background_image(&mut self) -> Option<ImageFrame> {
        let background_instant =
            Instant::now() - Duration::from_millis((TIME_DIFF_MS + FRAME_DURATION_MS) as u64);

        while !self.frame_buffer.is_empty()
            && self.frame_buffer.front().unwrap().instant < background_instant
        {
            let background_image = self.frame_buffer.pop_front().unwrap();
            let next_frame_instant = background_image.instant
                + Duration::from_millis((FRAME_DURATION_MS * N_FRAMES_MOVEMENT_CHECK) as u64);
            if next_frame_instant > background_instant {
                return Some(background_image);
            }
        }
        println!(
            "No background image found. Frame buffer size={}",
            self.frame_buffer.len()
        );
        None
    }
}

fn get_image_diff(image: &Mat, background_image: &Mat) -> VideoResult<Mat> {
    // compare image with background
    let mut diff = Mat::default();
    opencv::core::absdiff(&image, &background_image, &mut diff)?;

    // filter the diff img to get only differences larger than the defined threshold
    let mut thresh_diff = Mat::default();
    imgproc::threshold(
        &diff,
        &mut thresh_diff,
        IMG_DIFF_THRESHOLD,
        255.0,
        imgproc::THRESH_BINARY,
    )?;

    Ok(thresh_diff)
}

fn write_text(image: &mut Mat, line: u8, text: &str) {
    let line_height: i32 = 30;
    let line_offset: i32 = 40;
    let origin = Point::new(
        10,
        image.size().unwrap().height - (line as i32 * line_height + line_offset),
    );

    let color = opencv::core::Scalar::new(0.0, 255.0, 0.0, 0.0);
    let _ = put_text(
        image,
        text,
        origin,
        imgproc::FONT_HERSHEY_SIMPLEX,
        1.0,
        color,
        2,
        imgproc::LINE_8,
        false,
    );
}

fn get_movement_contours(img_diff: &Mat) -> VideoResult<Vector<Vector<Point>>> {
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_RECT,
        opencv::core::Size {
            width: 3,
            height: 3,
        },
        Point { x: -1, y: -1 },
    )?;

    let mut dilated = Mat::default();

    imgproc::dilate(
        &img_diff,
        &mut dilated,
        &kernel,
        Point { x: -1, y: -1 },
        2,
        opencv::core::BorderTypes::BORDER_CONSTANT as i32,
        opencv::core::Scalar::default(),
    )?;

    let mut contours = Vector::<Vector<Point>>::new();
    imgproc::find_contours_def(
        &dilated,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
    )?;

    Ok(contours)
}

fn draw_contours(contours: &Vector<Vector<Point>>, image: &mut Mat) -> VideoResult<u32> {
    let mut moving_parts = 0;
    for contour in contours {
        let area = imgproc::contour_area(&contour, false)?;

        if area < MIN_CONTOUR_AREA {
            continue;
        }
        moving_parts += 1;

        let contour_rect = imgproc::bounding_rect(&contour)?;

        let color = opencv::core::Scalar {
            0: [0.0, 0.0, 255.0, 0.0],
        };
        imgproc::rectangle(
            image,
            contour_rect,
            color,
            3,
            imgproc::LineTypes::LINE_8 as i32,
            0,
        )?;
    }

    Ok(moving_parts)
}

fn to_gray_image(image: &Mat) -> VideoResult<Mat> {
    let mut gray = Mat::default();
    imgproc::cvt_color(&image, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let blur_k_size = opencv::core::Size {
        width: 19,
        height: 19,
    };
    let mut blurred_gray = Mat::default();
    imgproc::gaussian_blur_def(&gray, &mut blurred_gray, blur_k_size, 0.0)?;

    Ok(blurred_gray)
}
