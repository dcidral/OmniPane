use crate::video_display::VideoResult;
use crate::video_display::{image_manipulation, ImageFrame};
use opencv::core::{Mat, Point, Vector};
use opencv::hub_prelude::VideoCaptureTrait;
use opencv::imgproc;
use opencv::videoio::VideoCapture;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const TIME_DIFF_MS: u16 = 500;
pub(crate) const FRAME_DURATION_MS: u16 = 1000 / 30; // 30 frames per second
const N_FRAMES_MOVEMENT_CHECK: u16 = 5; // Number of frames between each movement check

const IMG_DIFF_THRESHOLD: f64 = 10.0;
const MIN_CONTOUR_AREA: f64 = 10000.0;

pub struct VideoChannel {
    pub camera: VideoCapture,
    frame_buffer: VecDeque<ImageFrame>,
    contours: Vector<Vector<Point>>,
    frame_index: u32,
}

impl VideoChannel {
    pub(crate) fn new(camera: VideoCapture) -> Self {
        Self {
            camera,
            frame_buffer: VecDeque::new(),
            contours: Vector::new(),
            frame_index: 0
        }
    }

    fn get_background_image(&mut self) -> Option<ImageFrame> {
        let mvt_check_duration =
            Duration::from_millis(TIME_DIFF_MS as u64);

        let background_instant = Instant::now() - mvt_check_duration;

        while !self.frame_buffer.is_empty()
            && self.frame_buffer.front().unwrap().instant < background_instant
        {
            let background_frame = self.frame_buffer.pop_front().unwrap();
            let next_frame_instant =
                background_instant + Duration::from_millis(FRAME_DURATION_MS as u64);

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

        let update_movement = self.frame_index % N_FRAMES_MOVEMENT_CHECK as u32 == 0;
        self.frame_index = self.frame_index.wrapping_add(1);

        if update_movement {
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