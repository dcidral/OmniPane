use crate::video_display::VideoResult;
use opencv::core::{Mat, MatTraitConst, Point, Size};
use opencv::imgproc;
use opencv::imgproc::{get_text_size, put_text};

pub(crate) enum TextPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub(crate) fn write_text(image: &mut Mat, line_index: u8, text: &str, position: TextPosition) {
    let color = opencv::core::Scalar::new(0.0, 255.0, 0.0, 0.0);
    let text_font = imgproc::FONT_HERSHEY_SIMPLEX;
    let font_scale = 1.0;
    let font_thickness = 2;
    let mut baseline = 0;
    let padding = 10;

    // TODO: Error handling
    let image_size = image.size().unwrap();
    let text_size =
        get_text_size(text, text_font, font_scale, font_thickness, &mut baseline).unwrap();

    let line_height: i32 = text_size.height + baseline + padding;
    let first_line_offset: i32 = 60;
    let line_position: i32 = line_index as i32 * line_height + first_line_offset;

    let origin = calculate_text_origin(&text_size, &image_size, padding, line_position, &position);

    let _ = put_text(
        image,
        text,
        origin,
        text_font,
        font_scale,
        color,
        font_thickness,
        imgproc::LINE_8,
        false,
    );
}

fn calculate_text_origin(
    text_size: &Size,
    image_size: &Size,
    padding: i32,
    line_position: i32,
    position: &TextPosition,
) -> Point {
    let y_padding = padding + line_position;
    match position {
        TextPosition::TopLeft => Point::new(padding, y_padding),
        TextPosition::BottomLeft => {
            Point::new(padding, image_size.height - (text_size.height + y_padding))
        }
        TextPosition::TopRight => {
            Point::new(image_size.width - (text_size.width + padding), y_padding)
        }
        TextPosition::BottomRight => Point::new(
            image_size.width - (text_size.width + y_padding),
            image_size.height - (text_size.height + y_padding),
        ),
    }
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
