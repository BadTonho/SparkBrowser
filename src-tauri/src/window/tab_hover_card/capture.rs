//! Small picture of the visible part of a page, as a `data:` URL, for the tab hover card.
//! WebView2 uses the DevTools `Page.captureScreenshot` method at full size, scaled down here;
//! WebKitGTK takes a snapshot of the visible region, scaled down with cairo.
use std::time::Duration;

/// Width in pixels of the picture: twice the card's picture area, so it stays sharp on HiDPI screens.
const WIDTH: f64 = 528.0;
const TIMEOUT: Duration = Duration::from_secs(5);

async fn wait<T: Send + 'static>(
    receive: std::sync::mpsc::Receiver<Result<T, String>>,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || {
        receive
            .recv_timeout(TIMEOUT)
            .map_err(|error| format!("no picture within {TIMEOUT:?}: {error}"))?
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(windows)]
pub async fn thumbnail(view: &tauri::Webview) -> Result<String, String> {
    use tauri::webview::PlatformWebview;
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
    use windows_core::PCWSTR;

    type Done = Box<dyn FnOnce(Result<String, String>) + 'static>;

    fn call(core: &ICoreWebView2, method: &str, params: serde_json::Value, done: Done) {
        let handler =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |status, result| {
                done(
                    status
                        .map(|()| result.to_string())
                        .map_err(|error| error.to_string()),
                );
                Ok(())
            }));
        let method: Vec<u16> = method.encode_utf16().chain(Some(0)).collect();
        let params: Vec<u16> = params.to_string().encode_utf16().chain(Some(0)).collect();
        // On failure the handler is never called; the caller's timeout reports it.
        crate::error_log::ignored(unsafe {
            core.CallDevToolsProtocolMethod(
                PCWSTR(method.as_ptr()),
                PCWSTR(params.as_ptr()),
                &handler,
            )
        });
    }

    let (send, receive) = std::sync::mpsc::channel::<Result<String, String>>();
    view.with_webview(move |platform: PlatformWebview| {
        let core = match unsafe { platform.controller().CoreWebView2() } {
            Ok(core) => core,
            Err(error) => {
                crate::error_log::ignored(send.send(Err(error.to_string())));
                return;
            }
        };
        // No `clip`: with a clip or scale, Chromium re-renders the page at another size for the
        // capture, which makes the page on screen flash. The plain capture copies what is drawn.
        call(
            &core,
            "Page.captureScreenshot",
            serde_json::json!({ "format": "jpeg", "quality": 80, "fromSurface": true }),
            Box::new(move |shot| {
                let picture = shot.and_then(|text| {
                    let json: serde_json::Value =
                        serde_json::from_str(&text).map_err(|error| error.to_string())?;
                    json["data"]
                        .as_str()
                        .filter(|data| !data.is_empty())
                        .map(str::to_owned)
                        .ok_or_else(|| "the engine returned no image".to_string())
                });
                crate::error_log::ignored(send.send(picture));
            }),
        );
    })
    .map_err(|error| error.to_string())?;
    let full = wait(receive).await?;
    tauri::async_runtime::spawn_blocking(move || shrink_jpeg(&full))
        .await
        .map_err(|error| error.to_string())?
}

/// Scales a full-size JPEG capture (base64) down to the card size.
#[cfg(windows)]
fn shrink_jpeg(encoded: &str) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|error| error.to_string())?;
    let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
        .map_err(|error| error.to_string())?;
    if image.width() == 0 || image.height() == 0 {
        return Err("the page has no visible area".into());
    }
    let image = if f64::from(image.width()) > WIDTH {
        image.resize(
            WIDTH as u32,
            u32::MAX,
            image::imageops::FilterType::Triangle,
        )
    } else {
        image
    };
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 75)
        .encode_image(&image.to_rgb8())
        .map_err(|error| error.to_string())?;
    Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(out)))
}

#[cfg(target_os = "linux")]
pub async fn thumbnail(view: &tauri::Webview) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use tauri::webview::PlatformWebview;
    use webkit2gtk::{SnapshotOptions, SnapshotRegion, WebViewExt};

    fn scaled_png(surface: cairo::Surface) -> Result<Vec<u8>, String> {
        let image = cairo::ImageSurface::try_from(surface)
            .map_err(|_| "the snapshot is not an image".to_string())?;
        let (width, height) = (image.width(), image.height());
        if width < 1 || height < 1 {
            return Err("the page has no visible area".into());
        }
        let scale = (WIDTH / f64::from(width)).min(1.0);
        let target = cairo::ImageSurface::create(
            cairo::Format::Rgb24,
            ((f64::from(width) * scale).round() as i32).max(1),
            ((f64::from(height) * scale).round() as i32).max(1),
        )
        .map_err(|error| error.to_string())?;
        {
            let context = cairo::Context::new(&target).map_err(|error| error.to_string())?;
            context.scale(scale, scale);
            context
                .set_source_surface(&image, 0.0, 0.0)
                .map_err(|error| error.to_string())?;
            context.paint().map_err(|error| error.to_string())?;
        }
        let mut bytes = Vec::new();
        target
            .write_to_png(&mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }

    let (send, receive) = std::sync::mpsc::channel::<Result<String, String>>();
    view.with_webview(move |platform: PlatformWebview| {
        platform.inner().snapshot(
            SnapshotRegion::Visible,
            SnapshotOptions::NONE,
            None::<&webkit2gtk::gio::Cancellable>,
            move |result| {
                let picture = result
                    .map_err(|error| error.to_string())
                    .and_then(scaled_png)
                    .map(|png| format!("data:image/png;base64,{}", STANDARD.encode(png)));
                crate::error_log::ignored(send.send(picture));
            },
        );
    })
    .map_err(|error| error.to_string())?;
    wait(receive).await
}

#[cfg(not(any(windows, target_os = "linux")))]
pub async fn thumbnail(_view: &tauri::Webview) -> Result<String, String> {
    Err("page pictures are not supported on this platform".into())
}
