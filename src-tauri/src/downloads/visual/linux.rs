//! Download thumbnails on Linux through gdk-pixbuf, the counterpart of the Windows WIC path.
//! The decoder is chosen from the file header among four explicit formats, never by file name,
//! and the image is scaled while it is decoded so a large picture is never fully held in memory.
use gtk::gdk_pixbuf::{prelude::*, PixbufLoader};
use std::cell::Cell;

fn format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.starts_with(b"BM") {
        Some("bmp")
    } else {
        None
    }
}

pub(super) fn supported_header(bytes: &[u8]) -> bool {
    format(bytes).is_some()
}

pub(super) fn thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    let loader = PixbufLoader::with_type(format(bytes)?).ok()?;
    let rejected = std::rc::Rc::new(Cell::new(false));
    let flag = rejected.clone();
    loader.connect_size_prepared(move |loader, width, height| {
        match super::thumbnail_size(width.max(0) as u32, height.max(0) as u32) {
            Some((width, height)) => loader.set_size(width as i32, height as i32),
            None => {
                // Oversized or empty: decode to a single pixel and discard the result.
                flag.set(true);
                loader.set_size(1, 1);
            }
        }
    });
    let decoded = loader.write(bytes).is_ok();
    // Always close, even after a failed write, to release the loader.
    let closed = loader.close().is_ok();
    if !decoded || !closed || rejected.get() {
        return None;
    }
    let png = loader.pixbuf()?.save_to_bufferv("png", &[]).ok()?;
    (png.len() <= 64 * 1024).then_some(png)
}
