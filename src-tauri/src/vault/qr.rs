//! Reads the QR code of a 2FA setup from a picture (a screenshot or a saved image), so the person
//! does not have to type the key. Only a QR that holds a valid 2FA secret is accepted: any other
//! QR code (a web address, a Wi-Fi login) is refused, and its text is never returned.
use super::totp::Totp;
use image::{ImageReader, Limits};
use std::io::Cursor;

/// The largest picture accepted, so a huge file cannot exhaust memory.
pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_SIDE: u32 = 8000;

/// The 2FA secret (an `otpauth://` link) found in the QR code of a picture.
/// Stable error codes: `vault.qr_unreadable` (not a usable picture), `vault.qr_not_found` (no QR
/// code in it) and `vault.qr_not_2fa` (a QR code, but not a 2FA setup).
pub fn read(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err("vault.qr_unreadable".into());
    }
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "vault.qr_unreadable")?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let picture = reader.decode().map_err(|_| "vault.qr_unreadable")?;

    // A QR code is easier to find on a plain background: try the picture as it is, then inverted
    // (light code on a dark page, as in dark mode screenshots).
    let mut gray = picture.to_luma8();
    let mut found_any = false;
    for inverted in [false, true] {
        if inverted {
            image::imageops::invert(&mut gray);
        }
        let mut prepared = rqrr::PreparedImage::prepare(gray.clone());
        for grid in prepared.detect_grids() {
            let Ok((_, text)) = grid.decode() else {
                continue;
            };
            found_any = true;
            let text = text.trim();
            if text.to_ascii_lowercase().starts_with("otpauth://") && Totp::parse(text).is_ok() {
                return Ok(text.to_string());
            }
        }
    }
    Err(if found_any {
        "vault.qr_not_2fa".into()
    } else {
        "vault.qr_not_found".into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Luma};
    use qrcode::QrCode;

    fn picture(text: &str, format: ImageFormat, invert: bool) -> Vec<u8> {
        let code = QrCode::new(text.as_bytes()).unwrap();
        let mut image = code
            .render::<Luma<u8>>()
            .min_dimensions(300, 300)
            .quiet_zone(true)
            .build();
        if invert {
            image::imageops::invert(&mut image);
        }
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    const LINK: &str = "otpauth://totp/Example:me@example.test?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Example";

    #[test]
    fn it_reads_a_2fa_qr_code_from_common_picture_formats() {
        for format in [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Bmp] {
            assert_eq!(
                read(&picture(LINK, format, false)).unwrap(),
                LINK,
                "{format:?}"
            );
        }
    }

    #[test]
    fn a_light_code_on_a_dark_page_is_read_too() {
        assert_eq!(read(&picture(LINK, ImageFormat::Png, true)).unwrap(), LINK);
    }

    #[test]
    fn other_qr_codes_are_refused_and_their_text_is_not_returned() {
        let web = picture(
            "https://example.test/login?token=abc",
            ImageFormat::Png,
            false,
        );
        assert_eq!(read(&web).unwrap_err(), "vault.qr_not_2fa");
        let broken = picture("otpauth://totp/x?secret=!!", ImageFormat::Png, false);
        assert_eq!(read(&broken).unwrap_err(), "vault.qr_not_2fa");
    }

    #[test]
    fn pictures_without_a_code_or_that_are_not_pictures_are_refused() {
        let blank = image::GrayImage::from_pixel(200, 200, Luma([255]));
        let mut out = Cursor::new(Vec::new());
        blank.write_to(&mut out, ImageFormat::Png).unwrap();
        assert_eq!(read(&out.into_inner()).unwrap_err(), "vault.qr_not_found");
        for bad in [&b""[..], b"not a picture", &[0u8; 64]] {
            assert_eq!(read(bad).unwrap_err(), "vault.qr_unreadable");
        }
        assert_eq!(
            read(&vec![0u8; MAX_IMAGE_BYTES + 1]).unwrap_err(),
            "vault.qr_unreadable"
        );
    }
}
