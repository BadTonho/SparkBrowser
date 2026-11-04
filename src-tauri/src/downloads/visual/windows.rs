use std::{
    fs::File,
    os::windows::{ffi::OsStringExt, io::AsRawHandle},
    path::PathBuf,
};
use windows::{
    core::GUID,
    Win32::{
        Foundation::HANDLE,
        Graphics::Imaging::*,
        Storage::FileSystem::{GetFinalPathNameByHandleW, GETFINALPATHNAMEBYHANDLE_FLAGS},
        System::Com::*,
    },
};

pub(super) fn final_path(file: &File) -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    let size = unsafe {
        GetFinalPathNameByHandleW(
            HANDLE(file.as_raw_handle()),
            &mut buffer,
            GETFINALPATHNAMEBYHANDLE_FLAGS(0),
        )
    } as usize;
    if size == 0 || size >= buffer.len() {
        return None;
    }
    Some(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..size],
    )))
}

fn codec(bytes: &[u8]) -> Option<GUID> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(CLSID_WICPngDecoder)
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some(CLSID_WICJpegDecoder)
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(CLSID_WICGifDecoder)
    } else if bytes.starts_with(b"BM") {
        Some(CLSID_WICBmpDecoder)
    } else {
        None
    }
}

pub(super) fn supported_header(bytes: &[u8]) -> bool {
    codec(bytes).is_some()
}

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub(super) fn thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    let codec = codec(bytes)?;
    // Explicit Microsoft codecs, never Shell thumbnail handlers or extension discovery.
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()?;
    }
    let _apartment = Apartment;
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let stream = factory.CreateStream().ok()?;
        stream.InitializeFromMemory(bytes).ok()?;
        let decoder: IWICBitmapDecoder =
            CoCreateInstance(&codec, None, CLSCTX_INPROC_SERVER).ok()?;
        decoder
            .Initialize(&stream, WICDecodeMetadataCacheOnDemand)
            .ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let (mut width, mut height) = (0, 0);
        frame.GetSize(&mut width, &mut height).ok()?;
        let (width, height) = super::thumbnail_size(width, height)?;
        let scaler = factory.CreateBitmapScaler().ok()?;
        scaler
            .Initialize(&frame, width, height, WICBitmapInterpolationModeLinear)
            .ok()?;
        let converter = factory.CreateFormatConverter().ok()?;
        converter
            .Initialize(
                &scaler,
                &GUID_WICPixelFormat32bppRGBA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )
            .ok()?;
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        converter
            .CopyPixels(std::ptr::null(), width * 4, &mut pixels)
            .ok()?;
        let mut output = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut output, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&pixels).ok()?;
        }
        (output.len() <= 64 * 1024).then_some(output)
    }
}
