//! WebKitGTK tab audio: native mute and audible state. Per-frame media control, volume and
//! Picture-in-Picture depend on WebView2 execution contexts and are not available here.
use super::{current, MediaService};
use tauri::{AppHandle, Manager};

pub async fn set_muted(app: &AppHandle, view: &tauri::Webview, muted: bool) -> Result<(), String> {
    let label = view.label().to_owned();
    let token = app
        .state::<crate::state::browser_state::BrowserState>()
        .security()
        .keyboard_bridge_tokens()
        .lock()
        .map_err(|_| "media.error")?
        .get(&label)
        .cloned()
        .ok_or("media.stale")?;
    let app = app.clone();
    let (send, receive) = tokio::sync::oneshot::channel();
    view.with_webview(move |platform| {
        use webkit2gtk::WebViewExt;
        let result = if current(&app, &label, &token) {
            platform.inner().set_is_muted(muted);
            Ok(())
        } else {
            Err("media.stale".to_string())
        };
        crate::error_log::ignored(send.send(result));
    })
    .map_err(|_| "media.unavailable")?;
    tokio::time::timeout(std::time::Duration::from_secs(5), receive)
        .await
        .map_err(|_| "media.error")?
        .map_err(|_| "media.error")?
}

pub fn install(
    app: &AppHandle,
    view: &tauri::Webview,
    id: &str,
    token: &str,
) -> Result<(), String> {
    let (app, id, label, token) = (
        app.clone(),
        id.to_owned(),
        view.label().to_owned(),
        token.to_owned(),
    );
    view.with_webview(move |platform| {
        use webkit2gtk::WebViewExt;
        let webview = platform.inner();
        let initial = app.state::<MediaService>().tab(&id);
        webview.set_is_muted(initial.muted || initial.volume == 0);
        app.state::<MediaService>().update(&app, &id, |t| {
            t.audible = false;
            t.mute_available = true;
        });
        webview.connect_is_playing_audio_notify(move |webview| {
            if current(&app, &label, &token) {
                let audible = webview.is_playing_audio();
                app.state::<MediaService>()
                    .update(&app, &id, |t| t.audible = audible);
                super::signals::sync_activity(&app, &id);
            }
        });
    })
    .map_err(|_| "media.unavailable".into())
}
