use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::png::PngDecoder, DynamicImage, ImageDecoder, ImageFormat};
use std::io::Cursor;

const PREFIX: &str = "data:image/png;base64,";
const SIDE: u32 = 96;
const MAX_DATA_URL: usize = 60_000;

pub fn is_thumbnail(value: &str) -> bool {
    value.len() <= MAX_DATA_URL && value.starts_with(PREFIX)
}

pub fn normalize(value: &str) -> Result<String, String> {
    fn png(value: &str) -> Option<String> {
        if !is_thumbnail(value) {
            return None;
        }
        let bytes = STANDARD.decode(value.strip_prefix(PREFIX)?).ok()?;
        let decoder = PngDecoder::new(Cursor::new(bytes)).ok()?;
        if decoder.dimensions() != (SIDE, SIDE) {
            return None;
        }
        let image = DynamicImage::from_decoder(decoder).ok()?.into_rgba8();
        let mut output = Cursor::new(Vec::new());
        image.write_to(&mut output, ImageFormat::Png).ok()?;
        let thumbnail = format!("{PREFIX}{}", STANDARD.encode(output.into_inner()));
        is_thumbnail(&thumbnail).then_some(thumbnail)
    }
    png(value).ok_or_else(|| "profile.invalid_avatar".into())
}
