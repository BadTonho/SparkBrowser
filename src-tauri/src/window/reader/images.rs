//! Article images for the reader view.
//!
//! The reader is drawn in the shell, whose content security policy only allows `data:` and `blob:`
//! images, so a `https://` picture from the article would show as broken. The images are instead
//! read from the tab's own resource cache through the DevTools protocol: nothing is downloaded
//! again, no request leaves the browser outside the page's network stack, and an image the page
//! never loaded is simply left out. Reader extraction exists on Windows only.
#![cfg_attr(not(windows), allow(dead_code))]
use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Most images one article carries into the reader.
pub const MAX_IMAGES: usize = 40;
/// Largest single image kept, in bytes.
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
/// Largest total of image bytes kept for one article.
const MAX_TOTAL_BYTES: usize = 24 * 1024 * 1024;

/// The image type of `bytes`, read from its signature. Only the types the reader sanitizer accepts.
fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Why a cached resource was not turned into an image.
#[derive(Debug, PartialEq, Eq)]
pub enum Skipped {
    Unreadable,
    NotAnImage,
    TooLarge,
}

/// Turns a `Page.getResourceContent` reply into a `data:` URL. `total` is the running byte count of
/// the article's images and grows by the size of the image when it is kept.
pub fn data_url(reply: &serde_json::Value, total: &mut usize) -> Result<String, Skipped> {
    let content = reply["content"].as_str().ok_or(Skipped::Unreadable)?;
    let bytes = if reply["base64Encoded"].as_bool().unwrap_or(false) {
        STANDARD.decode(content).map_err(|_| Skipped::Unreadable)?
    } else {
        content.as_bytes().to_vec()
    };
    let mime = image_mime(&bytes).ok_or(Skipped::NotAnImage)?;
    if bytes.len() > MAX_IMAGE_BYTES || *total + bytes.len() > MAX_TOTAL_BYTES {
        return Err(Skipped::TooLarge);
    }
    *total += bytes.len();
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(&bytes)))
}

/// Calls one DevTools protocol method on the tab and returns its JSON reply.
#[cfg(windows)]
async fn devtools(
    view: &tauri::Webview,
    method: &'static str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use crate::sync::Mutex;
    use std::sync::Arc;
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows_core::PCWSTR;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let send = move |slot: &Arc<Mutex<Option<tokio::sync::oneshot::Sender<_>>>>, value| {
        if let Ok(mut lock) = slot.lock() {
            if let Some(tx) = lock.take() {
                crate::error_log::ignored(tx.send(value));
            }
        }
    };
    let callback = sender.clone();
    view.with_webview(move |platform| {
        let handler =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |status, raw| {
                let value = status.map_err(|e| e.to_string()).and_then(|()| {
                    serde_json::from_str::<serde_json::Value>(&raw.to_string())
                        .map_err(|e| e.to_string())
                });
                send(&callback, value);
                Ok(())
            }));
        let method_wide: Vec<u16> = method.encode_utf16().chain(Some(0)).collect();
        let params_wide: Vec<u16> = params.to_string().encode_utf16().chain(Some(0)).collect();
        let result = unsafe { platform.controller().CoreWebView2() }.and_then(|core| unsafe {
            core.CallDevToolsProtocolMethod(
                PCWSTR(method_wide.as_ptr()),
                PCWSTR(params_wide.as_ptr()),
                &handler,
            )
        });
        if let Err(error) = result {
            send(&sender, Err(error.to_string()));
        }
    })
    .map_err(|e| e.to_string())?;
    tokio::time::timeout(std::time::Duration::from_secs(5), receiver)
        .await
        .map_err(|_| format!("{method} timed out"))?
        .map_err(|_| format!("{method} reply was lost"))?
}

/// Reads the article's images from the tab's cache, keyed by their address in the article. Images
/// the tab does not hold are left out, and the reason is written to the error log as `Info`.
#[cfg(windows)]
pub async fn cached(
    view: &tauri::Webview,
    urls: Vec<String>,
) -> std::collections::HashMap<String, String> {
    use crate::error_log::{report, Level};
    use serde_json::json;

    let mut images = std::collections::HashMap::new();
    if urls.is_empty() {
        return images;
    }
    let frame = async {
        devtools(view, "Page.enable", json!({})).await?;
        let tree = devtools(view, "Page.getResourceTree", json!({})).await?;
        tree["frameTree"]["frame"]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "Page.getResourceTree returned no main frame".to_string())
    }
    .await;
    let frame = match frame {
        Ok(frame) => frame,
        Err(error) => {
            report(
                Level::Warning,
                "native",
                "Reader view could not read the page's images; the article is shown without them",
                &error,
            );
            return images;
        }
    };
    let (mut total, mut missing, mut rejected) = (0, 0, 0);
    for url in urls.into_iter().take(MAX_IMAGES) {
        let params = json!({ "frameId": frame, "url": url });
        match devtools(view, "Page.getResourceContent", params).await {
            Ok(reply) => match data_url(&reply, &mut total) {
                Ok(data) => {
                    images.insert(url, data);
                }
                Err(_) => rejected += 1,
            },
            Err(_) => missing += 1,
        }
    }
    if missing + rejected > 0 {
        report(
            Level::Info,
            "native",
            &format!(
                "Reader view left out {missing} image(s) the page had not loaded and {rejected} unsupported or too large"
            ),
            "",
        );
    }
    images
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn reply(bytes: &[u8]) -> serde_json::Value {
        json!({ "content": STANDARD.encode(bytes), "base64Encoded": true })
    }

    #[test]
    fn known_image_signatures_become_data_urls_and_count_their_size() {
        let mut total = 0;
        let png = b"\x89PNG\r\n\x1a\nrest";
        let url = data_url(&reply(png), &mut total).unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
        assert_eq!(total, png.len());
        assert!(data_url(&reply(&[0xFF, 0xD8, 0xFF, 0xE0]), &mut total)
            .unwrap()
            .starts_with("data:image/jpeg;base64,"));
        assert!(data_url(&reply(b"GIF89a.."), &mut total)
            .unwrap()
            .starts_with("data:image/gif;base64,"));
        assert!(data_url(&reply(b"RIFF\0\0\0\0WEBPVP8 "), &mut total)
            .unwrap()
            .starts_with("data:image/webp;base64,"));
    }

    #[test]
    fn svg_html_and_unreadable_replies_are_refused() {
        let mut total = 0;
        let svg = json!({ "content": "<svg onload=alert(1)></svg>", "base64Encoded": false });
        assert_eq!(data_url(&svg, &mut total), Err(Skipped::NotAnImage));
        assert_eq!(
            data_url(&json!({ "base64Encoded": true }), &mut total),
            Err(Skipped::Unreadable)
        );
        assert_eq!(
            data_url(
                &json!({ "content": "not base64!", "base64Encoded": true }),
                &mut total
            ),
            Err(Skipped::Unreadable)
        );
        assert_eq!(total, 0);
    }

    #[test]
    fn images_over_the_single_or_total_limit_are_left_out() {
        let mut big = b"\x89PNG\r\n\x1a\n".to_vec();
        big.resize(MAX_IMAGE_BYTES + 1, 0);
        let mut total = 0;
        assert_eq!(data_url(&reply(&big), &mut total), Err(Skipped::TooLarge));
        let mut total = MAX_TOTAL_BYTES - 4;
        assert_eq!(
            data_url(&reply(b"\x89PNG\r\n\x1a\n"), &mut total),
            Err(Skipped::TooLarge)
        );
        assert_eq!(total, MAX_TOTAL_BYTES - 4);
    }
}
