//! Small pictures that tell the videos of a page apart. The popover only accepts bounded PNG bitmaps.
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::Cursor;

/// The popover rejects anything larger.
const SIDE: u32 = 96;
/// A frame arrives from the page, so its size is bounded before decoding.
pub const MAX_FRAME_TEXT: usize = 64 * 1024;
pub const MAX_POSTER_BYTES: usize = 4 * 1024 * 1024;

fn decode(bytes: &[u8]) -> Option<image::DynamicImage> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().ok()
}

fn encode(picture: &image::DynamicImage) -> Option<String> {
    let mut png = Cursor::new(Vec::new());
    picture
        .thumbnail(SIDE, SIDE)
        .to_rgb8()
        .write_to(&mut png, image::ImageFormat::Png)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(png.into_inner())
    ))
}

/// A frame captured before the video drew anything is one flat colour and shows nothing.
fn blank(picture: &image::DynamicImage) -> bool {
    let pixels = picture.thumbnail(16, 16).to_luma8();
    let (low, high) = pixels
        .pixels()
        .fold((u8::MAX, u8::MIN), |(low, high), pixel| {
            (low.min(pixel[0]), high.max(pixel[0]))
        });
    high.saturating_sub(low) < 8
}

/// From the `data:image/jpeg;base64,` frame the page drew.
pub fn from_frame(frame: &str) -> Option<String> {
    if frame.len() > MAX_FRAME_TEXT {
        return None;
    }
    let bytes = STANDARD
        .decode(frame.strip_prefix("data:image/jpeg;base64,")?)
        .ok()?;
    let picture = decode(&bytes)?;
    (!blank(&picture)).then(|| encode(&picture)).flatten()
}

/// From the bytes of the video's poster picture.
pub fn from_poster(bytes: &[u8]) -> Option<String> {
    encode(&decode(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(flat: bool, width: u32, height: u32) -> Vec<u8> {
        let picture = image::RgbImage::from_fn(width, height, |x, _| {
            image::Rgb([if flat { 0 } else { (x % 256) as u8 }; 3])
        });
        let mut bytes = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(picture)
            .write_to(&mut bytes, image::ImageFormat::Jpeg)
            .unwrap();
        bytes.into_inner()
    }
    fn size(thumbnail: &str) -> (u32, u32) {
        let png = STANDARD
            .decode(thumbnail.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        let picture = decode(&png).unwrap();
        (picture.width(), picture.height())
    }

    #[test]
    fn makes_a_small_png_and_ignores_blank_or_foreign_frames() {
        let frame = |flat| {
            format!(
                "data:image/jpeg;base64,{}",
                STANDARD.encode(picture(flat, 192, 108))
            )
        };
        assert_eq!(size(&from_frame(&frame(false)).unwrap()), (96, 54));
        assert_eq!(from_frame(&frame(true)), None);
        assert_eq!(from_frame("https://example.test/a.jpg"), None);
        assert_eq!(
            size(&from_poster(&picture(false, 640, 1280)).unwrap()),
            (48, 96)
        );
        assert_eq!(from_poster(b"<html>"), None);
    }
}
