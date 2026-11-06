//! Native-only feasibility probe for full-document PNG capture.
//! The command is restricted to the isolated native test application.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::{io::Cursor, path::PathBuf, time::Instant};

#[derive(Debug, Serialize)]
pub struct CaptureProbeResult {
    pub engine: &'static str,
    pub width: u32,
    pub height: u32,
    pub png_bytes: usize,
    pub elapsed_ms: u128,
    pub artifact: &'static str,
}

fn artifact_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if app.config().identifier != "org.veloce.browser.native-tests" {
        return Err("capture_probe.test_application_only".into());
    }
    let requested = std::env::var_os("VELOCE_NATIVE_TEST_RUN_DIR")
        .ok_or("capture_probe.missing_test_directory")?;
    let run = PathBuf::from(requested)
        .canonicalize()
        .map_err(|_| "capture_probe.invalid_test_directory")?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.cache/native-runs")
        .canonicalize()
        .map_err(|_| "capture_probe.invalid_test_root")?;
    if !run.starts_with(&root) || run == root {
        return Err("capture_probe.invalid_test_directory".into());
    }
    let artifacts = run
        .join("artifacts")
        .canonicalize()
        .map_err(|_| "capture_probe.invalid_test_directory")?;
    if !artifacts.starts_with(&run) {
        return Err("capture_probe.invalid_test_directory".into());
    }
    Ok(artifacts.join("full-page-probe.png"))
}

fn save_png(
    path: &std::path::Path,
    bytes: &[u8],
    started: Instant,
    engine: &'static str,
) -> Result<CaptureProbeResult, String> {
    if bytes.len() < 8 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("capture_probe.invalid_png".into());
    }
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let reader = decoder.read_info().map_err(|error| error.to_string())?;
    let width = reader.info().width;
    let height = reader.info().height;
    std::fs::write(path, bytes).map_err(|error| error.to_string())?;
    Ok(CaptureProbeResult {
        engine,
        width,
        height,
        png_bytes: bytes.len(),
        elapsed_ms: started.elapsed().as_millis(),
        artifact: "artifacts/full-page-probe.png",
    })
}

#[tauri::command]
pub async fn full_page_capture_probe(app: tauri::AppHandle) -> Result<CaptureProbeResult, String> {
    let path = artifact_path(&app)?;
    let view = crate::window::webview_manager::active_content_webview(&app)
        .ok_or("capture_probe.active_webview_missing")?;
    let started = Instant::now();

    #[cfg(windows)]
    let bytes = capture_webview2(&view).await?;
    #[cfg(target_os = "linux")]
    let bytes = capture_webkitgtk(&view).await?;
    #[cfg(not(any(windows, target_os = "linux")))]
    let bytes: Vec<u8> = return Err("capture_probe.unsupported_platform".into());

    save_png(&path, &bytes, started, engine_name())
}

#[cfg(windows)]
fn engine_name() -> &'static str {
    "WebView2 CDP Page.captureScreenshot"
}

#[cfg(target_os = "linux")]
fn engine_name() -> &'static str {
    "WebKitGTK FULL_DOCUMENT snapshot"
}

#[cfg(not(any(windows, target_os = "linux")))]
fn engine_name() -> &'static str {
    "unsupported"
}

#[cfg(windows)]
async fn capture_webview2(view: &tauri::Webview) -> Result<Vec<u8>, String> {
    use std::{sync::mpsc, time::Duration};
    use tauri::webview::PlatformWebview;
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows_core::PCWSTR;

    let (send, receive) = mpsc::channel();
    view.with_webview(move |platform: PlatformWebview| {
        let result = unsafe { platform.controller().CoreWebView2() };
        let core = match result {
            Ok(core) => core,
            Err(error) => {
                let _ = send.send(Err(error.to_string()));
                return;
            }
        };
        let callback_send = send.clone();
        let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |status, result| {
            let output = status.map_err(|error| error.to_string()).and_then(|()| {
                let json: serde_json::Value = serde_json::from_str(&result.to_string()).map_err(|error| error.to_string())?;
                let data = json["data"].as_str().ok_or("capture_probe.no_image_data")?;
                STANDARD.decode(data).map_err(|error| error.to_string())
            });
            let _ = callback_send.send(output);
            Ok(())
        }));
        let method: Vec<u16> = "Page.captureScreenshot".encode_utf16().chain(Some(0)).collect();
        let params: Vec<u16> = serde_json::json!({ "format": "png", "captureBeyondViewport": true, "fromSurface": true })
            .to_string().encode_utf16().chain(Some(0)).collect();
        let result = unsafe { core.CallDevToolsProtocolMethod(PCWSTR(method.as_ptr()), PCWSTR(params.as_ptr()), &handler) };
        if let Err(error) = result {
            let _ = send.send(Err(error.to_string()));
        }
    }).map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        receive
            .recv_timeout(Duration::from_secs(45))
            .map_err(|error| error.to_string())?
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(target_os = "linux")]
async fn capture_webkitgtk(view: &tauri::Webview) -> Result<Vec<u8>, String> {
    use std::{sync::mpsc, time::Duration};
    use tauri::webview::PlatformWebview;
    use webkit2gtk::{prelude::WebViewExt, SnapshotOptions, SnapshotRegion};

    let (send, receive) = mpsc::channel();
    view.with_webview(move |platform: PlatformWebview| {
        platform.inner().snapshot(
            SnapshotRegion::FullDocument,
            SnapshotOptions::NONE,
            None::<&webkit2gtk::gio::Cancellable>,
            move |result| {
                let png = match result {
                    Err(error) => Err(error.to_string()),
                    Ok(surface) => {
                        let mut bytes = Vec::new();
                        surface
                            .write_to_png(&mut bytes)
                            .map(|()| bytes)
                            .map_err(|error| error.to_string())
                    }
                };
                let _ = send.send(png);
            },
        );
    })
    .map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        receive
            .recv_timeout(Duration::from_secs(45))
            .map_err(|error| error.to_string())?
    })
    .await
    .map_err(|error| error.to_string())?
}
