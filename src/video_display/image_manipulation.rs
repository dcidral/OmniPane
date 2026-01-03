use opencv::core::{Mat, MatTraitConst, Point};
use opencv::imgproc;
use opencv::imgproc::put_text;
use crate::video_display::VideoResult;

pub(crate) fn write_text(image: &mut Mat, line: u8, text: &str) {
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

pub(crate) fn to_gray_image(image: &Mat) -> VideoResult<Mat> {
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
