use crate::video_display::VideoResult;
use crate::video_display::{image_manipulation, ImageFrame};
use opencv::core::{Mat, Point, Vector};
use opencv::hub_prelude::VideoCaptureTrait;
use opencv::imgproc;
use opencv::videoio::{VideoCapture, CAP_PROP_BUFFERSIZE};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// TODO: does it make sense for these to be configurable?
const IMG_DIFF_THRESHOLD: f64 = 10.0;
const MIN_CONTOUR_AREA: f64 = 10000.0;

pub struct VideoChannelSettings {
    frame_duration: Duration,
    mvn_update_interval: Duration,
    mvn_comparison_interval: Duration,
}

impl VideoChannelSettings {
    pub(crate) fn new(
        frame_duration: Duration,
        mvn_check_interval: Duration,
        mvn_comparison_interval: Duration,
    ) -> Self {
        Self {
            frame_duration,
            mvn_update_interval: mvn_check_interval,
            mvn_comparison_interval,
        }
    }

    pub(crate) fn default() -> Self {
        // 50 frames per second
        let frame_duration = Duration::from_millis(1000 / 50);
        Self::new(
            frame_duration,
            // Check for movement every 5 frames
            frame_duration * 5,
            // Use the image from 10 frames ago to compare against the current frame
            frame_duration * 10,
        )
    }

    pub(crate) fn get_frame_duration(&self) -> Duration {
        self.frame_duration.clone()
    }
}

pub struct VideoChannel {
    pub camera: VideoCapture,
    pub settings: VideoChannelSettings,
    frame_buffer: VecDeque<ImageFrame>,
    contours: Vector<Vector<Point>>,
    last_mvn_check: Instant,
}

impl VideoChannel {
    pub(crate) fn new(mut camera: VideoCapture, settings: VideoChannelSettings) -> Self {
        /*
        Ensure the buffer is small enough that we are always reading the latest
        image from the stream.
        */
        let _ = camera.set(CAP_PROP_BUFFERSIZE, 1.0);

        Self {
            camera,
            settings,
            frame_buffer: VecDeque::new(),
            contours: Vector::new(),
            last_mvn_check: Instant::now(),
        }
    }

    fn get_background_image(&mut self) -> Option<ImageFrame> {
        let background_instant = Instant::now() - self.settings.mvn_comparison_interval;

        while !self.frame_buffer.is_empty()
            && self.frame_buffer.front().unwrap().instant < background_instant
        {
            let background_frame = self.frame_buffer.pop_front().unwrap();
            let next_frame_instant = background_instant + self.settings.frame_duration;

            if next_frame_instant > background_instant {
                return Some(background_frame);
            }
        }

        println!(
            "No background image found. Frame buffer size={}",
            self.frame_buffer.len()
        );
        None
    }

    pub(crate) fn create_frame_image(&mut self) -> VideoResult<Mat> {
        let mut image = Mat::default();
        self.camera.read(&mut image)?;

        let update_movement = self.last_mvn_check.elapsed() >= self.settings.mvn_update_interval;
        if update_movement {
            self.last_mvn_check = Instant::now();
            let background = self.get_background_image();

            self.frame_buffer.push_back(ImageFrame {
                image: image_manipulation::to_gray_image(&image)?,
                instant: Instant::now(),
            });

            let current_frame = self.frame_buffer.back().unwrap();
            if let Some(background_image) = background {
                let img_diff = get_image_diff(&current_frame.image, &background_image.image)?;
                self.contours = get_movement_contours(&img_diff)?;
            } else {
                println!("No background image found for {:?}", current_frame.instant);
            }
        }

        self.draw_contours(&mut image)?;

        Ok(image)
    }

    fn draw_contours(&mut self, image: &mut Mat) -> VideoResult<u32> {
        let mut moving_parts = 0;
        for contour in &self.contours {
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
